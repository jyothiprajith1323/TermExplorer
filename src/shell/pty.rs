//! PTY management — spawn a real pseudo-terminal with the user's shell.

use anyhow::{Context, Result};
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// A live PTY session.
pub struct PtySession {
    pub master: Box<dyn MasterPty + Send>,
    pub child: Box<dyn Child + Send + Sync>,
    pub writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl PtySession {
    /// Spawn the user's shell in a PTY, starting in `working_dir`.
    pub fn spawn(
        working_dir: &Path,
        cols: u16,
        rows: u16,
        tx: mpsc::UnboundedSender<Vec<u8>>,
    ) -> Result<Self> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        // Verify the shell exists, fall back to bash.
        let shell = if std::path::Path::new(&shell).exists() {
            shell
        } else {
            "/bin/bash".to_string()
        };

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Opening PTY pair")?;

        let mut cmd = CommandBuilder::new(&shell);
        cmd.cwd(working_dir);

        // Inherit the full environment.
        cmd.env("TERM", "xterm-256color");

        let child = pair.slave.spawn_command(cmd).context("Spawning shell")?;

        let mut reader = pair
            .master
            .try_clone_reader()
            .context("Cloning PTY reader")?;
        let writer = pair.master.take_writer().context("Taking PTY writer")?;

        std::thread::Builder::new()
            .name("termexplorer-pty-reader".to_string())
            .spawn(move || {
                let mut buf = [0u8; 4096];
                loop {
                    match reader.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            if tx.send(buf[..n].to_vec()).is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            })
            .context("Spawning PTY reader thread")?;

        Ok(Self {
            master: pair.master,
            child,
            writer: Arc::new(Mutex::new(writer)),
        })
    }

    /// Write bytes to the shell's stdin.
    pub fn write_input(&self, data: &[u8]) -> Result<()> {
        let mut w = self.writer.lock().unwrap();
        w.write_all(data).context("Writing to PTY")?;
        w.flush().context("Flushing PTY writer")?;
        Ok(())
    }

    /// Resize the PTY.
    pub fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        self.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Resizing PTY")
    }

    /// Return true if the child process has exited.
    #[allow(dead_code)]
    pub fn is_alive(&mut self) -> bool {
        self.child.try_wait().ok().flatten().is_none()
    }
}
