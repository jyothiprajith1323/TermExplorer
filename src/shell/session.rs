//! Terminal session management — wraps PtySession, TerminalScreen, and process lifecycle.

use crate::shell::pty::PtySession;
use crate::shell::terminal_screen::TerminalScreen;
use anyhow::Result;
use std::collections::VecDeque;
use std::path::PathBuf;
use tokio::sync::mpsc;

pub struct TerminalSession {
    pub id: usize,
    pub title: String,
    pub pty: PtySession,
    pub screen: TerminalScreen,
    /// Raw scrollback bytes (unparsed ANSI) for backward compatibility.
    pub scrollback: VecDeque<u8>,
    /// Current PTY dimensions.
    pub cols: u16,
    pub rows: u16,
    pub working_dir: PathBuf,
    pub is_alive: bool,
}

impl TerminalSession {
    /// Create a new session with default ID 1 (legacy single-channel interface).
    pub fn new(
        working_dir: PathBuf,
        cols: u16,
        rows: u16,
        tx: mpsc::UnboundedSender<Vec<u8>>,
    ) -> Result<Self> {
        let pty = PtySession::spawn(&working_dir, cols, rows, tx)?;
        let screen = TerminalScreen::new(rows, cols, 10_000);
        let shell_name = detect_shell_name();
        let title = format!("1: {}", shell_name);

        Ok(Self {
            id: 1,
            title,
            pty,
            screen,
            scrollback: VecDeque::new(),
            cols,
            rows,
            working_dir,
            is_alive: true,
        })
    }

    /// Create a tagged session for multi-session management.
    pub fn new_tagged(
        id: usize,
        working_dir: PathBuf,
        cols: u16,
        rows: u16,
        tx: mpsc::UnboundedSender<(usize, Vec<u8>)>,
    ) -> Result<Self> {
        let pty = PtySession::spawn_tagged(&working_dir, cols, rows, id, tx)?;
        let screen = TerminalScreen::new(rows, cols, 10_000);
        let shell_name = detect_shell_name();
        let title = format!("{}: {}", id, shell_name);

        Ok(Self {
            id,
            title,
            pty,
            screen,
            scrollback: VecDeque::new(),
            cols,
            rows,
            working_dir,
            is_alive: true,
        })
    }

    /// Feed bytes from the PTY into the virtual terminal screen, raw scrollback, and parse OSC 7 directory updates.
    pub fn push_output(&mut self, data: &[u8]) {
        if let Some(new_dir) = parse_osc7_path(data) {
            self.working_dir = new_dir;
        }

        // Process through vt100 virtual screen emulator
        self.screen.process(data);

        // Keep raw bytes buffer bounded to 1 MB for compatibility
        for &b in data {
            self.scrollback.push_back(b);
        }
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

    /// Send input to the shell stdin.
    pub fn send_input(&self, data: &[u8]) -> Result<()> {
        self.pty.write_input(data)
    }

    /// Resize the PTY and the virtual terminal screen emulator.
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        self.cols = cols;
        self.rows = rows;
        self.screen.resize(rows, cols);
        self.pty.resize(cols, rows)
    }

    /// Check if child process is still running.
    pub fn check_alive(&mut self) -> bool {
        self.is_alive = self.pty.is_alive();
        self.is_alive
    }

    /// Check if shell process has exited.
    pub fn is_exited(&self) -> bool {
        !self.is_alive
    }

    /// Get shell base name for this session.
    pub fn shell_name(&self) -> &str {
        if let Some((_, name)) = self.title.split_once(": ") {
            name
        } else {
            &self.title
        }
    }

    /// Scroll up in terminal history.
    pub fn scroll_up(&mut self, delta: usize) {
        self.screen.scroll_up(delta);
    }

    /// Scroll down in terminal history.
    pub fn scroll_down(&mut self, delta: usize) {
        self.screen.scroll_down(delta);
    }

    /// Reset scroll offset to live screen.
    pub fn reset_scroll(&mut self) {
        self.screen.reset_scroll();
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.pty.child.kill();
        let _ = self.pty.child.wait();
    }
}

/// Detect the user's shell base name (e.g. "bash", "zsh", "fish").
pub fn detect_shell_name() -> String {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string());
    std::path::Path::new(&shell)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "bash".to_string())
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
