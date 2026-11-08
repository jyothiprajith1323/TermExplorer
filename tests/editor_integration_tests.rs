//! Comprehensive Editor & Completion Engine Integration Tests.

use std::path::PathBuf;
use tempfile::tempdir;
use termexplorer::completion::engine::CentralCompletionEngine;
use termexplorer::completion::provider::CompletionContext;
use termexplorer::editor::buffer::{Buffer, Cursor, LineEnding};

#[test]
fn test_rope_backing_and_line_synchronization() {
    let mut buf = Buffer::new();
    assert_eq!(buf.line_count(), 1);

    buf.insert_char('A');
    buf.insert_newline();
    buf.insert_char('B');

    assert_eq!(buf.line_count(), 2);
    assert_eq!(buf.lines[0], "A");
    assert_eq!(buf.lines[1], "B");
    assert!(buf.total_char_len() >= 2);
}

#[test]
fn test_visual_selection_and_deletion() {
    let mut buf = Buffer::new();
    buf.lines = vec![
        "First line of text".to_string(),
        "Second line of text".to_string(),
    ];
    buf.cursor = Cursor { row: 0, col: 6 };

    // Start selection at (0, 6) "line"
    buf.start_selection();
    buf.cursor = Cursor { row: 0, col: 10 };
    buf.update_selection();

    assert!(buf.has_selection());
    assert_eq!(buf.selected_text(), Some("line".to_string()));

    // Delete selection
    let deleted = buf.delete_selection();
    assert_eq!(deleted, Some("line".to_string()));
    assert_eq!(buf.lines[0], "First  of text");
    assert!(!buf.has_selection());

    // Undo should restore selection text
    buf.undo();
    assert_eq!(buf.lines[0], "First line of text");
}

#[test]
fn test_search_and_replace_all() {
    let mut buf = Buffer::new();
    buf.lines = vec![
        "foo bar foo".to_string(),
        "another foo line".to_string(),
        "baz without match".to_string(),
    ];

    buf.search_query = "foo".to_string();
    buf.cursor = Cursor { row: 0, col: 0 };

    // Search next match
    buf.search_next();
    assert_eq!(buf.cursor.row, 1);
    assert_eq!(buf.cursor.col, 8);

    // Replace all
    let replaced_count = buf.replace_all("foo", "qux");
    assert_eq!(replaced_count, 3);
    assert_eq!(buf.lines[0], "qux bar qux");
    assert_eq!(buf.lines[1], "another qux line");
    assert_eq!(buf.lines[2], "baz without match");

    // Undo restores all replacements
    buf.undo();
    assert_eq!(buf.lines[0], "foo bar foo");
    assert_eq!(buf.lines[1], "another foo line");
}

#[test]
fn test_toggle_line_comment_multi_language() {
    // Rust file
    let mut rust_buf = Buffer::new();
    rust_buf.path = Some(PathBuf::from("src/main.rs"));
    rust_buf.lines = vec!["    let x = 42;".to_string()];
    rust_buf.cursor = Cursor { row: 0, col: 4 };

    rust_buf.toggle_comment();
    assert_eq!(rust_buf.lines[0], "    // let x = 42;");

    rust_buf.toggle_comment();
    assert_eq!(rust_buf.lines[0], "    let x = 42;");

    // Python file
    let mut py_buf = Buffer::new();
    py_buf.path = Some(PathBuf::from("script.py"));
    py_buf.lines = vec!["print('hello')".to_string()];
    py_buf.cursor = Cursor { row: 0, col: 0 };

    py_buf.toggle_comment();
    assert_eq!(py_buf.lines[0], "# print('hello')");

    py_buf.toggle_comment();
    assert_eq!(py_buf.lines[0], "print('hello')");

    // SQL file
    let mut sql_buf = Buffer::new();
    sql_buf.path = Some(PathBuf::from("query.sql"));
    sql_buf.lines = vec!["SELECT * FROM users;".to_string()];
    sql_buf.cursor = Cursor { row: 0, col: 0 };

    sql_buf.toggle_comment();
    assert_eq!(sql_buf.lines[0], "-- SELECT * FROM users;");

    sql_buf.toggle_comment();
    assert_eq!(sql_buf.lines[0], "SELECT * FROM users;");
}

#[test]
fn test_four_layer_completion_engine() {
    let engine = CentralCompletionEngine::new();
    let lines = vec![
        "fn calculate_metrics(buffer_state: &BufferState) -> usize {".to_string(),
        "    let local_variable = 100;".to_string(),
        "    local_variable * 2".to_string(),
        "}".to_string(),
    ];
    let file_path = PathBuf::from("src/metrics.rs");

    // 1. Buffer words query
    let ctx_buf = CompletionContext {
        file_path: Some(&file_path),
        lines: &lines,
        cursor_row: 2,
        cursor_col: 4,
        prefix: "local",
    };
    let items_buf = engine.complete(&ctx_buf, 5);
    assert!(items_buf.iter().any(|c| c.label == "local_variable"));

    // 2. Language Snippets query (Rust fn snippet)
    let ctx_snip = CompletionContext {
        file_path: Some(&file_path),
        lines: &lines,
        cursor_row: 3,
        cursor_col: 2,
        prefix: "fn",
    };
    let items_snip = engine.complete(&ctx_snip, 5);
    assert!(items_snip
        .iter()
        .any(|c| c.kind == termexplorer::completion::provider::CompletionItemKind::Snippet));

    // 3. LSP query (Rust standard functions)
    let ctx_lsp = CompletionContext {
        file_path: Some(&file_path),
        lines: &lines,
        cursor_row: 1,
        cursor_col: 2,
        prefix: "unwrap",
    };
    let items_lsp = engine.complete(&ctx_lsp, 5);
    assert!(items_lsp.iter().any(|c| c.label.starts_with("unwrap")));
}

#[test]
fn test_line_endings_detection_and_preservation() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("crlf_sample.txt");

    std::fs::write(&file_path, "First\r\nSecond\r\nThird\r\n").unwrap();

    let mut buf = Buffer::from_file(file_path.clone()).unwrap();
    assert_eq!(buf.line_ending, LineEnding::Crlf);
    assert_eq!(buf.line_count(), 3);

    // Save should preserve CRLF
    buf.insert_char('!');
    buf.save().unwrap();

    let saved_content = std::fs::read_to_string(&file_path).unwrap();
    assert!(saved_content.contains("\r\n"));
}

#[test]
fn test_crash_recovery_snapshot_lifecycle() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("important_doc.rs");
    std::fs::write(&file_path, "original content").unwrap();

    let mut buf = Buffer::from_file(file_path.clone()).unwrap();
    buf.insert_char('+');
    assert!(buf.dirty);

    // Create recovery snapshot
    buf.create_recovery_snapshot().unwrap();
    assert!(Buffer::has_recovery_snapshot(&file_path));

    // Recover buffer from snapshot
    let recovered = Buffer::recover_from_snapshot(&file_path).unwrap();
    assert_eq!(recovered.lines[0], "+original content");
    assert!(recovered.dirty);

    // Clear snapshot on clean save
    buf.save().unwrap();
    assert!(!Buffer::has_recovery_snapshot(&file_path));
}
