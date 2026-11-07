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

    /// Feed bytes from the PTY into the scrollback ring.
    pub fn push_output(&mut self, data: &[u8]) {
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
