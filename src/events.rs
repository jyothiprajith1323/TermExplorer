//! Cross-terminal event handling.
//!
//! Wraps crossterm events and application-internal events into a single stream.

use crate::preview::FullPreviewData;
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
    /// Async preview data is ready.
    PreviewReady {
        ticket: u64,
        data: Box<FullPreviewData>,
    },
    /// Directory entries chunk loaded asynchronously.
    DirChunkReady {
        ticket: u64,
        chunk: Box<crate::filesystem::chunked::DirChunk>,
    },
    /// Application should quit.
    Quit,
}

/// Spawns a background task that reads crossterm events and sends them through
/// the channel. Returns the sender and receiver ends.
pub fn start_event_loop_with_sender(
    tick_rate_ms: u64,
) -> (
    mpsc::UnboundedSender<AppEvent>,
    mpsc::UnboundedReceiver<AppEvent>,
) {
    let (tx, rx) = mpsc::unbounded_channel::<AppEvent>();
    let tick = Duration::from_millis(tick_rate_ms);
    let loop_tx = tx.clone();

    tokio::spawn(async move {
        loop {
            // Poll with timeout — emit Tick if no event arrives in time.
            match tokio::task::spawn_blocking(move || event::poll(tick)).await {
                Ok(Ok(true)) => match tokio::task::spawn_blocking(event::read).await {
                    Ok(Ok(CrosstermEvent::Key(k))) => {
                        let _ = loop_tx.send(AppEvent::Key(k));
                    }
                    Ok(Ok(CrosstermEvent::Mouse(m))) => {
                        let _ = loop_tx.send(AppEvent::Mouse(m));
                    }
                    Ok(Ok(CrosstermEvent::Resize(w, h))) => {
                        let _ = loop_tx.send(AppEvent::Resize(w, h));
                    }
                    Ok(Ok(CrosstermEvent::FocusGained))
                    | Ok(Ok(CrosstermEvent::FocusLost))
                    | Ok(Ok(CrosstermEvent::Paste(_))) => {}
                    Ok(Err(_)) | Err(_) => break,
                },
                Ok(Ok(false)) => {
                    let _ = loop_tx.send(AppEvent::Tick);
                }
                _ => break,
            }
        }
    });

    (tx, rx)
}

/// Spawns a background task that reads crossterm events and sends them through
/// the channel. Returns the receiver end.
pub fn start_event_loop(tick_rate_ms: u64) -> mpsc::UnboundedReceiver<AppEvent> {
    let (_tx, rx) = start_event_loop_with_sender(tick_rate_ms);
    rx
}
