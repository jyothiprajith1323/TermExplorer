//! Terminal session management — wraps PtySession and maintains a scrollback buffer.

use crate::shell::pty::PtySession;
use anyhow::Result;
use std::collections::VecDeque;
use std::path::PathBuf;
use tokio::sync::mpsc;

#[allow(dead_code)]
pub struct TerminalSession {
    pub pty: PtySession,
    /// Raw scrollback bytes (unparsed ANSI).
    pub scrollback: VecDeque<u8>,
    /// Current PTY dimensions.
    pub cols: u16,
    pub rows: u16,
    pub working_dir: PathBuf,
    /// How far from the bottom the view is scrolled.
    pub scroll_offset: usize,
}

impl TerminalSession {
    pub fn new(
        working_dir: PathBuf,
        cols: u16,
        rows: u16,
        tx: mpsc::UnboundedSender<Vec<u8>>,
    ) -> Result<Self> {
        let pty = PtySession::spawn(&working_dir, cols, rows, tx)?;
        Ok(Self {
            pty,
            scrollback: VecDeque::new(),
            cols,
            rows,
            working_dir,
            scroll_offset: 0,
        })
    }

    /// Feed bytes from the PTY into the scrollback ring and detect OSC 7 working directory updates.
    pub fn push_output(&mut self, data: &[u8]) {
        if let Some(new_dir) = parse_osc7_path(data) {
            self.working_dir = new_dir;
        }

        for &b in data {
            self.scrollback.push_back(b);
        }
        // Trim by bytes rather than lines for simplicity; limit to ~1 MB.
        const MAX_BYTES: usize = 1024 * 1024;
        while self.scrollback.len() > MAX_BYTES {
            self.scrollback.pop_front();
        }
    }

    /// All scrollback as a contiguous byte slice (allocates once).
    #[allow(dead_code)]
    pub fn scrollback_bytes(&self) -> Vec<u8> {
        self.scrollback.iter().copied().collect()
    }

    /// Send input to the shell.
    pub fn send_input(&self, data: &[u8]) -> Result<()> {
        self.pty.write_input(data)
    }

    /// Resize the PTY and update stored dimensions.
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        self.cols = cols;
        self.rows = rows;
        self.pty.resize(cols, rows)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.pty.child.kill();
        let _ = self.pty.child.wait();
    }
}

/// Parse OSC 7 directory notifications (`\x1b]7;file://hostname/path\x07` or `\x1b]7;file://hostname/path\x1b\`).
pub fn parse_osc7_path(data: &[u8]) -> Option<PathBuf> {
    let pattern = b"\x1b]7;file://";
    let pos = data.windows(pattern.len()).position(|w| w == pattern)?;
    let start = pos + pattern.len();
    let remaining = &data[start..];

    // Find the next '/' which separates hostname from path
    let path_offset = remaining.iter().position(|&b| b == b'/')?;
    let path_bytes = &remaining[path_offset..];

    // Find terminator: \x07 (BEL) or \x1b (ESC)
    let end_offset = path_bytes
        .iter()
        .position(|&b| b == 0x07 || b == 0x1b)
        .unwrap_or(path_bytes.len());
    let raw_path = &path_bytes[..end_offset];

    if let Ok(path_str) = std::str::from_utf8(raw_path) {
        let decoded = percent_decode(path_str);
        Some(PathBuf::from(decoded))
    } else {
        None
    }
}

fn percent_decode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(val) = u8::from_str_radix(&input[i + 1..i + 3], 16) {
                result.push(val as char);
                i += 3;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_osc7_bel_terminated() {
        let seq = b"\x1b]7;file://myhost/home/user/my%20projects\x07";
        let parsed = parse_osc7_path(seq);
        assert_eq!(parsed, Some(PathBuf::from("/home/user/my projects")));
    }

    #[test]
    fn test_parse_osc7_st_terminated() {
        let seq = b"\x1b]7;file://localhost/var/log\x1b\\";
        let parsed = parse_osc7_path(seq);
        assert_eq!(parsed, Some(PathBuf::from("/var/log")));
    }
}
