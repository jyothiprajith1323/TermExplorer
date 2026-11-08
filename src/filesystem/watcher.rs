//! Filesystem watching using `notify` with event debouncing and graceful error handling.

use anyhow::Result;
use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::time::Duration;
use tokio::sync::mpsc;

/// Spawn a debounced filesystem watcher for `path`.
/// Events occurring within `debounce_duration` are coalesced into a single notification.
pub fn watch_directory_debounced(
    path: &Path,
    debounce_duration: Duration,
) -> Result<(RecommendedWatcher, mpsc::UnboundedReceiver<()>)> {
    let (raw_tx, mut raw_rx) = mpsc::unbounded_channel::<notify::Result<notify::Event>>();
    let (out_tx, out_rx) = mpsc::unbounded_channel::<()>();

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let _ = raw_tx.send(res);
        },
        NotifyConfig::default(),
    )?;

    watcher.watch(path, RecursiveMode::NonRecursive)?;

    // Spawn background debouncer task
    tokio::spawn(async move {
        loop {
            // Wait for initial event
            match raw_rx.recv().await {
                Some(_) => {}
                None => break, // Channel closed
            }

            // Debounce loop: keep draining while events keep arriving within debounce_duration
            loop {
                tokio::select! {
                    more = raw_rx.recv() => {
                        match more {
                            Some(_) => continue,
                            None => break,
                        }
                    }
                    _ = tokio::time::sleep(debounce_duration) => {
                        // Quiet period elapsed: emit single coalesced change event
                        let _ = out_tx.send(());
                        break;
                    }
                }
            }
        }
    });

    Ok((watcher, out_rx))
}

/// Spawn a raw filesystem watcher for `path`.
/// Events are sent through the returned receiver.
pub fn watch_directory(
    path: &Path,
) -> Result<(
    RecommendedWatcher,
    mpsc::UnboundedReceiver<notify::Result<notify::Event>>,
)> {
    let (tx, rx) = mpsc::unbounded_channel();

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let _ = tx.send(res);
        },
        NotifyConfig::default(),
    )?;

    watcher.watch(path, RecursiveMode::NonRecursive)?;
    Ok((watcher, rx))
}
