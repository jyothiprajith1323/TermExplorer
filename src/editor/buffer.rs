//! Text editor buffer — stores content, handles cursor movement,
//! undo/redo, selection, search, and atomic saving.

use crate::filesystem::operations::save_file_atomic;
use anyhow::{bail, Result};
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

// ── Cursor ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
}

use crate::editor::completion::{layered_completions, prefix_at, Completion};

// ── Completion state ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct CompletionState {
    pub active: bool,
    pub candidates: Vec<Completion>,
    pub selected: usize,
    pub prefix: String,
}

// ── Editor buffer ───────────────────────────────────────────────────────────

#[allow(dead_code)]
pub struct Buffer {
    pub path: Option<PathBuf>,
    pub lines: Vec<String>,
    pub cursor: Cursor,
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
}

impl Buffer {
    /// Create a new, empty buffer.
    pub fn new() -> Self {
        Self {
            path: None,
            lines: vec![String::new()],
            cursor: Cursor::default(),
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
        })
    }

    // ── Undo/redo helpers ────────────────────────────────────────────────────

    fn push_undo(&mut self) {
        self.undo_stack.push((self.lines.clone(), self.cursor));
        // Limit stack depth to avoid unbounded memory use.
        if self.undo_stack.len() > 200 {
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
        }
    }

    pub fn redo(&mut self) {
        if let Some((lines, cursor)) = self.redo_stack.pop() {
            self.undo_stack.push((self.lines.clone(), self.cursor));
            self.lines = lines;
            self.cursor = cursor;
            self.dirty = true;
            self.clamp_cursor();
        }
    }

    // ── Cursor movement ──────────────────────────────────────────────────────

    fn clamp_cursor(&mut self) {
        self.cursor.row = self.cursor.row.min(self.lines.len().saturating_sub(1));
        let line_len = self.current_line().len();
        self.cursor.col = self.cursor.col.min(line_len);
    }

    fn current_line(&self) -> &str {
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
            self.cursor.col = self.lines[self.cursor.row].len();
        }
    }

    pub fn move_right(&mut self) {
        let len = self.current_line().len();
        if self.cursor.col < len {
            self.cursor.col += 1;
        } else if self.cursor.row + 1 < self.lines.len() {
            self.cursor.row += 1;
            self.cursor.col = 0;
        }
    }

    pub fn move_home(&mut self) {
        // Smart home: go to first non-whitespace, then to column 0.
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
        self.cursor.col = self.current_line().len();
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

    // ── Editing ──────────────────────────────────────────────────────────────

    /// Insert a character at the current cursor position.
    pub fn insert_char(&mut self, c: char) {
        self.push_undo();
        let row = self.cursor.row;
        let col = self.cursor.col;
        // Ensure the row exists.
        while self.lines.len() <= row {
            self.lines.push(String::new());
        }
        self.lines[row].insert(col, c);
        self.cursor.col += 1;
        self.dirty = true;
    }

    /// Insert a newline and auto-indent.
    pub fn insert_newline(&mut self) {
        self.push_undo();
        let row = self.cursor.row;
        let col = self.cursor.col;
        let line = self.lines[row].clone();
        let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
        let rest = line[col..].to_string();
        self.lines[row] = line[..col].to_string();
        self.lines.insert(row + 1, format!("{indent}{rest}"));
        self.cursor.row += 1;
        self.cursor.col = indent.len();
        self.dirty = true;
    }

    /// Backspace at the current position.
    pub fn backspace(&mut self) {
        self.push_undo();
        let row = self.cursor.row;
        let col = self.cursor.col;

        if col > 0 {
            self.lines[row].remove(col - 1);
            self.cursor.col -= 1;
        } else if row > 0 {
            let cur = self.lines.remove(row);
            let prev_len = self.lines[row - 1].len();
            self.lines[row - 1].push_str(&cur);
            self.cursor.row -= 1;
            self.cursor.col = prev_len;
        }
        self.dirty = true;
    }

    /// Delete the character under the cursor.
    pub fn delete_char(&mut self) {
        self.push_undo();
        let row = self.cursor.row;
        let col = self.cursor.col;

        if col < self.lines[row].len() {
            self.lines[row].remove(col);
        } else if row + 1 < self.lines.len() {
            let next = self.lines.remove(row + 1);
            self.lines[row].push_str(&next);
        }
        self.dirty = true;
    }

    /// Insert a tab (using spaces according to `tab_width`).
    pub fn insert_tab(&mut self, tab_width: usize) {
        for _ in 0..tab_width {
            self.insert_char(' ');
        }
    }

    // ── Clipboard ────────────────────────────────────────────────────────────

    pub fn copy_line(&mut self) {
        let row = self.cursor.row;
        self.clipboard = self.lines.get(row).cloned().unwrap_or_default();
    }

    pub fn cut_line(&mut self) {
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
    }

    pub fn paste(&mut self) {
        self.push_undo();
        let clip = self.clipboard.clone();
        // Insert all characters of the clipboard at the cursor.
        for c in clip.chars() {
            if c == '\n' {
                self.insert_newline();
            } else {
                self.insert_char(c);
            }
        }
    }

    // ── Search ───────────────────────────────────────────────────────────────

    #[allow(dead_code)]
    pub fn search_next(&mut self) {
        if self.search_query.is_empty() {
            return;
        }
        let query = self.search_query.to_lowercase();
        let total = self.lines.len();
        for i in 0..total {
            let row = (self.cursor.row + 1 + i) % total;
            if let Some(col) = self.lines[row].to_lowercase().find(&query) {
                self.cursor = Cursor { row, col };
                self.search_match = row;
                return;
            }
        }
    }

    #[allow(dead_code)]
    pub fn search_prev(&mut self) {
        if self.search_query.is_empty() {
            return;
        }
        let query = self.search_query.to_lowercase();
        let total = self.lines.len();
        for i in 1..=total {
            let row = (self.cursor.row + total - i) % total;
            if let Some(col) = self.lines[row].to_lowercase().find(&query) {
                self.cursor = Cursor { row, col };
                self.search_match = row;
                return;
            }
        }
    }

    // ── Save & External Modification Protection ─────────────────────────────

    /// Check if the file on disk has been modified externally since it was opened or saved.
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

    /// Save buffer atomically to disk with external modification protection.
    pub fn save(&mut self) -> Result<()> {
        if self.check_external_modification() {
            bail!("Conflict: File on disk was modified by an external process since it was opened. Use save_force() to overwrite.");
        }
        self.save_force()
    }

    /// Save buffer atomically to disk, intentionally overriding any external modification.
    pub fn save_force(&mut self) -> Result<()> {
        let path = self
            .path
            .clone()
            .ok_or_else(|| anyhow::anyhow!("No file path set"))?;
        let sep = match self.line_ending {
            LineEnding::Crlf => "\r\n",
            LineEnding::Lf => "\n",
        };
        let content = self.lines.join(sep);
        save_file_atomic(&path, content.as_bytes())?;
        self.loaded_mtime = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
        self.dirty = false;
        self.clear_recovery_snapshot();
        Ok(())
    }

    #[allow(dead_code)]
    pub fn save_as(&mut self, path: PathBuf) -> Result<()> {
        self.path = Some(path);
        self.save_force()
    }

    // ── Buffer Recovery Snapshots ─────────────────────────────────────────────

    /// Deterministic recovery path in ~/.local/share/termexplorer/recovery/
    pub fn recovery_path_for(path: &Path) -> PathBuf {
        let dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("~/.local/share"))
            .join("termexplorer/recovery");
        let mut hasher = Sha256::new();
        hasher.update(path.to_string_lossy().as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        dir.join(format!("{}.rec", hash))
    }

    /// Save a recovery snapshot if the buffer has unsaved changes.
    pub fn create_recovery_snapshot(&self) -> Result<()> {
        if self.dirty {
            if let Some(ref path) = self.path {
                let rec_path = Self::recovery_path_for(path);
                if let Some(parent) = rec_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let sep = match self.line_ending {
                    LineEnding::Crlf => "\r\n",
                    LineEnding::Lf => "\n",
                };
                let content = self.lines.join(sep);
                std::fs::write(&rec_path, content)?;
            }
        }
        Ok(())
    }

    /// Clear the recovery snapshot after clean save or discard.
    pub fn clear_recovery_snapshot(&self) {
        if let Some(ref path) = self.path {
            let rec_path = Self::recovery_path_for(path);
            let _ = std::fs::remove_file(rec_path);
        }
    }

    /// Check if an unsaved crash recovery snapshot exists for the given file.
    pub fn has_recovery_snapshot(path: &Path) -> bool {
        Self::recovery_path_for(path).exists()
    }

    /// Restore buffer content from an unsaved crash recovery snapshot.
    pub fn recover_from_snapshot(path: &Path) -> Result<Self> {
        let rec_path = Self::recovery_path_for(path);
        let content = std::fs::read_to_string(&rec_path)?;
        let line_ending = if content.contains("\r\n") {
            LineEnding::Crlf
        } else {
            LineEnding::Lf
        };
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

    pub fn accept_completion(&mut self) -> bool {
        if !self.completion.active || self.completion.candidates.is_empty() {
            return false;
        }
        let selected_label = self.completion.candidates[self.completion.selected]
            .label
            .clone();
        let prefix_len = self.completion.prefix.len();
        self.push_undo();
        let row = self.cursor.row;
        let col = self.cursor.col;
        if col >= prefix_len {
            let line = &mut self.lines[row];
            let start = col - prefix_len;
            line.drain(start..col);
            line.insert_str(start, &selected_label);
            self.cursor.col = start + selected_label.len();
            self.dirty = true;
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

    // ── Scroll management ────────────────────────────────────────────────────

    /// Adjust scroll offsets so the cursor is visible in a viewport of size
    /// (visible_rows × visible_cols).
    pub fn scroll_to_cursor(&mut self, visible_rows: usize, visible_cols: usize) {
        // Vertical
        if self.cursor.row < self.scroll_row {
            self.scroll_row = self.cursor.row;
        } else if self.cursor.row >= self.scroll_row + visible_rows {
            self.scroll_row = self.cursor.row + 1 - visible_rows;
        }
        // Horizontal
        if self.cursor.col < self.scroll_col {
            self.scroll_col = self.cursor.col;
        } else if self.cursor.col >= self.scroll_col + visible_cols {
            self.scroll_col = self.cursor.col + 1 - visible_cols;
        }
    }

    // ── Line count ───────────────────────────────────────────────────────────

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    // ── Visible lines (for rendering) ────────────────────────────────────────

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
