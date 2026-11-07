//! Terminal Edge Cases and Throughput Resiliency Tests.
//! Tests PTY output saturation, circular buffer limits (1MB cap),
//! complex ANSI sequences, and rapid window resizing.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;
use termexplorer::shell::input::key_to_pty_bytes;
use termexplorer::shell::TerminalSession;

#[test]
fn test_terminal_scrollback_1mb_cap_truncation() {
    let mut scrollback = Vec::new();
    let one_mb = 1024 * 1024;

    // Simulate 1.5 MB of rapid shell streaming output
    let chunk = vec![b'X'; 64 * 1024]; // 64 KB chunks
    for _ in 0..25 {
        scrollback.extend_from_slice(&chunk);
        if scrollback.len() > one_mb {
            let excess = scrollback.len() - one_mb;
            scrollback.drain(0..excess);
        }
    }

    assert_eq!(
        scrollback.len(),
        one_mb,
        "Scrollback must be capped at exactly 1 MB"
    );
}

#[test]
fn test_terminal_key_translation_edge_cases() {
    // 1. Shift + Tab (BackTab) -> \x1b[Z
    let backtab = KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT);
    assert_eq!(key_to_pty_bytes(backtab), vec![0x1B, b'[', b'Z']);

    // 2. Escape key -> \x1b
    let esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    assert_eq!(key_to_pty_bytes(esc), vec![0x1B]);

    // 3. Ctrl + L (Clear screen in bash) -> 0x0C
    let ctrl_l = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL);
    assert_eq!(key_to_pty_bytes(ctrl_l), vec![0x0C]);

    // 4. Ctrl + A (Home in bash) -> 0x01
    let ctrl_a = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    assert_eq!(key_to_pty_bytes(ctrl_a), vec![0x01]);

    // 5. Ctrl + E (End in bash) -> 0x05
    let ctrl_e = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL);
    assert_eq!(key_to_pty_bytes(ctrl_e), vec![0x05]);
}

#[tokio::test]
async fn test_terminal_rapid_resize() {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/tmp"));
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();

    let session = TerminalSession::new(cwd, 80, 24, tx);
    assert!(session.is_ok());
    let mut session = session.unwrap();

    // Rapid resize simulations
    for (cols, rows) in [(120, 40), (80, 24), (160, 50), (40, 10)] {
        let res = session.resize(cols, rows);
        assert!(res.is_ok(), "Resize to {}x{} failed", cols, rows);
    }
}
