use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;
use termexplorer::shell::{input::key_to_pty_bytes, TerminalSession};

#[test]
fn test_key_to_pty_bytes_translation() {
    // Normal character
    let a_key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
    assert_eq!(key_to_pty_bytes(a_key), vec![b'a']);

    // Enter -> \r
    let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(key_to_pty_bytes(enter), vec![b'\r']);

    // Backspace -> 0x7F
    let bs = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
    assert_eq!(key_to_pty_bytes(bs), vec![0x7F]);

    // Ctrl+C -> 0x03
    let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert_eq!(key_to_pty_bytes(ctrl_c), vec![0x03]);

    // Ctrl+D -> 0x04
    let ctrl_d = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
    assert_eq!(key_to_pty_bytes(ctrl_d), vec![0x04]);

    // Arrow keys
    let up = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
    assert_eq!(key_to_pty_bytes(up), vec![0x1B, b'[', b'A']);

    let down = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
    assert_eq!(key_to_pty_bytes(down), vec![0x1B, b'[', b'B']);
}

#[tokio::test]
async fn test_terminal_session_lifecycle() {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/tmp"));
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    let session = TerminalSession::new(cwd, 80, 24, tx);
    assert!(session.is_ok(), "Failed to create PTY session");
    let session = session.unwrap();

    // Send command to shell
    session.send_input(b"echo 'TermExplorer_PTY_OK'\r").unwrap();

    // Read output from PTY
    let mut collected = Vec::new();
    let timeout = tokio::time::Duration::from_secs(3);
    let start = tokio::time::Instant::now();

    while start.elapsed() < timeout {
        if let Ok(Some(bytes)) =
            tokio::time::timeout(tokio::time::Duration::from_millis(200), rx.recv()).await
        {
            collected.extend_from_slice(&bytes);
            let s = String::from_utf8_lossy(&collected);
            if s.contains("TermExplorer_PTY_OK") {
                break;
            }
        }
    }

    let output_str = String::from_utf8_lossy(&collected);
    assert!(
        output_str.contains("TermExplorer_PTY_OK"),
        "PTY output did not contain expected echoed string: {}",
        output_str
    );
}

#[test]
fn test_terminal_screen_ansi_parsing_and_colors() {
    use ratatui::style::{Color, Modifier};
    use termexplorer::shell::TerminalScreen;

    let mut screen = TerminalScreen::new(24, 80, 1000);

    // Truecolor RGB: \x1b[38;2;255;100;50m
    // Bold & Underline: \x1b[1m\x1b[4m
    let text = b"\x1b[38;2;255;100;50m\x1b[1m\x1b[4mHello Truecolor\x1b[0m\r\n";
    screen.process(text);

    let row = screen.render_row(0, Color::White, Color::Black);
    assert!(!row.spans.is_empty());

    let span = &row.spans[0];
    assert!(span.content.starts_with("Hello Truecolor"));
    assert_eq!(span.style.fg, Some(Color::Rgb(255, 100, 50)));
    assert!(span.style.add_modifier.contains(Modifier::BOLD));
    assert!(span.style.add_modifier.contains(Modifier::UNDERLINED));
}

#[test]
fn test_terminal_screen_alternate_screen_and_cursor() {
    use termexplorer::shell::TerminalScreen;

    let mut screen = TerminalScreen::new(24, 80, 1000);

    // Initial state: normal screen, visible cursor at (0, 0)
    assert!(!screen.is_alternate_screen());
    assert!(!screen.hide_cursor());
    assert_eq!(screen.cursor_position(), (0, 0));

    // Move cursor to row 10, col 20 (1-based in ANSI: \x1b[11;21H)
    screen.process(b"\x1b[11;21H");
    assert_eq!(screen.cursor_position(), (10, 20));

    // Hide cursor: \x1b[?25l
    screen.process(b"\x1b[?25l");
    assert!(screen.hide_cursor());

    // Switch to alternate screen: \x1b[?1049h
    screen.process(b"\x1b[?1049h");
    assert!(screen.is_alternate_screen());

    // Switch back: \x1b[?1049l
    screen.process(b"\x1b[?1049l");
    assert!(!screen.is_alternate_screen());
}

#[test]
fn test_terminal_screen_bounded_scrollback_and_search() {
    use termexplorer::shell::TerminalScreen;

    let mut screen = TerminalScreen::new(10, 80, 100);

    // Write 200 lines to overflow the 100 scrollback limit
    for i in 0..200 {
        let line = format!("Log entry index {}\r\n", i);
        screen.process(line.as_bytes());
    }

    // Scrollback should be bounded
    assert!(screen.scrollback_len() <= 100);

    // Search buffer for matches
    let results = screen.search("Log entry index 19");
    assert!(!results.is_empty());
    assert!(results[0].contains("Log entry index 19"));

    // Scroll navigation
    assert_eq!(screen.scroll_offset(), 0);
    screen.scroll_up(10);
    assert_eq!(screen.scroll_offset(), 10);
    screen.scroll_down(5);
    assert_eq!(screen.scroll_offset(), 5);
    screen.reset_scroll();
    assert_eq!(screen.scroll_offset(), 0);
}

#[tokio::test]
async fn test_terminal_session_manager_tabs_and_multiplexing() {
    use termexplorer::shell::manager::TerminalSessionManager;

    let (mut manager, _rx) = TerminalSessionManager::new(4);
    let tmp = std::env::temp_dir();

    // Create session 1
    let id1 = manager.create_session(tmp.clone(), 80, 24).unwrap();
    assert_eq!(id1, 1);
    assert_eq!(manager.session_count(), 1);
    assert_eq!(manager.active_index(), 0);

    // Create session 2
    let id2 = manager.create_session(tmp.clone(), 80, 24).unwrap();
    assert_eq!(id2, 2);
    assert_eq!(manager.session_count(), 2);
    assert_eq!(manager.active_index(), 1);

    // Switch session
    assert!(manager.switch_session(0));
    assert_eq!(manager.active_index(), 0);

    // Cycle tabs
    manager.next_session();
    assert_eq!(manager.active_index(), 1);
    manager.prev_session();
    assert_eq!(manager.active_index(), 0);

    // Route multiplexed output to session 2
    manager.push_output(2, b"hello from session 2\r\n");
    let sess2 = manager.get_session(1).unwrap();
    let matches = sess2.screen.search("hello from session 2");
    assert!(!matches.is_empty());

    // Close session
    assert!(manager.close_session(0));
    assert_eq!(manager.session_count(), 1);

    // Cleanup all
    manager.cleanup_all();
    assert!(manager.is_empty());
}
