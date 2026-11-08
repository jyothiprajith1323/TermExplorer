//! Reversible Filesystem Operation Ledger & Safe Undo Management.
//!
//! Tracks filesystem operations (Move, Rename, Copy, Trash) and allows
//! reversing them safely without risking data loss.

use crate::filesystem::operations::{
    delete_entry, move_entry, restore_from_trash, CollisionPolicy,
};
use anyhow::{bail, Context, Result};
use std::path::PathBuf;

/// A reversible filesystem operation entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordedOperation {
    Move {
        from: PathBuf,
        to: PathBuf,
        was_cross_device: bool,
    },
    Rename {
        from: PathBuf,
        to: PathBuf,
    },
    Copy {
        src: PathBuf,
        dest: PathBuf,
    },
    Trash {
        original_path: PathBuf,
        trash_filename: String,
    },
}

impl RecordedOperation {
    pub fn description(&self) -> String {
        match self {
            Self::Move { from, to, .. } => format!(
                "Move: {} → {}",
                from.file_name().and_then(|s| s.to_str()).unwrap_or("entry"),
                to.display()
            ),
            Self::Rename { from, to } => format!(
                "Rename: {} → {}",
                from.file_name().and_then(|s| s.to_str()).unwrap_or("entry"),
                to.file_name().and_then(|s| s.to_str()).unwrap_or("entry")
            ),
            Self::Copy { dest, .. } => format!(
                "Copy: {}",
                dest.file_name().and_then(|s| s.to_str()).unwrap_or("entry")
            ),
            Self::Trash { original_path, .. } => format!(
                "Trash: {}",
                original_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("entry")
            ),
        }
    }
}

/// A transaction ledger tracking filesystem mutations for undo/redo capabilities.
#[derive(Debug)]
pub struct FsOperationLedger {
    history: Vec<RecordedOperation>,
    undone: Vec<RecordedOperation>,
    max_history: usize,
}

impl Default for FsOperationLedger {
    fn default() -> Self {
        Self::new(100)
    }
}

impl FsOperationLedger {
    pub fn new(max_history: usize) -> Self {
        Self {
            history: Vec::new(),
            undone: Vec::new(),
            max_history,
        }
    }

    /// Record a newly executed filesystem mutation.
    pub fn record(&mut self, op: RecordedOperation) {
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(op);
        self.undone.clear();
    }

    /// Check if an operation is available to be undone.
    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    /// Return description of the operation that would be undone.
    pub fn undo_description(&self) -> Option<String> {
        self.history.last().map(|op| op.description())
    }

    /// Revert the most recent operation safely with pre-flight validation.
    pub fn undo(&mut self) -> Result<String> {
        let op = match self.history.pop() {
            Some(o) => o,
            None => bail!("No operations available to undo"),
        };

        let result_msg = match &op {
            RecordedOperation::Move { from, to, .. } => {
                if !to.exists() {
                    bail!(
                        "Cannot undo move: target file no longer exists at {}",
                        to.display()
                    );
                }
                if from.exists() {
                    bail!(
                        "Cannot undo move: original location already occupied at {}",
                        from.display()
                    );
                }
                move_entry(to, from, CollisionPolicy::Fail).with_context(|| {
                    format!("Reverting move {} → {}", to.display(), from.display())
                })?;
                let msg = format!(
                    "Undid move: restored {}",
                    from.file_name().and_then(|s| s.to_str()).unwrap_or("entry")
                );
                msg
            }
            RecordedOperation::Rename { from, to } => {
                if !to.exists() {
                    bail!(
                        "Cannot undo rename: renamed file no longer exists at {}",
                        to.display()
                    );
                }
                if from.exists() {
                    bail!(
                        "Cannot undo rename: original name already occupied at {}",
                        from.display()
                    );
                }
                move_entry(to, from, CollisionPolicy::Fail).with_context(|| {
                    format!("Reverting rename {} → {}", to.display(), from.display())
                })?;
                let msg = format!(
                    "Undid rename: restored {}",
                    from.file_name().and_then(|s| s.to_str()).unwrap_or("entry")
                );
                msg
            }
            RecordedOperation::Copy { dest, .. } => {
                if !dest.exists() {
                    bail!(
                        "Cannot undo copy: created file does not exist at {}",
                        dest.display()
                    );
                }
                delete_entry(dest)
                    .with_context(|| format!("Removing copied entry at {}", dest.display()))?;
                let msg = format!(
                    "Undid copy: removed {}",
                    dest.file_name().and_then(|s| s.to_str()).unwrap_or("entry")
                );
                msg
            }
            RecordedOperation::Trash {
                original_path,
                trash_filename,
            } => {
                restore_from_trash(trash_filename).with_context(|| {
                    format!(
                        "Restoring trashed item {} → {}",
                        trash_filename,
                        original_path.display()
                    )
                })?;
                let msg = format!(
                    "Undid trash: restored {}",
                    original_path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("entry")
                );
                msg
            }
        };

        self.undone.push(op);
        Ok(result_msg)
    }

    /// Return total operations recorded in the ledger.
    pub fn len(&self) -> usize {
        self.history.len()
    }

    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_ledger_move_and_undo() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("source.txt");
        let dst = dir.path().join("dest.txt");
        std::fs::write(&src, "content").unwrap();

        let mut ledger = FsOperationLedger::default();
        move_entry(&src, &dst, CollisionPolicy::Fail).unwrap();
        ledger.record(RecordedOperation::Move {
            from: src.clone(),
            to: dst.clone(),
            was_cross_device: false,
        });

        assert!(dst.exists());
        assert!(!src.exists());
        assert!(ledger.can_undo());

        let res = ledger.undo();
        assert!(res.is_ok());
        assert!(src.exists());
        assert!(!dst.exists());
    }

    #[test]
    fn test_ledger_copy_and_undo() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("orig.txt");
        let dst = dir.path().join("copy.txt");
        std::fs::write(&src, "original data").unwrap();
        std::fs::copy(&src, &dst).unwrap();

        let mut ledger = FsOperationLedger::default();
        ledger.record(RecordedOperation::Copy {
            src: src.clone(),
            dest: dst.clone(),
        });

        assert!(dst.exists());
        let res = ledger.undo();
        assert!(res.is_ok());
        assert!(!dst.exists());
        assert!(src.exists());
    }
}
