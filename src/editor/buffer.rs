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
use unicode_segmentation::UnicodeSegmentation;

// ── File Encoding ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileEncoding {
    #[default]
    Utf8,
    Utf8Bom,
    Latin1,
}

impl FileEncoding {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Utf8 => "UTF-8",
            Self::Utf8Bom => "UTF-8 BOM",
            Self::Latin1 => "Latin-1",
        }
    }
}

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
    /// Detected or target file encoding.
    pub encoding: FileEncoding,
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
            encoding: FileEncoding::Utf8,
            rope: Rope::from_str(""),
        }
    }

    /// Load a file into the buffer, recording mtime, detecting line endings, and handling encodings/BOM.
    pub fn from_file(path: PathBuf) -> Result<Self> {
        let bytes = std::fs::read(&path)?;
        let mut encoding = FileEncoding::Utf8;

        let content = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
            encoding = FileEncoding::Utf8Bom;
            String::from_utf8_lossy(&bytes[3..]).into_owned()
        } else {
            match std::str::from_utf8(&bytes) {
                Ok(s) => s.to_string(),
                Err(_) => {
                    // Fall back to ISO-8859-1 (Latin-1) which maps byte 0x00..=0xFF directly to Unicode chars
                    encoding = FileEncoding::Latin1;
                    bytes.iter().map(|&b| b as char).collect::<String>()
                }
            }
        };

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
            encoding,
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
        self.lines
            .iter()
            .map(|l| l.graphemes(true).count() + 1)
            .sum()
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
        let count = self.current_line().graphemes(true).count();
        self.cursor.col = self.cursor.col.min(count);
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
            self.cursor.col = self.lines[self.cursor.row].graphemes(true).count();
        }
    }

    pub fn move_right(&mut self) {
        let count = self.current_line().graphemes(true).count();
        if self.cursor.col < count {
            self.cursor.col += 1;
        } else if self.cursor.row + 1 < self.lines.len() {
            self.cursor.row += 1;
            self.cursor.col = 0;
        }
    }

    pub fn move_home(&mut self) {
        // Smart home: first non-whitespace grapheme, then column 0.
        let first_nonws = self
            .current_line()
            .graphemes(true)
            .take_while(|g| g.chars().all(|c| c.is_whitespace()))
            .count();
        if self.cursor.col != first_nonws {
            self.cursor.col = first_nonws;
        } else {
            self.cursor.col = 0;
        }
    }

    pub fn move_end(&mut self) {
        self.cursor.col = self.current_line().graphemes(true).count();
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
            let graphemes: Vec<&str> = line.graphemes(true).collect();
            let start_col = start.col.min(graphemes.len());
            let end_col = end.col.min(graphemes.len());
            Some(graphemes[start_col..end_col].concat())
        } else {
            let mut result = Vec::new();
            for r in start.row..=end.row {
                if let Some(line) = self.lines.get(r) {
                    let graphemes: Vec<&str> = line.graphemes(true).collect();
                    if r == start.row {
                        let sc = start.col.min(graphemes.len());
                        result.push(graphemes[sc..].concat());
                    } else if r == end.row {
                        let ec = end.col.min(graphemes.len());
                        result.push(graphemes[..ec].concat());
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
            let mut graphemes: Vec<&str> = line.graphemes(true).collect();
            let sc = start.col.min(graphemes.len());
            let ec = end.col.min(graphemes.len());
            graphemes.drain(sc..ec);
            *line = graphemes.concat();
        } else {
            let mut new_lines = Vec::new();
            for (idx, line) in self.lines.iter().enumerate() {
                if idx < start.row || idx > end.row {
                    new_lines.push(line.clone());
                } else if idx == start.row {
                    let first_graphemes: Vec<&str> = line.graphemes(true).collect();
                    let sc = start.col.min(first_graphemes.len());
                    let prefix: String = first_graphemes[..sc].concat();

                    let last_line = &self.lines[end.row];
                    let last_graphemes: Vec<&str> = last_line.graphemes(true).collect();
                    let ec = end.col.min(last_graphemes.len());
                    let suffix: String = last_graphemes[ec..].concat();

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
        let s = c.to_string();
        self.insert_grapheme_no_undo(&s);
    }

    /// Insert a grapheme cluster at the current cursor position safely.
    pub fn insert_grapheme(&mut self, g: &str) {
        self.push_undo();
        self.insert_grapheme_no_undo(g);
        self.dirty = true;
        self.sync_rope();
    }

    fn insert_grapheme_no_undo(&mut self, g: &str) {
        let row = self.cursor.row;
        let col = self.cursor.col;

        while self.lines.len() <= row {
            self.lines.push(String::new());
        }

        let mut graphemes: Vec<String> = self.lines[row]
            .graphemes(true)
            .map(str::to_string)
            .collect();
        let safe_col = col.min(graphemes.len());
        graphemes.insert(safe_col, g.to_string());
        self.lines[row] = graphemes.concat();
        self.cursor.col = safe_col + 1;
    }

    /// Insert arbitrary multi-line text at cursor (for snippets, completions, paste).
    pub fn insert_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.push_undo();
        for g in text.graphemes(true) {
            if g == "\n" {
                self.insert_newline_no_undo();
            } else if g != "\r" {
                self.insert_grapheme_no_undo(g);
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
        let graphemes: Vec<&str> = line.graphemes(true).collect();
        let safe_col = col.min(graphemes.len());

        let indent: String = graphemes
            .iter()
            .take_while(|g| g.chars().all(|c| c.is_whitespace()))
            .copied()
            .collect();
        let prefix: String = graphemes[..safe_col].concat();
        let suffix: String = graphemes[safe_col..].concat();

        self.lines[row] = prefix;
        self.lines.insert(row + 1, format!("{indent}{suffix}"));
        self.cursor.row += 1;
        self.cursor.col = indent.graphemes(true).count();
    }

    /// Backspace at the current cursor position (grapheme-safe).
    pub fn backspace(&mut self) {
        self.push_undo();
        let row = self.cursor.row;
        let col = self.cursor.col;

        if col > 0 {
            let mut graphemes: Vec<String> = self.lines[row]
                .graphemes(true)
                .map(str::to_string)
                .collect();
            let safe_col = col.min(graphemes.len());
            if safe_col > 0 {
                graphemes.remove(safe_col - 1);
                self.lines[row] = graphemes.concat();
                self.cursor.col = safe_col - 1;
            }
        } else if row > 0 {
            let cur = self.lines.remove(row);
            let prev_len = self.lines[row - 1].graphemes(true).count();
            self.lines[row - 1].push_str(&cur);
            self.cursor.row -= 1;
            self.cursor.col = prev_len;
        }
        self.dirty = true;
        self.sync_rope();
    }

    /// Delete character/grapheme under cursor (grapheme-safe).
    pub fn delete_char(&mut self) {
        self.push_undo();
        let row = self.cursor.row;
        let col = self.cursor.col;

        if row < self.lines.len() {
            let mut graphemes: Vec<String> = self.lines[row]
                .graphemes(true)
                .map(str::to_string)
                .collect();
            if col < graphemes.len() {
                graphemes.remove(col);
                self.lines[row] = graphemes.concat();
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
                    let mut graphemes: Vec<String> =
                        line.graphemes(true).map(str::to_string).collect();
                    let g_pos = line[..pos].graphemes(true).count();
                    let prefix_len = comment_prefix.graphemes(true).count();
                    graphemes.drain(g_pos..g_pos + prefix_len);
                    self.lines[row] = graphemes.concat();
                    self.cursor.col = self.cursor.col.saturating_sub(prefix_len);
                } else if let Some(pos) = line.find(comment_prefix.trim()) {
                    let mut graphemes: Vec<String> =
                        line.graphemes(true).map(str::to_string).collect();
                    let g_pos = line[..pos].graphemes(true).count();
                    let prefix_len = comment_prefix.trim().graphemes(true).count();
                    graphemes.drain(g_pos..g_pos + prefix_len);
                    self.lines[row] = graphemes.concat();
                    self.cursor.col = self.cursor.col.saturating_sub(prefix_len);
                }
            } else {
                // Comment: insert prefix after leading whitespace
                let indent_len = line
                    .graphemes(true)
                    .take_while(|g| g.chars().all(|c| c.is_whitespace()))
                    .count();
                let mut graphemes: Vec<String> = line.graphemes(true).map(str::to_string).collect();
                for (idx, g) in comment_prefix.graphemes(true).enumerate() {
                    graphemes.insert(indent_len + idx, g.to_string());
                }
                self.lines[row] = graphemes.concat();
                self.cursor.col += comment_prefix.graphemes(true).count();
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
        let mut bytes = Vec::new();
        if self.encoding == FileEncoding::Utf8Bom {
            bytes.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
        }
        if self.encoding == FileEncoding::Latin1 {
            let all_latin1 = content.chars().all(|c| (c as u32) <= 255);
            if all_latin1 {
                bytes.extend(content.chars().map(|c| c as u8));
            } else {
                bytes.extend_from_slice(content.as_bytes());
            }
        } else {
            bytes.extend_from_slice(content.as_bytes());
        }

        save_file_atomic(&path, &bytes)?;
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
            encoding: FileEncoding::Utf8,
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
        let prefix_len = self.completion.prefix.graphemes(true).count();

        self.push_undo();
        let row = self.cursor.row;
        let col = self.cursor.col;

        if col >= prefix_len {
            let line = &mut self.lines[row];
            let mut graphemes: Vec<String> = line.graphemes(true).map(str::to_string).collect();
            let start = col - prefix_len;
            let end = col.min(graphemes.len());
            graphemes.drain(start..end);
            *line = graphemes.concat();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unicode_grapheme_navigation_and_editing() {
        let mut buf = Buffer::new();
        // Insert complex multi-codepoint emojis:
        // "👍🏽" is thumbs up (U+1F44D) + medium skin tone (U+1F3FD)
        // "🇮🇳" is regional indicators (U+1F1EE, U+1F1F3)
        // "👨‍💻" is man (U+1F468) + ZWJ (U+200D) + laptop (U+1F4BB)
        buf.insert_text("Hi 👍🏽 🇮🇳 👨‍💻 End");
        assert_eq!(buf.lines[0], "Hi 👍🏽 🇮🇳 👨‍💻 End");

        // "Hi " = 3 graphemes
        // "👍🏽" = 1 grapheme
        // " " = 1 grapheme
        // "🇮🇳" = 1 grapheme
        // " " = 1 grapheme
        // "👨‍💻" = 1 grapheme
        // " End" = 4 graphemes
        // Total = 12 graphemes
        assert_eq!(buf.cursor.col, 12);

        // Move left 4 times to land right after 👨‍💻
        for _ in 0..4 {
            buf.move_left();
        }
        assert_eq!(buf.cursor.col, 8);

        // Move left across 👨‍💻 (1 grapheme move)
        buf.move_left();
        assert_eq!(buf.cursor.col, 7);

        // Move right across 👨‍💻
        buf.move_right();
        assert_eq!(buf.cursor.col, 8);

        // Backspace deletes entire 👨‍💻 cluster as a unit
        buf.backspace();
        assert_eq!(buf.cursor.col, 7);
        assert_eq!(buf.lines[0], "Hi 👍🏽 🇮🇳  End");

        // Move left past space
        buf.move_left();
        assert_eq!(buf.cursor.col, 6);

        // Backspace deletes entire 🇮🇳 flag cluster as a unit
        buf.backspace();
        assert_eq!(buf.cursor.col, 5);
        assert_eq!(buf.lines[0], "Hi 👍🏽   End");

        // Move left past space
        buf.move_left();
        assert_eq!(buf.cursor.col, 4);

        // Backspace deletes entire 👍🏽 skin-tone emoji as a unit
        buf.backspace();
        assert_eq!(buf.cursor.col, 3);
        assert_eq!(buf.lines[0], "Hi    End");
    }

    #[test]
    fn test_utf8_bom_preservation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("bom_test.txt");

        // Write file with UTF-8 BOM
        let mut initial_bytes = vec![0xEF, 0xBB, 0xBF];
        initial_bytes.extend_from_slice(b"Hello BOM\nWorld\n");
        std::fs::write(&file_path, &initial_bytes).unwrap();

        // Load into buffer
        let mut buf = Buffer::from_file(file_path.clone()).unwrap();
        assert_eq!(buf.encoding, FileEncoding::Utf8Bom);
        assert_eq!(buf.lines[0], "Hello BOM");
        assert_eq!(buf.lines[1], "World");

        // Edit and save
        buf.cursor.row = 0;
        buf.cursor.col = 9;
        buf.insert_text(" Preserved");
        buf.save().unwrap();

        // Verify written file still has UTF-8 BOM
        let saved_bytes = std::fs::read(&file_path).unwrap();
        assert!(saved_bytes.starts_with(&[0xEF, 0xBB, 0xBF]));
        let saved_str = std::str::from_utf8(&saved_bytes[3..]).unwrap();
        assert!(saved_str.starts_with("Hello BOM Preserved\nWorld"));
    }

    #[test]
    fn test_latin1_fallback_encoding() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("latin1_test.txt");

        // Write raw Latin-1 bytes with 0xE9 ('é' in ISO-8859-1) which is invalid UTF-8
        let latin1_bytes = vec![b'C', b'a', b'f', 0xE9];
        std::fs::write(&file_path, &latin1_bytes).unwrap();

        let buf = Buffer::from_file(file_path).unwrap();
        assert_eq!(buf.encoding, FileEncoding::Latin1);
        assert_eq!(buf.lines[0], "Café");
    }
}
