//! Text editor buffer — stores content using an efficient rope structure,
//! handles cursor movement, undo/redo, visual selection, search/replace,
//! Unicode grapheme safety, and atomic saving.

use crate::editor::completion::{layered_completions, prefix_at, Completion};
use crate::filesystem::operations::save_file_atomic;
use anyhow::{bail, Result};
use ropey::Rope;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

// ── Line Endings ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineEnding {
    #[default]
    Lf,
    Crlf,
}

impl LineEnding {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::Crlf => "\r\n",
        }
    }
}

// ── Cursor & Selection ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Selection {
    pub anchor: Cursor,
    pub head: Cursor,
}

impl Selection {
    pub fn new(anchor: Cursor, head: Cursor) -> Self {
        Self { anchor, head }
    }

    pub fn is_empty(&self) -> bool {
        self.anchor == self.head
    }

    pub fn start(&self) -> Cursor {
        if self.anchor.row < self.head.row
            || (self.anchor.row == self.head.row && self.anchor.col <= self.head.col)
        {
            self.anchor
        } else {
            self.head
        }
    }

    pub fn end(&self) -> Cursor {
        if self.anchor.row > self.head.row
            || (self.anchor.row == self.head.row && self.anchor.col >= self.head.col)
        {
            self.anchor
        } else {
            self.head
        }
    }
}

// ── Completion State ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct CompletionState {
    pub active: bool,
    pub candidates: Vec<Completion>,
    pub selected: usize,
    pub prefix: String,
}

// ── Editor Buffer ───────────────────────────────────────────────────────────

#[allow(dead_code)]
pub struct Buffer {
    pub path: Option<PathBuf>,
    pub lines: Vec<String>,
    pub cursor: Cursor,
    pub selection: Option<Selection>,
    pub dirty: bool,
    pub scroll_row: usize,
    pub scroll_col: usize,
    undo_stack: Vec<(Vec<String>, Cursor)>,
    redo_stack: Vec<(Vec<String>, Cursor)>,
    /// Clipboard (application-internal).
    clipboard: String,
    /// Active search query.
    pub search_query: String,
    /// Index of current search match.
    pub search_match: usize,
    /// Autocompletion state.
    pub completion: CompletionState,
    /// Last modified time recorded when loaded or saved.
    pub loaded_mtime: Option<SystemTime>,
    /// Line ending format.
    pub line_ending: LineEnding,
    /// Internal rope structure for efficient large-document operations.
    rope: Rope,
}

impl Buffer {
    /// Create a new, empty buffer.
    pub fn new() -> Self {
        Self {
            path: None,
            lines: vec![String::new()],
            cursor: Cursor::default(),
            selection: None,
            dirty: false,
            scroll_row: 0,
            scroll_col: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            clipboard: String::new(),
            search_query: String::new(),
            search_match: 0,
            completion: CompletionState::default(),
            loaded_mtime: None,
            line_ending: LineEnding::Lf,
            rope: Rope::from_str(""),
        }
    }

    /// Load a file into the buffer, recording mtime and detecting line endings.
    pub fn from_file(path: PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        let line_ending = if content.contains("\r\n") {
            LineEnding::Crlf
        } else {
            LineEnding::Lf
        };
        let rope = Rope::from_str(&content);
        let lines: Vec<String> = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(str::to_string).collect()
        };
        let loaded_mtime = std::fs::metadata(&path).and_then(|m| m.modified()).ok();

        Ok(Self {
            path: Some(path),
            lines,
            cursor: Cursor::default(),
            selection: None,
            dirty: false,
            scroll_row: 0,
            scroll_col: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            clipboard: String::new(),
            search_query: String::new(),
            search_match: 0,
            completion: CompletionState::default(),
            loaded_mtime,
            line_ending,
            rope,
        })
    }

    /// Synchronize the internal rope with the current lines.
    pub fn sync_rope(&mut self) {
        let sep = self.line_ending.as_str();
        self.rope = Rope::from_str(&self.lines.join(sep));
    }

    /// Return total character length of the entire buffer.
    pub fn total_char_len(&self) -> usize {
        self.lines.iter().map(|l| l.chars().count() + 1).sum()
    }

    // ── Undo/Redo ────────────────────────────────────────────────────────────

    fn push_undo(&mut self) {
        self.undo_stack.push((self.lines.clone(), self.cursor));
        // Bound stack depth to prevent unbounded memory growth.
        if self.undo_stack.len() > 300 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some((lines, cursor)) = self.undo_stack.pop() {
            self.redo_stack.push((self.lines.clone(), self.cursor));
            self.lines = lines;
            self.cursor = cursor;
            self.dirty = true;
            self.clamp_cursor();
            self.sync_rope();
        }
    }

    pub fn redo(&mut self) {
        if let Some((lines, cursor)) = self.redo_stack.pop() {
            self.undo_stack.push((self.lines.clone(), self.cursor));
            self.lines = lines;
            self.cursor = cursor;
            self.dirty = true;
            self.clamp_cursor();
            self.sync_rope();
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    // ── Cursor Movement ──────────────────────────────────────────────────────

    pub fn clamp_cursor(&mut self) {
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.cursor.row = self.cursor.row.min(self.lines.len().saturating_sub(1));
        let char_count = self.current_line().chars().count();
        self.cursor.col = self.cursor.col.min(char_count);
    }

    pub fn current_line(&self) -> &str {
        self.lines
            .get(self.cursor.row)
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    pub fn move_up(&mut self) {
        if self.cursor.row > 0 {
            self.cursor.row -= 1;
        }
        self.clamp_cursor();
    }

    pub fn move_down(&mut self) {
        if self.cursor.row + 1 < self.lines.len() {
            self.cursor.row += 1;
        }
        self.clamp_cursor();
    }

    pub fn move_left(&mut self) {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        } else if self.cursor.row > 0 {
            self.cursor.row -= 1;
            self.cursor.col = self.lines[self.cursor.row].chars().count();
        }
    }

    pub fn move_right(&mut self) {
        let char_count = self.current_line().chars().count();
        if self.cursor.col < char_count {
            self.cursor.col += 1;
        } else if self.cursor.row + 1 < self.lines.len() {
            self.cursor.row += 1;
            self.cursor.col = 0;
        }
    }

    pub fn move_home(&mut self) {
        // Smart home: first non-whitespace char, then column 0.
        let first_nonws = self
            .current_line()
            .chars()
            .take_while(|c| c.is_whitespace())
            .count();
        if self.cursor.col != first_nonws {
            self.cursor.col = first_nonws;
        } else {
            self.cursor.col = 0;
        }
    }

    pub fn move_end(&mut self) {
        self.cursor.col = self.current_line().chars().count();
    }

    pub fn move_page_up(&mut self, page: usize) {
        self.cursor.row = self.cursor.row.saturating_sub(page);
        self.clamp_cursor();
    }

    pub fn move_page_down(&mut self, page: usize) {
        self.cursor.row = (self.cursor.row + page).min(self.lines.len().saturating_sub(1));
        self.clamp_cursor();
    }

    pub fn goto_line(&mut self, line: usize) {
        self.cursor.row = line
            .saturating_sub(1)
            .min(self.lines.len().saturating_sub(1));
        self.cursor.col = 0;
    }

    // ── Visual Selection ─────────────────────────────────────────────────────

    pub fn start_selection(&mut self) {
        self.selection = Some(Selection::new(self.cursor, self.cursor));
    }

    pub fn update_selection(&mut self) {
        if let Some(sel) = &mut self.selection {
            sel.head = self.cursor;
        }
    }

    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    pub fn has_selection(&self) -> bool {
        self.selection.as_ref().is_some_and(|s| !s.is_empty())
    }

    pub fn selected_text(&self) -> Option<String> {
        let sel = self.selection.as_ref()?;
        if sel.is_empty() {
            return None;
        }
        let start = sel.start();
        let end = sel.end();

        if start.row == end.row {
            let line = self.lines.get(start.row)?;
            let chars: Vec<char> = line.chars().collect();
            let start_col = start.col.min(chars.len());
            let end_col = end.col.min(chars.len());
            Some(chars[start_col..end_col].iter().collect())
        } else {
            let mut result = Vec::new();
            for r in start.row..=end.row {
                if let Some(line) = self.lines.get(r) {
                    let chars: Vec<char> = line.chars().collect();
                    if r == start.row {
                        let sc = start.col.min(chars.len());
                        result.push(chars[sc..].iter().collect::<String>());
                    } else if r == end.row {
                        let ec = end.col.min(chars.len());
                        result.push(chars[..ec].iter().collect::<String>());
                    } else {
                        result.push(line.clone());
                    }
                }
            }
            Some(result.join("\n"))
        }
    }

    pub fn delete_selection(&mut self) -> Option<String> {
        let text = self.selected_text()?;
        let sel = self.selection.take()?;
        let start = sel.start();
        let end = sel.end();

        self.push_undo();

        if start.row == end.row {
            let line = &mut self.lines[start.row];
            let mut chars: Vec<char> = line.chars().collect();
            let sc = start.col.min(chars.len());
            let ec = end.col.min(chars.len());
            chars.drain(sc..ec);
            *line = chars.into_iter().collect();
        } else {
            let mut new_lines = Vec::new();
            for (idx, line) in self.lines.iter().enumerate() {
                if idx < start.row || idx > end.row {
                    new_lines.push(line.clone());
                } else if idx == start.row {
                    let first_chars: Vec<char> = line.chars().collect();
                    let sc = start.col.min(first_chars.len());
                    let prefix: String = first_chars[..sc].iter().collect();

                    let last_line = &self.lines[end.row];
                    let last_chars: Vec<char> = last_line.chars().collect();
                    let ec = end.col.min(last_chars.len());
                    let suffix: String = last_chars[ec..].iter().collect();

                    new_lines.push(format!("{prefix}{suffix}"));
                }
            }
            self.lines = new_lines;
        }

        self.cursor = start;
        self.dirty = true;
        self.clamp_cursor();
        self.sync_rope();
        Some(text)
    }

    // ── Editing Operations (Unicode / Grapheme Safe) ─────────────────────────

    /// Insert a character at the current cursor position safely without UTF-8 slicing panics.
    pub fn insert_char(&mut self, c: char) {
        self.push_undo();
        self.insert_char_no_undo(c);
        self.dirty = true;
        self.sync_rope();
    }

    fn insert_char_no_undo(&mut self, c: char) {
        let row = self.cursor.row;
        let col = self.cursor.col;

        while self.lines.len() <= row {
            self.lines.push(String::new());
        }

        let mut chars: Vec<char> = self.lines[row].chars().collect();
        let safe_col = col.min(chars.len());
        chars.insert(safe_col, c);
        self.lines[row] = chars.into_iter().collect();
        self.cursor.col = safe_col + 1;
    }

    /// Insert arbitrary multi-line text at cursor (for snippets, completions, paste).
    pub fn insert_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.push_undo();
        for c in text.chars() {
            if c == '\n' {
                self.insert_newline_no_undo();
            } else if c != '\r' {
                self.insert_char_no_undo(c);
            }
        }
        self.dirty = true;
        self.sync_rope();
    }

    /// Insert a newline with auto-indentation matching the current line.
    pub fn insert_newline(&mut self) {
        self.push_undo();
        self.insert_newline_no_undo();
        self.dirty = true;
        self.sync_rope();
    }

    fn insert_newline_no_undo(&mut self) {
        let row = self.cursor.row;
        let col = self.cursor.col;

        while self.lines.len() <= row {
            self.lines.push(String::new());
        }

        let line = &self.lines[row];
        let chars: Vec<char> = line.chars().collect();
        let safe_col = col.min(chars.len());

        let indent: String = chars.iter().take_while(|c| c.is_whitespace()).collect();
        let prefix: String = chars[..safe_col].iter().collect();
        let suffix: String = chars[safe_col..].iter().collect();

        self.lines[row] = prefix;
        self.lines.insert(row + 1, format!("{indent}{suffix}"));
        self.cursor.row += 1;
        self.cursor.col = indent.chars().count();
    }

    /// Backspace at the current cursor position (char-safe).
    pub fn backspace(&mut self) {
        self.push_undo();
        let row = self.cursor.row;
        let col = self.cursor.col;

        if col > 0 {
            let mut chars: Vec<char> = self.lines[row].chars().collect();
            let safe_col = col.min(chars.len());
            if safe_col > 0 {
                chars.remove(safe_col - 1);
                self.lines[row] = chars.into_iter().collect();
                self.cursor.col = safe_col - 1;
            }
        } else if row > 0 {
            let cur = self.lines.remove(row);
            let prev_len = self.lines[row - 1].chars().count();
            self.lines[row - 1].push_str(&cur);
            self.cursor.row -= 1;
            self.cursor.col = prev_len;
        }
        self.dirty = true;
        self.sync_rope();
    }

    /// Delete character under cursor (char-safe).
    pub fn delete_char(&mut self) {
        self.push_undo();
        let row = self.cursor.row;
        let col = self.cursor.col;

        if row < self.lines.len() {
            let mut chars: Vec<char> = self.lines[row].chars().collect();
            if col < chars.len() {
                chars.remove(col);
                self.lines[row] = chars.into_iter().collect();
            } else if row + 1 < self.lines.len() {
                let next = self.lines.remove(row + 1);
                self.lines[row].push_str(&next);
            }
        }
        self.dirty = true;
        self.sync_rope();
    }

    /// Insert spaces for tab width.
    pub fn insert_tab(&mut self, tab_width: usize) {
        for _ in 0..tab_width {
            self.insert_char(' ');
        }
    }

    /// Toggle line comment for the current line based on file extension.
    pub fn toggle_comment(&mut self) {
        let ext = self
            .path
            .as_ref()
            .and_then(|p| p.extension())
            .and_then(|e| e.to_str())
            .unwrap_or("rs");

        let comment_prefix = match ext.to_lowercase().as_str() {
            "rs" | "c" | "cpp" | "h" | "hpp" | "java" | "go" | "js" | "ts" | "jsx" | "tsx" => "// ",
            "py" | "sh" | "bash" | "yaml" | "yml" | "toml" => "# ",
            "sql" => "-- ",
            _ => "// ",
        };

        let row = self.cursor.row;
        if row < self.lines.len() {
            self.push_undo();
            let line = &self.lines[row];
            let trimmed = line.trim_start();
            if trimmed.starts_with(comment_prefix.trim()) {
                // Uncomment: remove the comment prefix
                if let Some(pos) = line.find(comment_prefix) {
                    let mut chars: Vec<char> = line.chars().collect();
                    let char_pos = line[..pos].chars().count();
                    let prefix_len = comment_prefix.chars().count();
                    chars.drain(char_pos..char_pos + prefix_len);
                    self.lines[row] = chars.into_iter().collect();
                    self.cursor.col = self.cursor.col.saturating_sub(prefix_len);
                } else if let Some(pos) = line.find(comment_prefix.trim()) {
                    let mut chars: Vec<char> = line.chars().collect();
                    let char_pos = line[..pos].chars().count();
                    let prefix_len = comment_prefix.trim().chars().count();
                    chars.drain(char_pos..char_pos + prefix_len);
                    self.lines[row] = chars.into_iter().collect();
                    self.cursor.col = self.cursor.col.saturating_sub(prefix_len);
                }
            } else {
                // Comment: insert prefix after leading whitespace
                let indent_len = line.chars().take_while(|c| c.is_whitespace()).count();
                let mut chars: Vec<char> = line.chars().collect();
                for (idx, c) in comment_prefix.chars().enumerate() {
                    chars.insert(indent_len + idx, c);
                }
                self.lines[row] = chars.into_iter().collect();
                self.cursor.col += comment_prefix.chars().count();
            }
            self.dirty = true;
            self.sync_rope();
        }
    }

    // ── Clipboard ────────────────────────────────────────────────────────────

    pub fn copy_line(&mut self) {
        if let Some(sel_text) = self.selected_text() {
            self.clipboard = sel_text;
            return;
        }
        let row = self.cursor.row;
        self.clipboard = self.lines.get(row).cloned().unwrap_or_default();
    }

    pub fn cut_line(&mut self) {
        if self.has_selection() {
            if let Some(text) = self.delete_selection() {
                self.clipboard = text;
            }
            return;
        }
        self.push_undo();
        let row = self.cursor.row;
        if self.lines.len() > 1 {
            self.clipboard = self.lines.remove(row);
            if self.cursor.row >= self.lines.len() {
                self.cursor.row = self.lines.len() - 1;
            }
        } else {
            self.clipboard = std::mem::take(&mut self.lines[0]);
            self.cursor.col = 0;
        }
        self.dirty = true;
        self.sync_rope();
    }

    pub fn paste(&mut self) {
        let clip = self.clipboard.clone();
        if !clip.is_empty() {
            self.insert_text(&clip);
        }
    }

    // ── Search & Replace ─────────────────────────────────────────────────────

    pub fn search_next(&mut self) {
        if self.search_query.is_empty() {
            return;
        }
        let query = self.search_query.to_lowercase();
        let total = self.lines.len();
        for i in 0..total {
            let row = (self.cursor.row + 1 + i) % total;
            if let Some(byte_col) = self.lines[row].to_lowercase().find(&query) {
                let char_col = self.lines[row][..byte_col].chars().count();
                self.cursor = Cursor { row, col: char_col };
                self.search_match = row;
                return;
            }
        }
    }

    pub fn search_prev(&mut self) {
        if self.search_query.is_empty() {
            return;
        }
        let query = self.search_query.to_lowercase();
        let total = self.lines.len();
        for i in 1..=total {
            let row = (self.cursor.row + total - i) % total;
            if let Some(byte_col) = self.lines[row].to_lowercase().find(&query) {
                let char_col = self.lines[row][..byte_col].chars().count();
                self.cursor = Cursor { row, col: char_col };
                self.search_match = row;
                return;
            }
        }
    }

    /// Replace current match at cursor with replacement string.
    pub fn replace_current(&mut self, replacement: &str) -> bool {
        if self.search_query.is_empty() {
            return false;
        }
        let row = self.cursor.row;
        let line = &self.lines[row];
        let query_chars = self.search_query.chars().count();

        let chars: Vec<char> = line.chars().collect();
        let col = self.cursor.col;
        if col + query_chars <= chars.len() {
            let sub: String = chars[col..col + query_chars].iter().collect();
            if sub.eq_ignore_ascii_case(&self.search_query) {
                self.push_undo();
                let prefix: String = chars[..col].iter().collect();
                let suffix: String = chars[col + query_chars..].iter().collect();
                self.lines[row] = format!("{prefix}{replacement}{suffix}");
                self.cursor.col = col + replacement.chars().count();
                self.dirty = true;
                self.sync_rope();
                return true;
            }
        }
        false
    }

    /// Replace all occurrences of query with replacement throughout the buffer.
    pub fn replace_all(&mut self, query: &str, replacement: &str) -> usize {
        if query.is_empty() {
            return 0;
        }
        self.push_undo();
        let mut count = 0;
        for line in &mut self.lines {
            if line.contains(query) {
                let occurrences = line.matches(query).count();
                *line = line.replace(query, replacement);
                count += occurrences;
            }
        }
        if count > 0 {
            self.dirty = true;
            self.clamp_cursor();
            self.sync_rope();
        }
        count
    }

    // ── Save & Conflict Protection ───────────────────────────────────────────

    pub fn check_external_modification(&self) -> bool {
        if let Some(ref path) = self.path {
            if let Ok(meta) = std::fs::metadata(path) {
                if let Ok(current_mtime) = meta.modified() {
                    if let Some(loaded) = self.loaded_mtime {
                        return current_mtime > loaded;
                    }
                }
            }
        }
        false
    }

    pub fn save(&mut self) -> Result<()> {
        if self.check_external_modification() {
            bail!("Conflict: File on disk was modified by an external process since it was opened. Use save_force() to overwrite.");
        }
        self.save_force()
    }

    pub fn save_force(&mut self) -> Result<()> {
        let path = self
            .path
            .clone()
            .ok_or_else(|| anyhow::anyhow!("No file path set"))?;
        let sep = self.line_ending.as_str();
        let content = self.lines.join(sep);
        save_file_atomic(&path, content.as_bytes())?;
        self.loaded_mtime = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
        self.dirty = false;
        self.clear_recovery_snapshot();
        Ok(())
    }

    pub fn save_as(&mut self, path: PathBuf) -> Result<()> {
        self.path = Some(path);
        self.save_force()
    }

    // ── Buffer Recovery Snapshots ─────────────────────────────────────────────

    pub fn recovery_path_for(path: &Path) -> PathBuf {
        let dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("~/.local/share"))
            .join("termexplorer/recovery");
        let mut hasher = Sha256::new();
        hasher.update(path.to_string_lossy().as_bytes());
        let hash: String = hasher
            .finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();
        dir.join(format!("{}.rec", hash))
    }

    pub fn create_recovery_snapshot(&self) -> Result<()> {
        if self.dirty {
            if let Some(ref path) = self.path {
                let rec_path = Self::recovery_path_for(path);
                if let Some(parent) = rec_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let sep = self.line_ending.as_str();
                let content = self.lines.join(sep);
                std::fs::write(&rec_path, content)?;
            }
        }
        Ok(())
    }

    pub fn clear_recovery_snapshot(&self) {
        if let Some(ref path) = self.path {
            let rec_path = Self::recovery_path_for(path);
            let _ = std::fs::remove_file(rec_path);
        }
    }

    pub fn has_recovery_snapshot(path: &Path) -> bool {
        Self::recovery_path_for(path).exists()
    }

    pub fn recover_from_snapshot(path: &Path) -> Result<Self> {
        let rec_path = Self::recovery_path_for(path);
        let content = std::fs::read_to_string(&rec_path)?;
        let line_ending = if content.contains("\r\n") {
            LineEnding::Crlf
        } else {
            LineEnding::Lf
        };
        let rope = Rope::from_str(&content);
        let lines: Vec<String> = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(str::to_string).collect()
        };
        let loaded_mtime = std::fs::metadata(path).and_then(|m| m.modified()).ok();

        Ok(Self {
            path: Some(path.to_path_buf()),
            lines,
            cursor: Cursor::default(),
            selection: None,
            dirty: true,
            scroll_row: 0,
            scroll_col: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            clipboard: String::new(),
            search_query: String::new(),
            search_match: 0,
            completion: CompletionState::default(),
            loaded_mtime,
            line_ending,
            rope,
        })
    }

    // ── Autocompletion ───────────────────────────────────────────────────────

    pub fn update_completion(&mut self) {
        let line = self.current_line();
        let prefix = prefix_at(line, self.cursor.col);
        if prefix.len() >= 2 {
            let completions = layered_completions(&self.lines, prefix, self.path.as_deref(), 8);
            if !completions.is_empty() {
                self.completion = CompletionState {
                    active: true,
                    candidates: completions,
                    selected: 0,
                    prefix: prefix.to_string(),
                };
                return;
            }
        }
        self.completion.active = false;
        self.completion.candidates.clear();
    }

    /// Accept selected completion item, supporting rich multi-line snippet expansions.
    pub fn accept_completion(&mut self) -> bool {
        if !self.completion.active || self.completion.candidates.is_empty() {
            return false;
        }
        let candidate = self.completion.candidates[self.completion.selected].clone();
        let prefix_len = self.completion.prefix.chars().count();

        self.push_undo();
        let row = self.cursor.row;
        let col = self.cursor.col;

        if col >= prefix_len {
            let line = &mut self.lines[row];
            let mut chars: Vec<char> = line.chars().collect();
            let start = col - prefix_len;
            let end = col.min(chars.len());
            chars.drain(start..end);
            *line = chars.into_iter().collect();
            self.cursor.col = start;

            // Insert candidate label
            let label = candidate.label.clone();
            self.insert_text(&label);
            self.dirty = true;
            self.sync_rope();
        }

        self.completion.active = false;
        self.completion.candidates.clear();
        true
    }

    pub fn completion_next(&mut self) {
        if self.completion.active && !self.completion.candidates.is_empty() {
            self.completion.selected =
                (self.completion.selected + 1) % self.completion.candidates.len();
        }
    }

    pub fn completion_prev(&mut self) {
        if self.completion.active && !self.completion.candidates.is_empty() {
            if self.completion.selected == 0 {
                self.completion.selected = self.completion.candidates.len() - 1;
            } else {
                self.completion.selected -= 1;
            }
        }
    }

    pub fn dismiss_completion(&mut self) {
        self.completion.active = false;
        self.completion.candidates.clear();
    }

    // ── Viewport Scroll Management ───────────────────────────────────────────

    pub fn scroll_to_cursor(&mut self, visible_rows: usize, visible_cols: usize) {
        if visible_rows == 0 || visible_cols == 0 {
            return;
        }
        // Vertical scroll
        if self.cursor.row < self.scroll_row {
            self.scroll_row = self.cursor.row;
        } else if self.cursor.row >= self.scroll_row + visible_rows {
            self.scroll_row = self.cursor.row + 1 - visible_rows;
        }
        // Horizontal scroll
        if self.cursor.col < self.scroll_col {
            self.scroll_col = self.cursor.col;
        } else if self.cursor.col >= self.scroll_col + visible_cols {
            self.scroll_col = self.cursor.col + 1 - visible_cols;
        }
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn visible_lines(&self, start: usize, count: usize) -> impl Iterator<Item = (usize, &str)> {
        self.lines
            .iter()
            .enumerate()
            .skip(start)
            .take(count)
            .map(|(i, l)| (i, l.as_str()))
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}
