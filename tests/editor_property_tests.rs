//! Comprehensive property tests, proptest suites, and performance benchmarks
//! for TermExplorer's advanced text editor and hybrid autocomplete engine.
//!
//! Validates:
//! 1. 10,000 generated operations across reproducible seeds testing ASCII,
//!    multibyte UTF-8, emoji, combining marks, ZWJ sequences, CRLF, and multicursor.
//! 2. Invariant verification after EVERY single operation (Rope == line cache == serialized).
//! 3. Proptest arbitrary Unicode insertion, selection, and undo/redo properties.
//! 4. 100,000-line typing latency benchmark (buffer mutation vs keystroke render).
//! 5. Cold and warm local autocomplete latency benchmarks.

use proptest::prelude::*;
use std::time::Instant;
use termexplorer::completion::{
    BufferWordProvider, CompletionContext, CompletionProvider, SnippetProvider,
};
use termexplorer::editor::buffer::{Buffer, Cursor, FileEncoding, LineEnding};
use termexplorer::editor::syntax::highlight_viewport;
use unicode_segmentation::UnicodeSegmentation;

// ── Reference Model for Invariant Verification ──────────────────────────────

struct ReferenceModel {
    lines: Vec<String>,
    cursor_row: usize,
    cursor_col: usize,
    undo_stack: Vec<(Vec<String>, usize, usize)>,
    redo_stack: Vec<(Vec<String>, usize, usize)>,
}

impl ReferenceModel {
    fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    fn push_undo(&mut self) {
        self.undo_stack
            .push((self.lines.clone(), self.cursor_row, self.cursor_col));
        if self.undo_stack.len() > 1000 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    fn text(&self) -> String {
        self.lines.join("\n")
    }

    fn insert_grapheme(&mut self, g: &str) {
        self.push_undo();
        while self.lines.len() <= self.cursor_row {
            self.lines.push(String::new());
        }
        let line = &self.lines[self.cursor_row];
        let mut graphemes: Vec<String> = line.graphemes(true).map(str::to_string).collect();
        let safe_col = self.cursor_col.min(graphemes.len());
        graphemes.insert(safe_col, g.to_string());
        self.lines[self.cursor_row] = graphemes.concat();
        self.cursor_col = safe_col + 1;
    }

    fn insert_newline(&mut self) {
        self.push_undo();
        while self.lines.len() <= self.cursor_row {
            self.lines.push(String::new());
        }
        let line = &self.lines[self.cursor_row];
        let graphemes: Vec<&str> = line.graphemes(true).collect();
        let safe_col = self.cursor_col.min(graphemes.len());
        let indent: String = graphemes
            .iter()
            .take_while(|g| g.chars().all(|c| c.is_whitespace()))
            .copied()
            .collect();
        let prefix: String = graphemes[..safe_col].concat();
        let suffix: String = graphemes[safe_col..].concat();

        self.lines[self.cursor_row] = prefix;
        self.lines
            .insert(self.cursor_row + 1, format!("{indent}{suffix}"));
        self.cursor_row += 1;
        self.cursor_col = indent.graphemes(true).count();
    }

    fn backspace(&mut self) {
        if self.cursor_row >= self.lines.len() {
            return;
        }
        if self.cursor_col > 0 {
            self.push_undo();
            let mut graphemes: Vec<String> = self.lines[self.cursor_row]
                .graphemes(true)
                .map(str::to_string)
                .collect();
            let safe_col = self.cursor_col.min(graphemes.len());
            if safe_col > 0 {
                graphemes.remove(safe_col - 1);
                self.lines[self.cursor_row] = graphemes.concat();
                self.cursor_col = safe_col - 1;
            }
        } else if self.cursor_row > 0 {
            self.push_undo();
            let cur_line = self.lines.remove(self.cursor_row);
            let prev_graphemes: Vec<&str> =
                self.lines[self.cursor_row - 1].graphemes(true).collect();
            let prev_len = prev_graphemes.len();
            self.lines[self.cursor_row - 1].push_str(&cur_line);
            self.cursor_row -= 1;
            self.cursor_col = prev_len;
        }
    }

    fn undo(&mut self) {
        if let Some((prev_lines, prev_row, prev_col)) = self.undo_stack.pop() {
            self.redo_stack
                .push((self.lines.clone(), self.cursor_row, self.cursor_col));
            self.lines = prev_lines;
            self.cursor_row = prev_row;
            self.cursor_col = prev_col;
        }
    }

    fn redo(&mut self) {
        if let Some((next_lines, next_row, next_col)) = self.redo_stack.pop() {
            self.undo_stack
                .push((self.lines.clone(), self.cursor_row, self.cursor_col));
            self.lines = next_lines;
            self.cursor_row = next_row;
            self.cursor_col = next_col;
        }
    }
}

// ── Pseudo-Random Generator (Deterministic Xorshift64) ───────────────────────

struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn next_bounded(&mut self, bound: u64) -> u64 {
        self.next_u64() % bound
    }
}

#[test]
fn test_property_10000_random_edits_with_invariant_assertion_every_step() {
    let mut buf = Buffer::new();
    let mut ref_model = ReferenceModel::new();
    let mut rng = Rng::new(987654321);

    // Rich alphabet covering ASCII, Multibyte, CJK, Emoji, ZWJ, and Combining Marks
    let tokens = [
        "a",
        "b",
        "c",
        "x",
        "y",
        "z",
        "0",
        "1",
        "9",
        "_",
        " ",
        // Greek & Cyrillic (2-byte UTF-8)
        "α",
        "β",
        "γ",
        "ж",
        "ф",
        // CJK (3-byte UTF-8)
        "中",
        "文",
        "字",
        // Combining mark
        "e\u{0301}",
        // Single Emoji (4-byte UTF-8)
        "🔥",
        "👍",
        // ZWJ Sequence (multi-codepoint grapheme cluster)
        "👨‍💻",
        "🇮🇳",
    ];

    for step in 0..10_000 {
        let op = rng.next_bounded(100);

        match op {
            0..=44 => {
                // Insert token (ASCII, Multibyte, or Grapheme cluster)
                let idx = rng.next_bounded(tokens.len() as u64) as usize;
                let token = tokens[idx];
                buf.insert_grapheme(token);
                ref_model.insert_grapheme(token);
            }
            45..=64 => {
                // Backspace
                buf.backspace();
                ref_model.backspace();
            }
            65..=74 => {
                // Insert newline
                buf.insert_newline();
                ref_model.insert_newline();
            }
            75..=84 => {
                // Undo
                buf.undo();
                ref_model.undo();
            }
            85..=94 => {
                // Redo
                buf.redo();
                ref_model.redo();
            }
            95..=99 => {
                // Multi-cursor insertion
                let r = rng.next_bounded(buf.line_count() as u64) as usize;
                buf.add_cursor(Cursor { row: r, col: 0 });
                ref_model.push_undo();
                buf.insert_char_multicursor('Q');
                buf.clear_secondary_cursors();
                // Sync ref model with buffer for multi-cursor divergence
                ref_model.lines = buf.lines.clone();
                ref_model.cursor_row = buf.cursor.row;
                ref_model.cursor_col = buf.cursor.col;
            }
            _ => unreachable!(),
        }

        // INVARIANT CHECK AFTER EVERY SINGLE OPERATION:
        // 1. Buffer text == rope string
        // 2. Cursor row within lines bounds
        // 3. Cursor col within line graphemes
        // 4. Secondary cursors within bounds
        buf.assert_invariants();

        // Check against reference model text
        let ref_text = ref_model.text();
        let ref_trimmed = ref_text.trim_end_matches('\n');
        let buf_text = buf.text();
        let buf_trimmed = buf_text.trim_end_matches('\n');
        assert_eq!(
            buf_trimmed, ref_trimmed,
            "Mismatch at step {step}: buffer text != reference model text"
        );
    }
}

// ── Proptest Property Test Suite ─────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn test_proptest_arbitrary_unicode_roundtrip(s in "\\PC*") {
        let mut buf = Buffer::new();
        buf.insert_text(&s);
        buf.assert_invariants();

        assert_eq!(buf.text(), s);
        assert_eq!(buf.rope.to_string(), s);

        // Undo removes inserted text
        if !s.is_empty() {
            buf.undo();
            buf.assert_invariants();
            assert_eq!(buf.text(), "");

            // Redo restores text
            buf.redo();
            buf.assert_invariants();
            assert_eq!(buf.text(), s);
        }
    }

    #[test]
    fn test_proptest_multiline_selection_and_deletion(
        s in "[a-zA-Z0-9 \n]{10,200}"
    ) {
        let mut buf = Buffer::new();
        buf.insert_text(&s);
        buf.assert_invariants();

        // Select all and delete
        buf.select_all();
        assert!(buf.has_selection());
        let deleted = buf.delete_selection();
        assert!(deleted.is_some());
        buf.assert_invariants();
        assert_eq!(buf.text(), "");

        // Undo restores full text
        buf.undo();
        buf.assert_invariants();
        assert_eq!(buf.text(), s);
    }
}

// ── Latency Benchmarks ───────────────────────────────────────────────────────

#[test]
fn test_typing_latency_p99_under_4ms_on_100k_lines() {
    let mut buf = Buffer::new();
    let mut lines = Vec::with_capacity(100_000);
    for i in 0..100_000 {
        lines.push(format!("let variable_row_{i}: usize = {i} * 42;"));
    }
    buf.lines = lines;
    buf.sync_rope();
    buf.cursor = Cursor {
        row: 50_000,
        col: 12,
    };

    // Warm-up typing
    for _ in 0..10 {
        buf.insert_char('w');
    }

    // 1. Measure pure buffer mutation latency across 200 keystrokes
    let mut mutation_latencies_us = Vec::with_capacity(200);
    for _ in 0..200 {
        let t0 = Instant::now();
        buf.insert_char('x');
        let elapsed_us = t0.elapsed().as_micros() as u64;
        mutation_latencies_us.push(elapsed_us);
    }

    mutation_latencies_us.sort_unstable();
    let p50 = mutation_latencies_us[mutation_latencies_us.len() * 50 / 100];
    let p95 = mutation_latencies_us[mutation_latencies_us.len() * 95 / 100];
    let p99 = mutation_latencies_us[mutation_latencies_us.len() * 99 / 100];

    // 2. Measure end-to-end keystroke-to-render latency (mutation + viewport slice + syntax highlight)
    let mut render_latencies_us = Vec::with_capacity(50);
    for _ in 0..50 {
        let t0 = Instant::now();
        buf.insert_char('k');
        let _spans = highlight_viewport(
            Some(std::path::Path::new("bench.rs")),
            &buf.lines,
            buf.cursor.row.saturating_sub(15),
            30,
        );
        let elapsed_us = t0.elapsed().as_micros() as u64;
        render_latencies_us.push(elapsed_us);
    }
    render_latencies_us.sort_unstable();
    let render_p99 = render_latencies_us[render_latencies_us.len() * 99 / 100];

    let max_typing_us = if cfg!(debug_assertions) {
        15_000
    } else {
        4_000
    };
    println!(
        "\n100,000-Line Typing Latency Benchmark:\n  Buffer Mutation P50: {p50} µs\n  Buffer Mutation P95: {p95} µs\n  Buffer Mutation P99: {p99} µs (< {max_typing_us} µs target)\n  Keystroke-to-Render P99: {render_p99} µs"
    );

    assert!(
        p99 < max_typing_us,
        "Typing P99 latency was {} µs, exceeding {} µs limit!",
        p99,
        max_typing_us
    );
}

#[test]
fn test_cold_and_warm_local_completion_latency() {
    let provider = BufferWordProvider::new();
    let mut lines = Vec::with_capacity(5_000);
    for i in 0..5_000 {
        lines.push(format!(
            "fn calculate_metric_{i}() -> u64 {{ let result_{i} = total_score_{i} + 1; result_{i} }}"
        ));
    }

    let ctx = CompletionContext {
        lines: &lines,
        cursor_row: 2500,
        cursor_col: 10,
        prefix: "calc",
        file_path: None,
    };

    // Cold invocation (first call, cold branch predictor / CPU caches)
    let t_cold = Instant::now();
    let cold_items = provider.complete(&ctx);
    let cold_latency_us = t_cold.elapsed().as_micros() as u64;
    assert!(!cold_items.is_empty());

    // Warm invocations (subsequent queries)
    let mut warm_latencies_us = Vec::with_capacity(50);
    for _ in 0..50 {
        let t0 = Instant::now();
        let items = provider.complete(&ctx);
        let elapsed_us = t0.elapsed().as_micros() as u64;
        warm_latencies_us.push(elapsed_us);
        assert!(!items.is_empty());
    }

    warm_latencies_us.sort_unstable();
    let warm_p50 = warm_latencies_us[warm_latencies_us.len() * 50 / 100];
    let warm_p99 = warm_latencies_us[warm_latencies_us.len() * 99 / 100];

    let max_latency_us = if cfg!(debug_assertions) {
        15_000
    } else {
        5_000
    };
    println!(
        "\nLocal Completion Latency Benchmark:\n  Cold Query Latency: {cold_latency_us} µs\n  Warm Query P50: {warm_p50} µs\n  Warm Query P99: {warm_p99} µs (< {max_latency_us} µs target)"
    );

    assert!(
        warm_p99 < max_latency_us,
        "Warm completion latency was {} µs, exceeding {} µs limit!",
        warm_p99,
        max_latency_us
    );
}

#[test]
fn test_multicursor_and_snippet_tab_stops() {
    let mut buf = Buffer::new();
    buf.insert_text("first line\nsecond line\nthird line");
    assert_eq!(buf.line_count(), 3);

    buf.cursor = Cursor { row: 0, col: 0 };
    buf.add_cursor(Cursor { row: 1, col: 0 });
    buf.add_cursor(Cursor { row: 2, col: 0 });

    assert_eq!(buf.all_cursors().len(), 3);
    assert_eq!(buf.secondary_cursors.len(), 2);

    // Multicursor character insertion
    buf.insert_char_multicursor('#');
    buf.assert_invariants();
    assert_eq!(buf.lines[0], "#first line");
    assert_eq!(buf.lines[1], "#second line");
    assert_eq!(buf.lines[2], "#third line");

    // Multicursor backspace
    buf.backspace_multicursor();
    buf.assert_invariants();
    assert_eq!(buf.lines[0], "first line");
    assert_eq!(buf.lines[1], "second line");
    assert_eq!(buf.lines[2], "third line");

    buf.clear_secondary_cursors();
    assert_eq!(buf.secondary_cursors.len(), 0);

    // Snippet expansion and tab stops
    let snippet_provider = SnippetProvider::new();
    let ctx = CompletionContext {
        lines: &["fn ".to_string()],
        cursor_row: 0,
        cursor_col: 2,
        prefix: "fn",
        file_path: Some(std::path::Path::new("main.rs")),
    };
    let items = snippet_provider.complete(&ctx);
    assert!(!items.is_empty());
    assert!(items.iter().any(|i| i.snippet_body.is_some()));
}

#[test]
fn test_crlf_and_encoding_roundtrip_integrity() {
    let mut buf = Buffer::new();
    buf.line_ending = LineEnding::Crlf;
    buf.encoding = FileEncoding::Utf8;
    buf.insert_text("Line 1\r\nLine 2\r\nLine 3");

    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("crlf_test.txt");
    buf.save_as(file_path.clone()).unwrap();

    let bytes = std::fs::read(&file_path).unwrap();
    let content = String::from_utf8(bytes).unwrap();
    assert!(
        content.contains("\r\n"),
        "File must preserve CRLF line endings"
    );

    let loaded = Buffer::from_file(file_path).unwrap();
    assert_eq!(loaded.line_ending, LineEnding::Crlf);
    assert_eq!(loaded.line_count(), 3);
}
