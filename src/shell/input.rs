//! Terminal input routing.
//!
//! Translates crossterm key events into the byte sequences that a real
//! terminal would send, so the PTY shell sees standard control codes.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Convert a crossterm KeyEvent into the bytes to send to the PTY.
pub fn key_to_pty_bytes(event: KeyEvent) -> Vec<u8> {
    let ctrl = event.modifiers.contains(KeyModifiers::CONTROL);
    let alt = event.modifiers.contains(KeyModifiers::ALT);

    match event.code {
        // Printable characters.
        KeyCode::Char(c) if ctrl => {
            // Ctrl+A..Z → 0x01..0x1A
            if c.is_ascii_alphabetic() {
                let b = (c.to_ascii_uppercase() as u8) - b'A' + 1;
                vec![b]
            } else if c == '\\' {
                vec![0x1C]
            } else if c == ']' {
                vec![0x1D]
            } else if c == '^' || c == '`' {
                vec![0x1E]
            } else if c == '_' {
                vec![0x1F]
            } else {
                let mut s = String::new();
                s.push(c);
                s.into_bytes()
            }
        }
        KeyCode::Char(c) if alt => {
            // Alt+<char> → ESC <char>
            vec![0x1B, c as u8]
        }
        KeyCode::Char(c) => {
            let mut buf = [0u8; 4];
            c.encode_utf8(&mut buf);
            buf[..c.len_utf8()].to_vec()
        }

        // Special keys.
        KeyCode::Enter => vec![b'\r'],
        KeyCode::Backspace => vec![0x7F],
        KeyCode::Delete => vec![0x1B, b'[', b'3', b'~'],
        KeyCode::Tab => vec![b'\t'],
        KeyCode::BackTab => vec![0x1B, b'[', b'Z'],
        KeyCode::Esc => vec![0x1B],
        KeyCode::Up => vec![0x1B, b'[', b'A'],
        KeyCode::Down => vec![0x1B, b'[', b'B'],
        KeyCode::Right => vec![0x1B, b'[', b'C'],
        KeyCode::Left => vec![0x1B, b'[', b'D'],
        KeyCode::Home => vec![0x1B, b'[', b'H'],
        KeyCode::End => vec![0x1B, b'[', b'F'],
        KeyCode::PageUp => vec![0x1B, b'[', b'5', b'~'],
        KeyCode::PageDown => vec![0x1B, b'[', b'6', b'~'],
        KeyCode::Insert => vec![0x1B, b'[', b'2', b'~'],
        KeyCode::F(n) => f_key_bytes(n),
        KeyCode::Null => vec![0x00],
        _ => vec![],
    }
}

fn f_key_bytes(n: u8) -> Vec<u8> {
    match n {
        1 => vec![0x1B, b'O', b'P'],
        2 => vec![0x1B, b'O', b'Q'],
        3 => vec![0x1B, b'O', b'R'],
        4 => vec![0x1B, b'O', b'S'],
        5 => vec![0x1B, b'[', b'1', b'5', b'~'],
        6 => vec![0x1B, b'[', b'1', b'7', b'~'],
        7 => vec![0x1B, b'[', b'1', b'8', b'~'],
        8 => vec![0x1B, b'[', b'1', b'9', b'~'],
        9 => vec![0x1B, b'[', b'2', b'0', b'~'],
        10 => vec![0x1B, b'[', b'2', b'1', b'~'],
        11 => vec![0x1B, b'[', b'2', b'3', b'~'],
        12 => vec![0x1B, b'[', b'2', b'4', b'~'],
        _ => vec![],
    }
}
