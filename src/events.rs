//! Cross-terminal event handling.
//!
//! Wraps crossterm events and application-internal events into a single stream.

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::time::Duration;
use tokio::sync::mpsc;

/// Events consumed by the application event loop.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// A key was pressed.
    Key(KeyEvent),
    /// A mouse action occurred.
    Mouse(MouseEvent),
    /// The terminal was resized.
    Resize(u16, u16),
    /// Periodic tick for background refresh.
    Tick,
    /// A file-system change was detected.
    FsChange,
    /// System monitor data is ready to refresh.
    MonitorRefresh,
    /// Shell PTY output is available.
    PtyOutput,
    /// A status message should be cleared.
    ClearStatus,
    /// Application should quit.
    Quit,
}

/// Spawns a background task that reads crossterm events and sends them through
/// the channel. Returns the receiver end.
pub fn start_event_loop(tick_rate_ms: u64) -> mpsc::UnboundedReceiver<AppEvent> {
    let (tx, rx) = mpsc::unbounded_channel::<AppEvent>();
    let tick = Duration::from_millis(tick_rate_ms);

    tokio::spawn(async move {
        loop {
            // Poll with timeout — emit Tick if no event arrives in time.
            match tokio::task::spawn_blocking(move || event::poll(tick)).await {
                Ok(Ok(true)) => match tokio::task::spawn_blocking(event::read).await {
                    Ok(Ok(CrosstermEvent::Key(k))) => {
                        let _ = tx.send(AppEvent::Key(k));
                    }
                    Ok(Ok(CrosstermEvent::Mouse(m))) => {
                        let _ = tx.send(AppEvent::Mouse(m));
                    }
                    Ok(Ok(CrosstermEvent::Resize(w, h))) => {
                        let _ = tx.send(AppEvent::Resize(w, h));
                    }
                    Ok(Ok(CrosstermEvent::FocusGained))
                    | Ok(Ok(CrosstermEvent::FocusLost))
                    | Ok(Ok(CrosstermEvent::Paste(_))) => {}
                    Ok(Err(_)) | Err(_) => break,
                },
                Ok(Ok(false)) => {
                    let _ = tx.send(AppEvent::Tick);
                }
                _ => break,
            }
        }
    });

    rx
}
