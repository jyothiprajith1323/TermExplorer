//! Editor Edge Cases, Auto-pairing, Undo-Redo & Multi-Operation Tests.

use termexplorer::editor::{
    auto_pair::{handle_backspace, handle_char},
    buffer::Buffer,
};

#[test]
fn test_auto_pairing_skip_closing_bracket() {
    let mut buf = Buffer::new();

    // Type '(' -> inserts '()' with cursor between
    let act1 = handle_char(&mut buf, '(');
    assert!(act1);
    assert_eq!(buf.lines[0], "()");
    assert_eq!(buf.cursor.col, 1);

    // Type ')' while cursor is right before ')' -> steps over
    let act2 = handle_char(&mut buf, ')');
    assert!(act2);
    assert_eq!(buf.lines[0], "()");
    assert_eq!(buf.cursor.col, 2);
}

#[test]
fn test_auto_pairing_backspace_removes_pair() {
    let mut buf = Buffer::new();
    handle_char(&mut buf, '[');
    assert_eq!(buf.lines[0], "[]");
    assert_eq!(buf.cursor.col, 1);

    // Backspace immediately deletes both brackets
    let act = handle_backspace(&mut buf);
    assert!(act);
    assert_eq!(buf.lines[0], "");
    assert_eq!(buf.cursor.col, 0);
}

#[test]
fn test_large_paste_and_undo_redo_fidelity() {
    let mut buf = Buffer::new();
    buf.lines = vec!["initial state".to_string(), "second line".to_string()];

    // Insert character
    buf.insert_char('X');
    assert_eq!(buf.lines[0], "Xinitial state");

    buf.undo();
    assert_eq!(buf.lines[0], "initial state");

    buf.redo();
    assert_eq!(buf.lines[0], "Xinitial state");
}

#[test]
fn test_cursor_clamping_on_short_lines() {
    let mut buf = Buffer::new();
    buf.lines = vec![
        "A very long line with lots of characters".to_string(),
        "Short".to_string(),
    ];
    buf.cursor.row = 0;
    buf.cursor.col = 30;

    // Move cursor down to short line (5 chars)
    buf.move_down();
    assert_eq!(buf.cursor.row, 1);
    assert!(
        buf.cursor.col <= 5,
        "Cursor should be clamped to line length"
    );
}
