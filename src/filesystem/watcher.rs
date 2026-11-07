//! Filesystem watching using `notify`.

use anyhow::Result;
use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use tokio::sync::mpsc;

/// Spawn a filesystem watcher for `path`.
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
