//! Multi-session terminal manager for TermExplorer workstation.
//!
//! Coordinates multiple concurrent terminal shells, session tabs, isolated PTY processes,
//! and routing of multiplexed background I/O channels.

use crate::shell::session::TerminalSession;
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use tokio::sync::mpsc;

/// Manages multiple terminal sessions and routes multiplexed PTY outputs.
pub struct TerminalSessionManager {
    sessions: Vec<TerminalSession>,
    active_idx: usize,
    next_id: usize,
    max_sessions: usize,
    tx: mpsc::UnboundedSender<(usize, Vec<u8>)>,
}

impl TerminalSessionManager {
    /// Create a new session manager with bounded maximum concurrent sessions.
    pub fn new(max_sessions: usize) -> (Self, mpsc::UnboundedReceiver<(usize, Vec<u8>)>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (
            Self {
                sessions: Vec::new(),
                active_idx: 0,
                next_id: 1,
                max_sessions: max_sessions.max(1),
                tx,
            },
            rx,
        )
    }

    /// Total count of active sessions.
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// True if no terminal sessions are active.
    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    /// Index of current focused session (0-based).
    pub fn active_index(&self) -> usize {
        self.active_idx
    }

    /// Reference to currently active terminal session.
    pub fn active_session(&self) -> Option<&TerminalSession> {
        self.sessions.get(self.active_idx)
    }

    /// Mutable reference to currently active terminal session.
    pub fn active_session_mut(&mut self) -> Option<&mut TerminalSession> {
        self.sessions.get_mut(self.active_idx)
    }

    /// Reference to session by index.
    pub fn get_session(&self, idx: usize) -> Option<&TerminalSession> {
        self.sessions.get(idx)
    }

    /// Mutable reference to session by index.
    pub fn get_session_mut(&mut self, idx: usize) -> Option<&mut TerminalSession> {
        self.sessions.get_mut(idx)
    }

    /// Slice of all managed sessions.
    pub fn sessions(&self) -> &[TerminalSession] {
        &self.sessions
    }

    /// Spawn a new terminal session in `working_dir` with given dimensions.
    pub fn create_session(&mut self, working_dir: PathBuf, cols: u16, rows: u16) -> Result<usize> {
        if self.sessions.len() >= self.max_sessions {
            return Err(anyhow!(
                "Maximum terminal sessions reached ({})",
                self.max_sessions
            ));
        }

        let id = self.next_id;
        self.next_id += 1;

        let session = TerminalSession::new_tagged(id, working_dir, cols, rows, self.tx.clone())?;
        self.sessions.push(session);
        self.active_idx = self.sessions.len() - 1;
        Ok(id)
    }

    /// Switch active session by index.
    pub fn switch_session(&mut self, idx: usize) -> bool {
        if idx < self.sessions.len() {
            self.active_idx = idx;
            true
        } else {
            false
        }
    }

    /// Cycle to next session.
    pub fn next_session(&mut self) {
        if !self.sessions.is_empty() {
            self.active_idx = (self.active_idx + 1) % self.sessions.len();
        }
    }

    /// Cycle to previous session.
    pub fn prev_session(&mut self) {
        if !self.sessions.is_empty() {
            if self.active_idx == 0 {
                self.active_idx = self.sessions.len() - 1;
            } else {
                self.active_idx -= 1;
            }
        }
    }

    /// Close session at `idx`. If closing active session, selects previous available session.
    pub fn close_session(&mut self, idx: usize) -> bool {
        if idx < self.sessions.len() {
            self.sessions.remove(idx);
            if self.sessions.is_empty() {
                self.active_idx = 0;
            } else if self.active_idx >= self.sessions.len() {
                self.active_idx = self.sessions.len() - 1;
            }
            true
        } else {
            false
        }
    }

    /// Route PTY output bytes directly to the session matching `session_id`.
    pub fn push_output(&mut self, session_id: usize, data: &[u8]) {
        for sess in &mut self.sessions {
            if sess.id == session_id {
                sess.push_output(data);
                break;
            }
        }
    }

    /// Resize all terminal sessions (e.g. on window resize).
    pub fn resize_all(&mut self, cols: u16, rows: u16) {
        for sess in &mut self.sessions {
            let _ = sess.resize(cols, rows);
        }
    }

    /// Restart the active terminal session if it exited.
    pub fn restart_active(&mut self) -> Result<()> {
        let active = self.active_idx;
        if let Some(sess) = self.sessions.get_mut(active) {
            let id = sess.id;
            let dir = sess.working_dir.clone();
            let cols = sess.cols;
            let rows = sess.rows;
            let new_sess = TerminalSession::new_tagged(id, dir, cols, rows, self.tx.clone())?;
            *sess = new_sess;
            Ok(())
        } else {
            Err(anyhow!("No active session to restart"))
        }
    }

    /// Send user input or command string to a specific session by ID.
    pub fn send_input_to_session(&self, session_id: usize, data: &[u8]) -> Result<()> {
        if let Some(session) = self.sessions.iter().find(|s| s.id == session_id) {
            session.send_input(data)?;
        }
        Ok(())
    }

    /// Clean up and kill all running child sessions.
    pub fn cleanup_all(&mut self) {
        self.sessions.clear();
        self.active_idx = 0;
    }
}
