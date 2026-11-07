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
