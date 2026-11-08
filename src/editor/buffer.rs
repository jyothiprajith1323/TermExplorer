//! Text editor buffer — stores content using an efficient rope structure,
//! handles cursor movement, transactional delta undo/redo, visual selection,
//! search/replace, Unicode grapheme safety, and atomic saving.

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
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

// ── Transactional Delta Undo/Redo ───────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditDelta {
    Insert { char_idx: usize, text: String },
    Delete { char_idx: usize, text: String },
    Batch(Vec<EditDelta>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndoTransaction {
    pub deltas: Vec<EditDelta>,
    pub cursor_before: Cursor,
    pub cursor_after: Cursor,
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

#[derive(Debug)]
#[allow(dead_code)]
pub struct Buffer {
    pub path: Option<PathBuf>,
    pub lines: Vec<String>,
    pub cursor: Cursor,
    pub secondary_cursors: Vec<Cursor>,
    pub selection: Option<Selection>,
    pub dirty: bool,
    pub scroll_row: usize,
    pub scroll_col: usize,
    undo_stack: Vec<UndoTransaction>,
    redo_stack: Vec<UndoTransaction>,
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
    /// Whether the file has a trailing newline at EOF.
    pub has_trailing_newline: bool,
    /// Internal rope structure for efficient large-document operations.
    pub rope: Rope,
    /// Ordered tab stops for active snippet expansion.
    pub snippet_tab_stops: Vec<Cursor>,
}

impl Buffer {
    /// Create a new, empty buffer.
    pub fn new() -> Self {
        let mut buf = Self {
            path: None,
            lines: vec![String::new()],
            cursor: Cursor::default(),
            secondary_cursors: Vec::new(),
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
            has_trailing_newline: false,
            rope: Rope::from_str(""),
            snippet_tab_stops: Vec::new(),
        };
        buf.sync_lines_from_rope();
        buf
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
        let has_trailing_newline = content.ends_with('\n');
        let rope = Rope::from_str(&content);
        let loaded_mtime = std::fs::metadata(&path).and_then(|m| m.modified()).ok();

        let mut buf = Self {
            path: Some(path),
            lines: Vec::new(),
            cursor: Cursor::default(),
            secondary_cursors: Vec::new(),
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
            has_trailing_newline,
            rope,
            snippet_tab_stops: Vec::new(),
        };
        buf.sync_lines_from_rope();
        Ok(buf)
    }

    /// Return the full serialized document text directly from lines and line endings.
    pub fn text(&self) -> String {
        if self.lines.is_empty() {
            return String::new();
        }
        let sep = self.line_ending.as_str();
        let mut s = self.lines.join(sep);
        if self.has_trailing_newline {
            s.push_str(sep);
        }
        s
    }

    /// Synchronize the internal rope with the current document text.
    pub fn sync_rope(&mut self) {
        self.rope = Rope::from_str(&self.text());
    }

    /// Ensure rope is synchronized if external code mutated `lines` directly.
    pub fn ensure_rope_synced(&mut self) {
        if self.rope.len_chars() == 0 && (self.lines.len() > 1 || !self.lines[0].is_empty()) {
            self.sync_rope();
        }
    }

    /// Synchronize lines from the authoritative internal rope.
    pub fn sync_lines_from_rope(&mut self) {
        let mut lines: Vec<String> = self
            .rope
            .lines()
            .map(|slice| {
                let mut s = slice.to_string();
                if s.ends_with("\r\n") {
                    s.pop();
                    s.pop();
                } else if s.ends_with('\n') {
                    s.pop();
                }
                s
            })
            .collect();
        if self.has_trailing_newline
            && lines.len() > 1
            && lines.last().map(|l| l.is_empty()).unwrap_or(false)
        {
            lines.pop();
        }
        if lines.is_empty() {
            lines.push(String::new());
        }
        self.lines = lines;
    }

    /// Invariant check for tests: verifies that rope, line cache, and serialized
    /// representations are in exact lockstep, and cursors are within valid bounds.
    pub fn assert_invariants(&self) {
        let text = self.text();
        let rope_str = self.rope.to_string();
        assert_eq!(
            text, rope_str,
            "Invariant violation: text() != rope.to_string()"
        );
        assert!(
            self.cursor.row < self.lines.len(),
            "Cursor row {} >= lines.len() {}",
            self.cursor.row,
            self.lines.len()
        );
        let cur_line = &self.lines[self.cursor.row];
        let grapheme_count = cur_line.graphemes(true).count();
        assert!(
            self.cursor.col <= grapheme_count,
            "Cursor col {} > graphemes {} on line {}",
            self.cursor.col,
            grapheme_count,
            self.cursor.row
        );
        for (idx, sc) in self.secondary_cursors.iter().enumerate() {
            assert!(
                sc.row < self.lines.len(),
                "Secondary cursor {idx} row {} >= lines.len() {}",
                sc.row,
                self.lines.len()
            );
            let sc_line = &self.lines[sc.row];
            let sc_graphemes = sc_line.graphemes(true).count();
            assert!(
                sc.col <= sc_graphemes,
                "Secondary cursor {idx} col {} > graphemes {}",
                sc.col,
                sc_graphemes
            );
        }
    }

    /// Return total character length of the entire buffer.
    pub fn total_char_len(&self) -> usize {
        self.rope.len_chars()
    }

    /// Convert row and grapheme column into absolute char index in rope.
    pub fn char_offset_for_cursor(&self, row: usize, col: usize) -> usize {
        if row >= self.lines.len() {
            return self.rope.len_chars();
        }
        let line_start = if row < self.rope.len_lines() {
            self.rope.line_to_char(row)
        } else {
            self.rope.len_chars()
        };
        let char_in_line: usize = self.lines[row]
            .graphemes(true)
            .take(col)
            .map(|g| g.chars().count())
            .sum();
        (line_start + char_in_line).min(self.rope.len_chars())
    }

    /// Convert char index in rope into Cursor (row and grapheme col).
    pub fn cursor_for_char_offset(&self, char_idx: usize) -> Cursor {
        let clamped_idx = char_idx.min(self.rope.len_chars());
        let row = self.rope.char_to_line(clamped_idx);
        let line_start = self.rope.line_to_char(row);
        let char_in_line = clamped_idx.saturating_sub(line_start);

        let col = if row < self.lines.len() {
            let mut count = 0;
            let mut accumulated_chars = 0;
            for g in self.lines[row].graphemes(true) {
                let g_chars = g.chars().count();
                if accumulated_chars + g_chars > char_in_line {
                    break;
                }
                accumulated_chars += g_chars;
                count += 1;
            }
            count
        } else {
            0
        };

        Cursor { row, col }
    }

    // ── Transactional Undo/Redo ──────────────────────────────────────────────

    pub fn push_transaction(&mut self, tx: UndoTransaction) {
        self.undo_stack.push(tx);
        if self.undo_stack.len() > 1000 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn mark_dirty_and_invalidate_syntax(&mut self, from_row: usize) {
        self.dirty = true;
        crate::editor::syntax::invalidate_syntax_from_line(self.path.as_deref(), from_row);
    }

    pub fn push_undo(&mut self) {
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(tx) = self.undo_stack.pop() {
            self.apply_inverse_transaction(&tx);
            self.cursor = tx.cursor_before;
            self.clamp_cursor();
            self.mark_dirty_and_invalidate_syntax(tx.cursor_before.row.min(self.cursor.row));
            self.redo_stack.push(tx);
        }
    }

    pub fn redo(&mut self) {
        if let Some(tx) = self.redo_stack.pop() {
            self.apply_forward_transaction(&tx);
            self.cursor = tx.cursor_after;
            self.clamp_cursor();
            self.mark_dirty_and_invalidate_syntax(tx.cursor_after.row.min(self.cursor.row));
            self.undo_stack.push(tx);
        }
    }

    fn apply_inverse_transaction(&mut self, tx: &UndoTransaction) {
        for delta in tx.deltas.iter().rev() {
            self.invert_delta(delta);
        }
        self.sync_lines_from_rope();
    }

    fn invert_delta(&mut self, delta: &EditDelta) {
        match delta {
            EditDelta::Insert { char_idx, text } => {
                let len = text.chars().count();
                if *char_idx + len <= self.rope.len_chars() {
                    self.rope.remove(*char_idx..*char_idx + len);
                }
            }
            EditDelta::Delete { char_idx, text } => {
                let idx = (*char_idx).min(self.rope.len_chars());
                self.rope.insert(idx, text);
            }
            EditDelta::Batch(sub_deltas) => {
                for sub in sub_deltas.iter().rev() {
                    self.invert_delta(sub);
                }
            }
        }
    }

    fn apply_forward_transaction(&mut self, tx: &UndoTransaction) {
        for delta in &tx.deltas {
            self.apply_delta(delta);
        }
        self.sync_lines_from_rope();
    }

    fn apply_delta(&mut self, delta: &EditDelta) {
        match delta {
            EditDelta::Insert { char_idx, text } => {
                let idx = (*char_idx).min(self.rope.len_chars());
                self.rope.insert(idx, text);
            }
            EditDelta::Delete { char_idx, text } => {
                let len = text.chars().count();
                if *char_idx + len <= self.rope.len_chars() {
                    self.rope.remove(*char_idx..*char_idx + len);
                }
            }
            EditDelta::Batch(sub_deltas) => {
                for sub in sub_deltas {
                    self.apply_delta(sub);
                }
            }
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    // ── Cursor & Selection Management ────────────────────────────────────────

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

    pub fn goto_line(&mut self, line: usize) {
        if line > 0 && line <= self.lines.len() {
            self.cursor.row = line - 1;
            self.cursor.col = 0;
            self.clamp_cursor();
        }
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

    pub fn move_word_left(&mut self) {
        if self.cursor.col == 0 {
            if self.cursor.row > 0 {
                self.cursor.row -= 1;
                self.cursor.col = self.lines[self.cursor.row].graphemes(true).count();
            }
            return;
        }
        let graphemes: Vec<&str> = self.current_line().graphemes(true).collect();
        let mut col = self.cursor.col.min(graphemes.len());
        while col > 0 && graphemes[col - 1].chars().all(|c| c.is_whitespace()) {
            col -= 1;
        }
        while col > 0
            && graphemes[col - 1]
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_')
        {
            col -= 1;
        }
        self.cursor.col = col;
    }

    pub fn move_word_right(&mut self) {
        let graphemes: Vec<&str> = self.current_line().graphemes(true).collect();
        let count = graphemes.len();
        if self.cursor.col >= count {
            if self.cursor.row + 1 < self.lines.len() {
                self.cursor.row += 1;
                self.cursor.col = 0;
            }
            return;
        }
        let mut col = self.cursor.col;
        while col < count
            && graphemes[col]
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_')
        {
            col += 1;
        }
        while col < count && graphemes[col].chars().all(|c| c.is_whitespace()) {
            col += 1;
        }
        self.cursor.col = col;
    }

    pub fn delete_word_backward(&mut self) {
        if self.cursor.col == 0 && self.cursor.row == 0 {
            return;
        }
        let start_cursor = self.cursor;
        self.move_word_left();
        let target_cursor = self.cursor;
        self.cursor = start_cursor;
        self.selection = Some(Selection::new(target_cursor, start_cursor));
        self.delete_selection();
    }

    pub fn delete_word_forward(&mut self) {
        let start_cursor = self.cursor;
        self.move_word_right();
        let target_cursor = self.cursor;
        self.cursor = start_cursor;
        self.selection = Some(Selection::new(start_cursor, target_cursor));
        self.delete_selection();
    }

    pub fn move_paragraph_prev(&mut self) {
        if self.cursor.row == 0 {
            self.cursor.col = 0;
            return;
        }
        let mut r = self.cursor.row.saturating_sub(1);
        while r > 0
            && self
                .lines
                .get(r)
                .map(|l| l.trim().is_empty())
                .unwrap_or(false)
        {
            r -= 1;
        }
        while r > 0
            && self
                .lines
                .get(r)
                .map(|l| !l.trim().is_empty())
                .unwrap_or(false)
        {
            r -= 1;
        }
        self.cursor.row = r;
        self.cursor.col = 0;
    }

    pub fn move_paragraph_next(&mut self) {
        let max = self.lines.len().saturating_sub(1);
        if self.cursor.row >= max {
            self.cursor.col = self.current_line().graphemes(true).count();
            return;
        }
        let mut r = self.cursor.row + 1;
        while r < max
            && self
                .lines
                .get(r)
                .map(|l| !l.trim().is_empty())
                .unwrap_or(false)
        {
            r += 1;
        }
        while r < max
            && self
                .lines
                .get(r)
                .map(|l| l.trim().is_empty())
                .unwrap_or(false)
        {
            r += 1;
        }
        self.cursor.row = r;
        self.cursor.col = 0;
    }

    pub fn add_cursor(&mut self, cursor: Cursor) {
        if cursor != self.cursor && !self.secondary_cursors.contains(&cursor) {
            self.secondary_cursors.push(cursor);
        }
    }

    pub fn clear_secondary_cursors(&mut self) {
        self.secondary_cursors.clear();
    }

    pub fn all_cursors(&self) -> Vec<Cursor> {
        let mut all = vec![self.cursor];
        all.extend(self.secondary_cursors.iter().copied());
        all
    }

    // ── Visual Selection ─────────────────────────────────────────────────────

    pub fn start_selection(&mut self) {
        if self.selection.is_none() {
            self.selection = Some(Selection::new(self.cursor, self.cursor));
        }
    }

    pub fn update_selection(&mut self) {
        if let Some(ref mut sel) = self.selection {
            sel.head = self.cursor;
        }
    }

    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    pub fn has_selection(&self) -> bool {
        self.selection.as_ref().is_some_and(|s| !s.is_empty())
    }

    pub fn select_all(&mut self) {
        let start = Cursor { row: 0, col: 0 };
        let end_row = self.lines.len().saturating_sub(1);
        let end_col = self
            .lines
            .last()
            .map(|l| l.graphemes(true).count())
            .unwrap_or(0);
        let end = Cursor {
            row: end_row,
            col: end_col,
        };
        self.selection = Some(Selection::new(start, end));
        self.cursor = end;
    }

    pub fn select_line(&mut self) {
        let row = self.cursor.row;
        let start = Cursor { row, col: 0 };
        let end = Cursor {
            row,
            col: self.current_line().graphemes(true).count(),
        };
        self.selection = Some(Selection::new(start, end));
        self.cursor = end;
    }

    pub fn select_word_at_cursor(&mut self) {
        let graphemes: Vec<&str> = self.current_line().graphemes(true).collect();
        if graphemes.is_empty() {
            return;
        }
        let col = self.cursor.col.min(graphemes.len().saturating_sub(1));
        let mut start_col = col;
        while start_col > 0
            && graphemes[start_col - 1]
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_')
        {
            start_col -= 1;
        }
        let mut end_col = col;
        while end_col < graphemes.len()
            && graphemes[end_col]
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_')
        {
            end_col += 1;
        }
        let start = Cursor {
            row: self.cursor.row,
            col: start_col,
        };
        let end = Cursor {
            row: self.cursor.row,
            col: end_col,
        };
        self.selection = Some(Selection::new(start, end));
        self.cursor = end;
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
        self.ensure_rope_synced();
        let text = self.selected_text()?;
        let sel = self.selection.take()?;
        let start = sel.start();
        let end = sel.end();

        let cursor_before = self.cursor;
        let char_idx = self.char_offset_for_cursor(start.row, start.col);

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
        let cursor_after = self.cursor;

        let del_len = text.chars().count();
        if char_idx + del_len <= self.rope.len_chars() {
            self.rope.remove(char_idx..char_idx + del_len);
        }

        self.push_transaction(UndoTransaction {
            deltas: vec![EditDelta::Delete {
                char_idx,
                text: text.clone(),
            }],
            cursor_before,
            cursor_after,
        });

        self.mark_dirty_and_invalidate_syntax(start.row);
        self.clamp_cursor();
        Some(text)
    }

    // ── Editing Operations (Unicode / Grapheme Safe & Delta Tracked) ─────────

    pub fn insert_char(&mut self, c: char) {
        let s = c.to_string();
        self.insert_grapheme(&s);
    }

    pub fn insert_grapheme(&mut self, g: &str) {
        self.ensure_rope_synced();
        let row = self.cursor.row;
        let col = self.cursor.col;

        while self.lines.len() <= row {
            self.lines.push(String::new());
        }

        let char_idx = self.char_offset_for_cursor(row, col);
        let cursor_before = self.cursor;

        let mut graphemes: Vec<String> = self.lines[row]
            .graphemes(true)
            .map(str::to_string)
            .collect();
        let safe_col = col.min(graphemes.len());
        graphemes.insert(safe_col, g.to_string());
        self.lines[row] = graphemes.concat();
        self.cursor.col = safe_col + 1;
        let cursor_after = self.cursor;

        let insert_idx = char_idx.min(self.rope.len_chars());
        self.rope.insert(insert_idx, g);

        self.push_transaction(UndoTransaction {
            deltas: vec![EditDelta::Insert {
                char_idx: insert_idx,
                text: g.to_string(),
            }],
            cursor_before,
            cursor_after,
        });
        self.mark_dirty_and_invalidate_syntax(row);
    }

    pub fn insert_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.ensure_rope_synced();
        let cursor_before = self.cursor;
        let char_idx = self.char_offset_for_cursor(cursor_before.row, cursor_before.col);

        let insert_idx = char_idx.min(self.rope.len_chars());
        self.rope.insert(insert_idx, text);
        self.sync_lines_from_rope();

        let new_cursor_char = insert_idx + text.chars().count();
        self.cursor = self.cursor_for_char_offset(new_cursor_char);
        let cursor_after = self.cursor;

        self.push_transaction(UndoTransaction {
            deltas: vec![EditDelta::Insert {
                char_idx: insert_idx,
                text: text.to_string(),
            }],
            cursor_before,
            cursor_after,
        });
        self.mark_dirty_and_invalidate_syntax(cursor_before.row);
    }

    pub fn insert_newline(&mut self) {
        self.ensure_rope_synced();
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

        let cursor_before = self.cursor;
        let char_idx = self.char_offset_for_cursor(row, col);

        self.lines[row] = prefix;
        self.lines.insert(row + 1, format!("{indent}{suffix}"));
        self.cursor.row += 1;
        self.cursor.col = indent.graphemes(true).count();
        let cursor_after = self.cursor;

        let sep = self.line_ending.as_str();
        let newline_text = format!("{sep}{indent}");
        let insert_idx = char_idx.min(self.rope.len_chars());
        self.rope.insert(insert_idx, &newline_text);

        self.push_transaction(UndoTransaction {
            deltas: vec![EditDelta::Insert {
                char_idx: insert_idx,
                text: newline_text,
            }],
            cursor_before,
            cursor_after,
        });
        self.mark_dirty_and_invalidate_syntax(row);
    }

    pub fn backspace(&mut self) {
        self.ensure_rope_synced();
        let row = self.cursor.row;
        let col = self.cursor.col;

        if col > 0 {
            let mut graphemes: Vec<String> = self.lines[row]
                .graphemes(true)
                .map(str::to_string)
                .collect();
            let safe_col = col.min(graphemes.len());
            if safe_col > 0 {
                let cursor_before = self.cursor;
                let char_idx = self.char_offset_for_cursor(row, safe_col - 1);
                let deleted_g = graphemes.remove(safe_col - 1);
                self.lines[row] = graphemes.concat();
                self.cursor.col = safe_col - 1;
                let cursor_after = self.cursor;

                let del_len = deleted_g.chars().count();
                if char_idx + del_len <= self.rope.len_chars() {
                    self.rope.remove(char_idx..char_idx + del_len);
                }

                self.push_transaction(UndoTransaction {
                    deltas: vec![EditDelta::Delete {
                        char_idx,
                        text: deleted_g,
                    }],
                    cursor_before,
                    cursor_after,
                });
                self.mark_dirty_and_invalidate_syntax(row);
            }
        } else if row > 0 {
            let cursor_before = self.cursor;
            let prev_len = self.lines[row - 1].graphemes(true).count();
            let char_idx = self.char_offset_for_cursor(row - 1, prev_len);
            let sep = self.line_ending.as_str();
            let sep_len = sep.chars().count();

            let cur = self.lines.remove(row);
            self.lines[row - 1].push_str(&cur);
            self.cursor.row -= 1;
            self.cursor.col = prev_len;
            let cursor_after = self.cursor;

            if char_idx + sep_len <= self.rope.len_chars() {
                self.rope.remove(char_idx..char_idx + sep_len);
            }

            self.push_transaction(UndoTransaction {
                deltas: vec![EditDelta::Delete {
                    char_idx,
                    text: sep.to_string(),
                }],
                cursor_before,
                cursor_after,
            });
            self.mark_dirty_and_invalidate_syntax(row - 1);
        }
    }

    pub fn delete_char(&mut self) {
        self.ensure_rope_synced();
        let row = self.cursor.row;
        let col = self.cursor.col;

        if row < self.lines.len() {
            let mut graphemes: Vec<String> = self.lines[row]
                .graphemes(true)
                .map(str::to_string)
                .collect();
            if col < graphemes.len() {
                let cursor_before = self.cursor;
                let char_idx = self.char_offset_for_cursor(row, col);
                let deleted_g = graphemes.remove(col);
                self.lines[row] = graphemes.concat();
                let cursor_after = self.cursor;

                let del_len = deleted_g.chars().count();
                if char_idx + del_len <= self.rope.len_chars() {
                    self.rope.remove(char_idx..char_idx + del_len);
                }

                self.push_transaction(UndoTransaction {
                    deltas: vec![EditDelta::Delete {
                        char_idx,
                        text: deleted_g,
                    }],
                    cursor_before,
                    cursor_after,
                });
                self.mark_dirty_and_invalidate_syntax(row);
            } else if row + 1 < self.lines.len() {
                let cursor_before = self.cursor;
                let char_idx = self.char_offset_for_cursor(row, graphemes.len());
                let sep = self.line_ending.as_str();
                let sep_len = sep.chars().count();
                let next = self.lines.remove(row + 1);
                self.lines[row].push_str(&next);
                let cursor_after = self.cursor;

                if char_idx + sep_len <= self.rope.len_chars() {
                    self.rope.remove(char_idx..char_idx + sep_len);
                }

                self.push_transaction(UndoTransaction {
                    deltas: vec![EditDelta::Delete {
                        char_idx,
                        text: sep.to_string(),
                    }],
                    cursor_before,
                    cursor_after,
                });
                self.mark_dirty_and_invalidate_syntax(row);
            }
        }
    }

    pub fn insert_tab(&mut self, tab_width: usize) {
        for _ in 0..tab_width {
            self.insert_char(' ');
        }
    }

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
            let old_line = self.lines[row].clone();
            let trimmed = old_line.trim_start();
            let cursor_before = self.cursor;

            if trimmed.starts_with(comment_prefix.trim()) {
                if let Some(pos) = old_line.find(comment_prefix) {
                    let mut graphemes: Vec<String> =
                        old_line.graphemes(true).map(str::to_string).collect();
                    let g_pos = old_line[..pos].graphemes(true).count();
                    let prefix_len = comment_prefix.graphemes(true).count();
                    graphemes.drain(g_pos..g_pos + prefix_len);
                    self.lines[row] = graphemes.concat();
                    self.cursor.col = self.cursor.col.saturating_sub(prefix_len);
                } else if let Some(pos) = old_line.find(comment_prefix.trim()) {
                    let mut graphemes: Vec<String> =
                        old_line.graphemes(true).map(str::to_string).collect();
                    let g_pos = old_line[..pos].graphemes(true).count();
                    let prefix_len = comment_prefix.trim().graphemes(true).count();
                    graphemes.drain(g_pos..g_pos + prefix_len);
                    self.lines[row] = graphemes.concat();
                    self.cursor.col = self.cursor.col.saturating_sub(prefix_len);
                }
            } else {
                let indent_len = old_line
                    .graphemes(true)
                    .take_while(|g| g.chars().all(|c| c.is_whitespace()))
                    .count();
                let mut graphemes: Vec<String> =
                    old_line.graphemes(true).map(str::to_string).collect();
                for (idx, g) in comment_prefix.graphemes(true).enumerate() {
                    graphemes.insert(indent_len + idx, g.to_string());
                }
                self.lines[row] = graphemes.concat();
                self.cursor.col += comment_prefix.graphemes(true).count();
            }

            let new_line = self.lines[row].clone();
            let char_idx = self.char_offset_for_cursor(row, 0);
            let cursor_after = self.cursor;

            self.push_transaction(UndoTransaction {
                deltas: vec![
                    EditDelta::Delete {
                        char_idx,
                        text: old_line,
                    },
                    EditDelta::Insert {
                        char_idx,
                        text: new_line,
                    },
                ],
                cursor_before,
                cursor_after,
            });
            self.mark_dirty_and_invalidate_syntax(row);
            self.sync_rope();
        }
    }

    pub fn wrapped_lines(&self, max_width: usize) -> Vec<(usize, String)> {
        if max_width == 0 {
            return self.lines.iter().cloned().enumerate().collect();
        }
        let mut result = Vec::new();
        for (line_idx, line) in self.lines.iter().enumerate() {
            if line.is_empty() {
                result.push((line_idx, String::new()));
                continue;
            }
            let words: Vec<&str> = line.split_inclusive(' ').collect();
            let mut cur = String::new();
            let mut cur_len = 0;
            for w in words {
                let w_len = w.chars().count();
                if cur_len + w_len > max_width && cur_len > 0 {
                    result.push((line_idx, cur));
                    cur = w.to_string();
                    cur_len = w_len;
                } else {
                    cur.push_str(w);
                    cur_len += w_len;
                }
            }
            if !cur.is_empty() || result.is_empty() {
                result.push((line_idx, cur));
            }
        }
        result
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
        self.ensure_rope_synced();
        if self.has_selection() {
            if let Some(text) = self.delete_selection() {
                self.clipboard = text;
            }
            return;
        }
        let row = self.cursor.row;
        let char_idx = self.char_offset_for_cursor(row, 0);
        let cursor_before = self.cursor;

        let sep = self.line_ending.as_str();
        let sep_len = sep.chars().count();

        if self.lines.len() > 1 {
            let removed = self.lines.remove(row);
            self.clipboard = removed.clone();
            if self.cursor.row >= self.lines.len() {
                self.cursor.row = self.lines.len() - 1;
            }
            let cursor_after = self.cursor;

            let del_len = removed.chars().count() + sep_len;
            if char_idx + del_len <= self.rope.len_chars() {
                self.rope.remove(char_idx..char_idx + del_len);
            }

            self.push_transaction(UndoTransaction {
                deltas: vec![EditDelta::Delete {
                    char_idx,
                    text: format!("{removed}{sep}"),
                }],
                cursor_before,
                cursor_after,
            });
        } else {
            let old_line = std::mem::take(&mut self.lines[0]);
            self.clipboard = old_line.clone();
            self.cursor.col = 0;
            let cursor_after = self.cursor;

            let del_len = old_line.chars().count();
            if char_idx + del_len <= self.rope.len_chars() {
                self.rope.remove(char_idx..char_idx + del_len);
            }

            self.push_transaction(UndoTransaction {
                deltas: vec![EditDelta::Delete {
                    char_idx,
                    text: old_line,
                }],
                cursor_before,
                cursor_after,
            });
        }
        self.mark_dirty_and_invalidate_syntax(row);
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
                let char_idx = self.char_offset_for_cursor(row, col);
                let prefix: String = chars[..col].iter().collect();
                let suffix: String = chars[col + query_chars..].iter().collect();
                self.lines[row] = format!("{prefix}{replacement}{suffix}");
                let cursor_before = self.cursor;
                self.cursor.col = col + replacement.chars().count();
                let cursor_after = self.cursor;

                self.push_transaction(UndoTransaction {
                    deltas: vec![
                        EditDelta::Delete {
                            char_idx,
                            text: sub,
                        },
                        EditDelta::Insert {
                            char_idx,
                            text: replacement.to_string(),
                        },
                    ],
                    cursor_before,
                    cursor_after,
                });
                self.mark_dirty_and_invalidate_syntax(row);
                self.sync_rope();
                return true;
            }
        }
        false
    }

    pub fn replace_all(&mut self, query: &str, replacement: &str) -> usize {
        if query.is_empty() {
            return 0;
        }
        let old_content = self.lines.join(self.line_ending.as_str());
        let mut count = 0;
        for line in &mut self.lines {
            if line.contains(query) {
                let occurrences = line.matches(query).count();
                *line = line.replace(query, replacement);
                count += occurrences;
            }
        }
        if count > 0 {
            let new_content = self.lines.join(self.line_ending.as_str());
            self.push_transaction(UndoTransaction {
                deltas: vec![
                    EditDelta::Delete {
                        char_idx: 0,
                        text: old_content,
                    },
                    EditDelta::Insert {
                        char_idx: 0,
                        text: new_content,
                    },
                ],
                cursor_before: self.cursor,
                cursor_after: self.cursor,
            });
            self.mark_dirty_and_invalidate_syntax(0);
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
        let loaded_mtime = std::fs::metadata(path).and_then(|m| m.modified()).ok();

        let mut buf = Self {
            path: Some(path.to_path_buf()),
            lines: Vec::new(),
            cursor: Cursor::default(),
            secondary_cursors: Vec::new(),
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
            has_trailing_newline: content.ends_with('\n'),
            rope,
            snippet_tab_stops: Vec::new(),
        };
        buf.sync_lines_from_rope();
        Ok(buf)
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
        self.ensure_rope_synced();
        let candidate = self.completion.candidates[self.completion.selected].clone();
        let prefix_graphemes = self.completion.prefix.graphemes(true).count();

        let row = self.cursor.row;
        let col = self.cursor.col;

        if col >= prefix_graphemes {
            let cursor_before = self.cursor;
            let start_col = col - prefix_graphemes;
            let char_idx = self.char_offset_for_cursor(row, start_col);

            let (insert_text, tab_offsets) = if let Some(ref body) = candidate.snippet_body {
                parse_snippet_tab_stops(body)
            } else {
                (candidate.label.clone(), Vec::new())
            };

            let prefix_char_count = self.completion.prefix.chars().count();
            if char_idx + prefix_char_count <= self.rope.len_chars() {
                self.rope.remove(char_idx..char_idx + prefix_char_count);
            }
            self.rope.insert(char_idx, &insert_text);
            self.sync_lines_from_rope();

            if let Some(&first_tab) = tab_offsets.first() {
                self.cursor = self.cursor_for_char_offset(char_idx + first_tab);
                self.snippet_tab_stops = tab_offsets[1..]
                    .iter()
                    .map(|&off| self.cursor_for_char_offset(char_idx + off))
                    .collect();
            } else {
                let end_char = char_idx + insert_text.chars().count();
                self.cursor = self.cursor_for_char_offset(end_char);
                self.snippet_tab_stops.clear();
            }
            let cursor_after = self.cursor;

            self.push_transaction(UndoTransaction {
                deltas: vec![
                    EditDelta::Delete {
                        char_idx,
                        text: self.completion.prefix.clone(),
                    },
                    EditDelta::Insert {
                        char_idx,
                        text: insert_text,
                    },
                ],
                cursor_before,
                cursor_after,
            });
            self.mark_dirty_and_invalidate_syntax(row);
            self.clamp_cursor();
        }

        self.completion.active = false;
        self.completion.candidates.clear();
        true
    }

    /// Advance cursor to the next active snippet tab stop, if any.
    pub fn jump_next_snippet_tab_stop(&mut self) -> bool {
        if !self.snippet_tab_stops.is_empty() {
            self.cursor = self.snippet_tab_stops.remove(0);
            self.clamp_cursor();
            true
        } else {
            false
        }
    }

    /// Insert character across primary and all secondary cursors atomically.
    pub fn insert_char_multicursor(&mut self, c: char) {
        if self.secondary_cursors.is_empty() {
            self.insert_char(c);
            return;
        }
        self.ensure_rope_synced();
        let s = c.to_string();
        let mut cursors = self.all_cursors();
        cursors.sort_by_key(|cur| (cur.row, cur.col));
        cursors.dedup();

        let s_len = s.chars().count();
        let mut new_positions = std::collections::HashMap::new();
        for (k, cur) in cursors.iter().enumerate() {
            let orig_off = self.char_offset_for_cursor(cur.row, cur.col);
            let new_off = orig_off + (k + 1) * s_len;
            new_positions.insert(*cur, new_off);
        }

        let mut offset_cursors: Vec<(usize, Cursor)> = cursors
            .into_iter()
            .map(|cur| (self.char_offset_for_cursor(cur.row, cur.col), cur))
            .collect();
        offset_cursors.sort_by_key(|a| std::cmp::Reverse(a.0));

        let mut deltas = Vec::new();
        let cursor_before = self.cursor;

        for (char_idx, _cur) in &offset_cursors {
            let idx = (*char_idx).min(self.rope.len_chars());
            self.rope.insert(idx, &s);
            deltas.push(EditDelta::Insert {
                char_idx: idx,
                text: s.clone(),
            });
        }

        self.sync_lines_from_rope();

        if let Some(&new_off) = new_positions.get(&self.cursor) {
            self.cursor = self.cursor_for_char_offset(new_off);
        }
        self.secondary_cursors = self
            .secondary_cursors
            .iter()
            .map(|sc| {
                if let Some(&new_off) = new_positions.get(sc) {
                    self.cursor_for_char_offset(new_off)
                } else {
                    *sc
                }
            })
            .collect();
        let cursor_after = self.cursor;

        self.push_transaction(UndoTransaction {
            deltas: vec![EditDelta::Batch(deltas)],
            cursor_before,
            cursor_after,
        });
        self.mark_dirty_and_invalidate_syntax(0);
    }

    /// Backspace across primary and all secondary cursors atomically.
    pub fn backspace_multicursor(&mut self) {
        if self.secondary_cursors.is_empty() {
            self.backspace();
            return;
        }
        self.ensure_rope_synced();
        let mut cursors = self.all_cursors();
        cursors.sort_by_key(|cur| (cur.row, cur.col));
        cursors.dedup();

        let mut del_count = 0;
        let mut new_positions = std::collections::HashMap::new();
        for cur in &cursors {
            let orig_off = self.char_offset_for_cursor(cur.row, cur.col);
            if orig_off > 0 || cur.row > 0 {
                del_count += 1;
            }
            let new_off = orig_off.saturating_sub(del_count);
            new_positions.insert(*cur, new_off);
        }

        let mut offset_cursors: Vec<(usize, Cursor)> = cursors
            .into_iter()
            .map(|cur| (self.char_offset_for_cursor(cur.row, cur.col), cur))
            .collect();
        offset_cursors.sort_by_key(|a| std::cmp::Reverse(a.0));

        let mut deltas = Vec::new();
        let cursor_before = self.cursor;

        for (char_idx, cur) in &offset_cursors {
            if *char_idx > 0 {
                let del_idx = char_idx - 1;
                if del_idx < self.rope.len_chars() {
                    let del_char = self.rope.char(del_idx).to_string();
                    self.rope.remove(del_idx..del_idx + 1);
                    deltas.push(EditDelta::Delete {
                        char_idx: del_idx,
                        text: del_char,
                    });
                }
            } else if cur.row > 0 {
                let sep = self.line_ending.as_str();
                let sep_len = sep.chars().count();
                let prev_offset = self.char_offset_for_cursor(cur.row - 1, 0);
                if prev_offset + sep_len <= self.rope.len_chars() {
                    self.rope.remove(prev_offset..prev_offset + sep_len);
                    deltas.push(EditDelta::Delete {
                        char_idx: prev_offset,
                        text: sep.to_string(),
                    });
                }
            }
        }

        self.sync_lines_from_rope();

        if let Some(&new_off) = new_positions.get(&self.cursor) {
            self.cursor = self.cursor_for_char_offset(new_off);
        }
        self.secondary_cursors = self
            .secondary_cursors
            .iter()
            .map(|sc| {
                if let Some(&new_off) = new_positions.get(sc) {
                    self.cursor_for_char_offset(new_off)
                } else {
                    *sc
                }
            })
            .collect();
        let cursor_after = self.cursor;

        if !deltas.is_empty() {
            self.push_transaction(UndoTransaction {
                deltas: vec![EditDelta::Batch(deltas)],
                cursor_before,
                cursor_after,
            });
            self.mark_dirty_and_invalidate_syntax(0);
        }
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
        if self.cursor.row < self.scroll_row {
            self.scroll_row = self.cursor.row;
        } else if self.cursor.row >= self.scroll_row + visible_rows {
            self.scroll_row = self.cursor.row + 1 - visible_rows;
        }
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

// ── Editor Workspace (Multi-Document Tab Manager) ───────────────────────────

#[derive(Debug, Default)]
pub struct EditorWorkspace {
    pub tabs: Vec<Buffer>,
    pub active_tab: usize,
}

impl EditorWorkspace {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: 0,
        }
    }

    pub fn with_buffer(buf: Buffer) -> Self {
        Self {
            tabs: vec![buf],
            active_tab: 0,
        }
    }

    pub fn open_buffer(&mut self, buf: Buffer) -> usize {
        if let Some(ref path) = buf.path {
            if let Some(idx) = self.tabs.iter().position(|b| b.path.as_ref() == Some(path)) {
                self.active_tab = idx;
                return idx;
            }
        }
        self.tabs.push(buf);
        self.active_tab = self.tabs.len() - 1;
        self.active_tab
    }

    pub fn open_file(&mut self, path: PathBuf) -> Result<usize> {
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|b| b.path.as_ref() == Some(&path))
        {
            self.active_tab = idx;
            return Ok(idx);
        }
        let buf = Buffer::from_file(path)?;
        self.tabs.push(buf);
        self.active_tab = self.tabs.len() - 1;
        Ok(self.active_tab)
    }

    pub fn close_tab(&mut self, idx: usize) -> Option<Buffer> {
        if idx >= self.tabs.len() {
            return None;
        }
        let removed = self.tabs.remove(idx);
        crate::editor::syntax::invalidate_syntax_cache_for_path(removed.path.as_deref());
        if self.active_tab >= self.tabs.len() && !self.tabs.is_empty() {
            self.active_tab = self.tabs.len() - 1;
        }
        Some(removed)
    }

    pub fn active(&self) -> Option<&Buffer> {
        self.tabs.get(self.active_tab)
    }

    pub fn active_mut(&mut self) -> Option<&mut Buffer> {
        self.tabs.get_mut(self.active_tab)
    }

    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab = (self.active_tab + 1) % self.tabs.len();
        }
    }

    pub fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            if self.active_tab == 0 {
                self.active_tab = self.tabs.len() - 1;
            } else {
                self.active_tab -= 1;
            }
        }
    }

    pub fn switch_tab(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            self.active_tab = idx;
        }
    }

    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    pub fn has_dirty_tabs(&self) -> bool {
        self.tabs.iter().any(|b| b.dirty)
    }
}

/// Parse snippet template into insertion string and ordered char offsets for tab stops.
pub fn parse_snippet_tab_stops(snippet: &str) -> (String, Vec<usize>) {
    let mut text = String::with_capacity(snippet.len());
    let mut tab_stops: std::collections::BTreeMap<usize, usize> = std::collections::BTreeMap::new();
    let chars: Vec<char> = snippet.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '$' && i + 1 < chars.len() {
            if chars[i + 1] == '{' {
                if let Some(close_brace) = chars[i + 2..].iter().position(|&c| c == '}') {
                    let inner: String = chars[i + 2..i + 2 + close_brace].iter().collect();
                    i = i + 3 + close_brace;
                    if let Some(colon_pos) = inner.find(':') {
                        let num_str = &inner[..colon_pos];
                        let default_val = &inner[colon_pos + 1..];
                        let num: usize = num_str.parse().unwrap_or(0);
                        let offset = text.chars().count();
                        tab_stops.insert(num, offset);
                        text.push_str(default_val);
                    } else {
                        let num: usize = inner.parse().unwrap_or(0);
                        let offset = text.chars().count();
                        tab_stops.insert(num, offset);
                    }
                    continue;
                }
            } else if chars[i + 1].is_ascii_digit() {
                let num: usize = chars[i + 1].to_digit(10).unwrap_or(0) as usize;
                let offset = text.chars().count();
                tab_stops.insert(num, offset);
                i += 2;
                continue;
            }
        }
        text.push(chars[i]);
        i += 1;
    }

    let mut ordered = Vec::new();
    for (&k, &off) in &tab_stops {
        if k > 0 {
            ordered.push(off);
        }
    }
    if let Some(&final_off) = tab_stops.get(&0) {
        ordered.push(final_off);
    }
    (text, ordered)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unicode_grapheme_navigation_and_editing() {
        let mut buf = Buffer::new();
        buf.insert_text("Hi 👍🏽 🇮🇳 👨‍💻 End");
        assert_eq!(buf.lines[0], "Hi 👍🏽 🇮🇳 👨‍💻 End");
        assert_eq!(buf.cursor.col, 12);

        for _ in 0..4 {
            buf.move_left();
        }
        assert_eq!(buf.cursor.col, 8);

        buf.move_left();
        assert_eq!(buf.cursor.col, 7);

        buf.move_right();
        assert_eq!(buf.cursor.col, 8);

        buf.backspace();
        assert_eq!(buf.cursor.col, 7);
        assert_eq!(buf.lines[0], "Hi 👍🏽 🇮🇳  End");

        buf.move_left();
        assert_eq!(buf.cursor.col, 6);

        buf.backspace();
        assert_eq!(buf.cursor.col, 5);
        assert_eq!(buf.lines[0], "Hi 👍🏽   End");

        buf.move_left();
        assert_eq!(buf.cursor.col, 4);

        buf.backspace();
        assert_eq!(buf.cursor.col, 3);
        assert_eq!(buf.lines[0], "Hi    End");
    }

    #[test]
    fn test_utf8_bom_preservation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test_bom.txt");

        let mut content_bytes = vec![0xEF, 0xBB, 0xBF];
        content_bytes.extend_from_slice(b"Hello BOM\nWorld");
        std::fs::write(&file_path, content_bytes).unwrap();

        let mut buf = Buffer::from_file(file_path.clone()).unwrap();
        assert_eq!(buf.encoding, FileEncoding::Utf8Bom);
        assert_eq!(buf.lines[0], "Hello BOM");
        assert_eq!(buf.lines[1], "World");

        buf.lines[0] = "Modified BOM".to_string();
        buf.save().unwrap();

        let saved = std::fs::read(&file_path).unwrap();
        assert_eq!(&saved[0..3], &[0xEF, 0xBB, 0xBF]);
        assert_eq!(&saved[3..], b"Modified BOM\nWorld");
    }

    #[test]
    fn test_latin1_fallback_encoding() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test_latin1.txt");

        let latin1_bytes = vec![b'C', b'a', b'f', 0xE9];
        std::fs::write(&file_path, latin1_bytes).unwrap();

        let mut buf = Buffer::from_file(file_path.clone()).unwrap();
        assert_eq!(buf.encoding, FileEncoding::Latin1);
        assert_eq!(buf.lines[0], "Café");

        buf.save().unwrap();
        let saved = std::fs::read(&file_path).unwrap();
        assert_eq!(saved, vec![b'C', b'a', b'f', 0xE9]);
    }

    #[test]
    fn test_word_and_paragraph_navigation() {
        let mut buf = Buffer::new();
        buf.lines = vec![
            "fn calculate_total_sum() -> u64 {".to_string(),
            "    let x = 100;".to_string(),
            "".to_string(),
            "    x * 2".to_string(),
            "}".to_string(),
        ];
        buf.sync_rope();

        buf.cursor = Cursor { row: 0, col: 0 };
        buf.move_word_right();
        assert_eq!(buf.cursor.col, 3); // after "fn "
        buf.move_word_right();
        assert_eq!(buf.cursor.col, 22); // after "calculate_total_sum", at "("

        buf.move_word_left();
        assert_eq!(buf.cursor.col, 3); // back to "calculate_total_sum"

        buf.move_paragraph_next();
        assert_eq!(buf.cursor.row, 3); // next paragraph block
    }

    #[test]
    fn test_editor_workspace_tabs() {
        let mut ws = EditorWorkspace::new();
        let mut buf1 = Buffer::new();
        buf1.path = Some(PathBuf::from("/tmp/file1.rs"));
        let mut buf2 = Buffer::new();
        buf2.path = Some(PathBuf::from("/tmp/file2.py"));

        ws.open_buffer(buf1);
        ws.open_buffer(buf2);
        assert_eq!(ws.tab_count(), 2);
        assert_eq!(ws.active_tab, 1);

        ws.prev_tab();
        assert_eq!(ws.active_tab, 0);
        ws.next_tab();
        assert_eq!(ws.active_tab, 1);

        let closed = ws.close_tab(1);
        assert!(closed.is_some());
        assert_eq!(ws.tab_count(), 1);
        assert_eq!(ws.active_tab, 0);
    }
}
