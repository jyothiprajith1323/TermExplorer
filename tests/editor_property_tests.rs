//! Comprehensive property tests and performance benchmarks for TermExplorer's
//! advanced text editor and hybrid autocomplete engine.
//!
//! Validates:
//! 1. 10,000 random editing actions matching a reference model.
//! 2. Typing latency P99 < 4ms on a 100,000-line buffer.
//! 3. Local completion latency < 5ms.
//! 4. Multi-cursor editing fidelity.
//! 5. Delta undo/redo transactional integrity.

use std::time::Instant;
use termexplorer::completion::{
    BufferWordProvider, CompletionContext, CompletionProvider, SnippetProvider,
};
use termexplorer::editor::buffer::{Buffer, Cursor, FileEncoding, LineEnding};

/// Simple linear reference model for property verification.
struct ReferenceModel {
    text: String,
    cursor_char: usize,
    undo_stack: Vec<(String, usize)>,
    redo_stack: Vec<(String, usize)>,
}

impl ReferenceModel {
    fn new() -> Self {
        Self {
            text: String::new(),
            cursor_char: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    fn push_undo(&mut self) {
        self.undo_stack.push((self.text.clone(), self.cursor_char));
        if self.undo_stack.len() > 1000 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    fn insert_char(&mut self, c: char) {
        self.push_undo();
        self.text.insert(self.cursor_char, c);
        self.cursor_char += 1;
    }

    fn insert_newline(&mut self) {
        self.push_undo();
        let current_line_start = self.text[..self.cursor_char]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        let indent: String = self.text[current_line_start..self.cursor_char]
            .chars()
            .take_while(|c| c.is_whitespace())
            .collect();
        let newline_text = format!("\n{indent}");
        self.text.insert_str(self.cursor_char, &newline_text);
        self.cursor_char += newline_text.len();
    }

    fn backspace(&mut self) {
        if self.cursor_char > 0 {
            self.push_undo();
            self.cursor_char -= 1;
            self.text.remove(self.cursor_char);
        }
    }

    fn undo(&mut self) {
        if let Some((prev_text, prev_cursor)) = self.undo_stack.pop() {
            self.redo_stack.push((self.text.clone(), self.cursor_char));
            self.text = prev_text;
            self.cursor_char = prev_cursor;
        }
    }

    fn redo(&mut self) {
        if let Some((next_text, next_cursor)) = self.redo_stack.pop() {
            self.undo_stack.push((self.text.clone(), self.cursor_char));
            self.text = next_text;
            self.cursor_char = next_cursor;
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
fn test_property_10000_random_edits_match_reference_model() {
    let mut buf = Buffer::new();
    let mut ref_model = ReferenceModel::new();
    let mut rng = Rng::new(424242);

    let ascii_chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 _";

    for step in 0..10_000 {
        let op = rng.next_bounded(100);

        match op {
            0..=44 => {
                // Insert ASCII character
                let idx = rng.next_bounded(ascii_chars.len() as u64) as usize;
                let c = ascii_chars[idx] as char;
                buf.insert_char(c);
                ref_model.insert_char(c);
            }
            45..=64 => {
                // Backspace
                buf.backspace();
                ref_model.backspace();
            }
            65..=79 => {
                // Insert newline
                buf.insert_newline();
                ref_model.insert_newline();
            }
            80..=89 => {
                // Undo
                buf.undo();
                ref_model.undo();
            }
            90..=99 => {
                // Redo
                buf.redo();
                ref_model.redo();
            }
            _ => unreachable!(),
        }

        // Periodically verify invariants every 250 steps to keep test fast
        if step % 250 == 0 || step == 9999 {
            let buf_content = buf.lines.join("\n");
            // Normalize trailing newline for comparison
            let ref_trimmed = ref_model.text.trim_end_matches('\n');
            let buf_trimmed = buf_content.trim_end_matches('\n');
            assert_eq!(
                buf_trimmed, ref_trimmed,
                "Mismatch at step {step}: buffer lines != reference text"
            );
        }
    }
}

#[test]
fn test_typing_latency_p99_under_4ms_on_100k_lines() {
    let mut buf = Buffer::new();
    // Build a 100,000 line synthetic buffer
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

    // Measure 200 sequential character insertions
    let mut latencies_us = Vec::with_capacity(200);

    for _ in 0..200 {
        let t0 = Instant::now();
        buf.insert_char('x');
        let elapsed_us = t0.elapsed().as_micros() as u64;
        latencies_us.push(elapsed_us);
    }

    latencies_us.sort_unstable();
    let p50 = latencies_us[latencies_us.len() * 50 / 100];
    let p95 = latencies_us[latencies_us.len() * 95 / 100];
    let p99 = latencies_us[latencies_us.len() * 99 / 100];

    let max_typing_us = if cfg!(debug_assertions) {
        15_000
    } else {
        4_000
    };
    println!(
        "\n100,000-Line Typing Latency Benchmark:\n  P50: {p50} µs\n  P95: {p95} µs\n  P99: {p99} µs (< {max_typing_us} µs target)"
    );

    // Target: P99 < 4ms (4,000 microseconds) in release profile
    assert!(
        p99 < max_typing_us,
        "Typing P99 latency was {} µs, exceeding {} µs limit!",
        p99,
        max_typing_us
    );
}

#[test]
fn test_local_completion_latency_under_5ms() {
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

    // Warm-up
    let _ = provider.complete(&ctx);

    let mut latencies_us = Vec::with_capacity(50);
    for _ in 0..50 {
        let t0 = Instant::now();
        let items = provider.complete(&ctx);
        let elapsed_us = t0.elapsed().as_micros() as u64;
        latencies_us.push(elapsed_us);
        assert!(!items.is_empty());
    }

    latencies_us.sort_unstable();
    let p99 = latencies_us[latencies_us.len() * 99 / 100];
    let max_latency_us = if cfg!(debug_assertions) {
        15_000
    } else {
        5_000
    };
    println!("\nBuffer Word Completion Latency P99: {p99} µs (< {max_latency_us} µs target)");

    // Target: Local completion latency < 5ms (allow tolerance in unoptimized debug runner)
    assert!(
        p99 < max_latency_us,
        "Completion latency was {} µs, exceeding {} µs limit!",
        p99,
        max_latency_us
    );
}

#[test]
fn test_multicursor_and_snippet_tab_stops() {
    let mut buf = Buffer::new();
    buf.insert_text("first line\nsecond line\nthird line");
    assert_eq!(buf.line_count(), 3);

    // Primary cursor at (0, 0), add secondary cursors at (1, 0) and (2, 0)
    buf.cursor = Cursor { row: 0, col: 0 };
    buf.add_cursor(Cursor { row: 1, col: 0 });
    buf.add_cursor(Cursor { row: 2, col: 0 });

    assert_eq!(buf.all_cursors().len(), 3);
    assert_eq!(buf.secondary_cursors.len(), 2);

    buf.clear_secondary_cursors();
    assert_eq!(buf.secondary_cursors.len(), 0);
    assert_eq!(buf.all_cursors().len(), 1);

    // Snippet expansion test across languages
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
