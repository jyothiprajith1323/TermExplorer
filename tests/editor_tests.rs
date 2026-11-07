use tempfile::tempdir;
use termexplorer::editor::{
    auto_pair::{handle_backspace, handle_char},
    buffer::{Buffer, Cursor},
    completion::{prefix_at, word_completions},
};

#[test]
fn test_buffer_creation_and_editing() {
    let mut buf = Buffer::new();
    assert_eq!(buf.line_count(), 1);
    assert_eq!(buf.cursor, Cursor { row: 0, col: 0 });
    assert!(!buf.dirty);

    // Insert characters
    buf.insert_char('h');
    buf.insert_char('e');
    buf.insert_char('l');
    buf.insert_char('l');
    buf.insert_char('o');
    assert_eq!(buf.lines[0], "hello");
    assert_eq!(buf.cursor.col, 5);
    assert!(buf.dirty);

    // Newline with auto-indent
    buf.insert_newline();
    assert_eq!(buf.line_count(), 2);
    assert_eq!(buf.cursor.row, 1);
    assert_eq!(buf.cursor.col, 0);

    // Tab insertion
    buf.insert_tab(4);
    assert_eq!(buf.lines[1], "    ");
    assert_eq!(buf.cursor.col, 4);

    // Backspace
    buf.backspace();
    assert_eq!(buf.lines[1], "   ");
    assert_eq!(buf.cursor.col, 3);
}

#[test]
fn test_buffer_undo_redo() {
    let mut buf = Buffer::new();
    buf.insert_char('a');
    buf.insert_char('b');
    assert_eq!(buf.lines[0], "ab");

    buf.undo();
    assert_eq!(buf.lines[0], "a");

    buf.undo();
    assert_eq!(buf.lines[0], "");

    buf.redo();
    assert_eq!(buf.lines[0], "a");

    buf.redo();
    assert_eq!(buf.lines[0], "ab");
}

#[test]
fn test_buffer_navigation() {
    let mut buf = Buffer::new();
    buf.lines = vec![
        "First line".to_string(),
        "Second line with text".to_string(),
        "Third line".to_string(),
    ];
    buf.cursor = Cursor { row: 0, col: 0 };

    buf.move_down();
    assert_eq!(buf.cursor.row, 1);

    buf.move_end();
    assert_eq!(buf.cursor.col, "Second line with text".len());

    buf.move_home();
    assert_eq!(buf.cursor.col, 0);

    buf.goto_line(3);
    assert_eq!(buf.cursor.row, 2);

    buf.move_up();
    assert_eq!(buf.cursor.row, 1);
}

#[test]
fn test_buffer_clipboard() {
    let mut buf = Buffer::new();
    buf.lines = vec!["Hello World".to_string(), "Second Line".to_string()];
    buf.cursor = Cursor { row: 0, col: 0 };

    buf.copy_line();
    buf.cursor.row = 1;
    buf.cursor.col = 11;
    buf.insert_newline();
    buf.paste();

    assert_eq!(buf.line_count(), 3);
    assert_eq!(buf.lines[2], "Hello World");
}

#[test]
fn test_auto_pairing_all_brackets_and_quotes() {
    let pairs = [
        ('(', ')'),
        ('[', ']'),
        ('{', '}'),
        ('"', '"'),
        ('\'', '\''),
        ('`', '`'),
    ];

    for (open, close) in pairs {
        let mut buf = Buffer::new();
        let handled = handle_char(&mut buf, open);
        assert!(handled, "Failed to handle open pair for {}", open);
        assert_eq!(
            buf.lines[0],
            format!("{}{}", open, close),
            "Pair not inserted for {}",
            open
        );
        // Cursor should be placed between the pair
        assert_eq!(buf.cursor.col, 1);

        // Typing closing char skips over it
        let skipped = handle_char(&mut buf, close);
        assert!(skipped, "Failed to skip over close pair for {}", close);
        assert_eq!(buf.cursor.col, 2);
        assert_eq!(buf.lines[0], format!("{}{}", open, close));

        // Backspace between pair deletes both
        let mut del_buf = Buffer::new();
        handle_char(&mut del_buf, open);
        assert_eq!(del_buf.cursor.col, 1);
        let bs_handled = handle_backspace(&mut del_buf);
        assert!(bs_handled, "Failed to backspace delete pair for {}", open);
        assert_eq!(del_buf.lines[0], "");
        assert_eq!(del_buf.cursor.col, 0);
    }
}

#[test]
fn test_nested_auto_pairing() {
    let mut buf = Buffer::new();
    // type '(' -> ()
    handle_char(&mut buf, '(');
    assert_eq!(buf.lines[0], "()");
    assert_eq!(buf.cursor.col, 1);

    // type '[' inside -> ([])
    handle_char(&mut buf, '[');
    assert_eq!(buf.lines[0], "([])");
    assert_eq!(buf.cursor.col, 2);

    // type '{' inside -> ([{}])
    handle_char(&mut buf, '{');
    assert_eq!(buf.lines[0], "([{}])");
    assert_eq!(buf.cursor.col, 3);
}

#[test]
fn test_word_completions_and_prefix() {
    let lines = vec![
        "let file_explorer = FileExplorer::new();".to_string(),
        "file_explorer.open_file();".to_string(),
        "let file_manager = true;".to_string(),
    ];

    assert_eq!(prefix_at("let file_exp", 12), "file_exp");
    assert_eq!(prefix_at("foo(bar_baz", 11), "bar_baz");

    let completions = word_completions(&lines, "file_", 5);
    let labels: Vec<String> = completions.into_iter().map(|c| c.label).collect();

    assert!(labels.contains(&"file_explorer".to_string()));
    assert!(labels.contains(&"file_manager".to_string()));
}

#[test]
fn test_buffer_completion_flow() {
    let mut buf = Buffer::new();
    buf.lines = vec![
        "fn calculate_checksum() {}".to_string(),
        "fn calculate_distance() {}".to_string(),
        "calc".to_string(),
    ];
    buf.cursor = Cursor { row: 2, col: 4 };

    buf.update_completion();
    assert!(buf.completion.active);
    assert_eq!(buf.completion.candidates.len(), 2);

    // Accept completion
    let accepted = buf.accept_completion();
    assert!(accepted);
    assert!(!buf.completion.active);
    assert!(buf.lines[2].starts_with("calculate_"));
}

#[test]
fn test_save_buffer_to_file() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("saved.txt");

    let mut buf = Buffer::new();
    buf.insert_char('T');
    buf.insert_char('e');
    buf.insert_char('s');
    buf.insert_char('t');

    buf.save_as(file.clone()).unwrap();
    assert!(!buf.dirty);
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "Test");
}
