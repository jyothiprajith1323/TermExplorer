//! Workspace Trust Security Boundary.
//!
//! Enforces that tasks defined inside project repositories or manifests
//! cannot execute on the host machine without explicit user confirmation.
//! Trust decisions are stored in the user's XDG configuration directory and
//! are scoped strictly to canonical workspace root paths.

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Persistent record of trusted workspace roots.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TrustRecord {
    version: u32,
    trusted_roots: Vec<PathBuf>,
}

/// Authoritative manager for workspace trust decisions.
#[derive(Debug, Clone)]
pub struct WorkspaceTrustStore {
    trusted_paths: HashSet<PathBuf>,
    store_file: PathBuf,
}

impl Default for WorkspaceTrustStore {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceTrustStore {
    /// Initializes the trust store from the standard XDG configuration location:
    /// `~/.config/termexplorer/trusted_workspaces.json`.
    pub fn new() -> Self {
        let store_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("termexplorer");

        let store_file = store_dir.join("trusted_workspaces.json");
        let mut store = Self {
            trusted_paths: HashSet::new(),
            store_file,
        };

        store.load();
        store
    }

    /// Creates an in-memory or custom path trust store (primarily for tests).
    pub fn with_custom_file(store_file: PathBuf) -> Self {
        let mut store = Self {
            trusted_paths: HashSet::new(),
            store_file,
        };
        store.load();
        store
    }

    /// Checks if the given workspace directory is explicitly trusted.
    pub fn is_trusted(&self, workspace_root: &Path) -> bool {
        let canonical = workspace_root
            .canonicalize()
            .unwrap_or_else(|_| workspace_root.to_path_buf());

        self.trusted_paths.contains(&canonical)
    }

    /// Grants trust to a workspace root and persists the decision.
    pub fn trust_workspace(&mut self, workspace_root: &Path) -> io::Result<()> {
        let canonical = workspace_root
            .canonicalize()
            .unwrap_or_else(|_| workspace_root.to_path_buf());

        self.trusted_paths.insert(canonical);
        self.save()
    }

    /// Revokes trust from a workspace root and persists the decision.
    pub fn revoke_trust(&mut self, workspace_root: &Path) -> io::Result<()> {
        let canonical = workspace_root
            .canonicalize()
            .unwrap_or_else(|_| workspace_root.to_path_buf());

        self.trusted_paths.remove(&canonical);
        self.save()
    }

    /// Returns a list of all currently trusted workspace roots.
    pub fn list_trusted(&self) -> Vec<PathBuf> {
        self.trusted_paths.iter().cloned().collect()
    }

    /// Loads the trust database from disk.
    fn load(&mut self) {
        if !self.store_file.exists() {
            return;
        }

        if let Ok(content) = fs::read_to_string(&self.store_file) {
            if let Ok(record) = serde_json::from_str::<TrustRecord>(&content) {
                for root in record.trusted_roots {
                    self.trusted_paths.insert(root);
                }
            }
        }
    }

    /// Atomically persists the trust database to disk.
    fn save(&self) -> io::Result<()> {
        if let Some(parent) = self.store_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let record = TrustRecord {
            version: 1,
            trusted_roots: self.trusted_paths.iter().cloned().collect(),
        };

        let json = serde_json::to_string_pretty(&record).map_err(io::Error::other)?;

        let temp_path = self.store_file.with_extension("tmp");
        fs::write(&temp_path, json)?;
        fs::rename(temp_path, &self.store_file)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_untrusted_by_default() {
        let dir = tempdir().unwrap();
        let store_file = dir.path().join("trusted.json");
        let store = WorkspaceTrustStore::with_custom_file(store_file);

        let test_repo = dir.path().join("untrusted_project");
        fs::create_dir_all(&test_repo).unwrap();

        assert!(!store.is_trusted(&test_repo));
    }

    #[test]
    fn test_trust_grant_and_persistence() {
        let dir = tempdir().unwrap();
        let store_file = dir.path().join("trusted.json");
        let mut store = WorkspaceTrustStore::with_custom_file(store_file.clone());

        let test_repo = dir.path().join("my_rust_project");
        fs::create_dir_all(&test_repo).unwrap();

        store.trust_workspace(&test_repo).unwrap();
        assert!(store.is_trusted(&test_repo));

        // Re-read from disk to test persistence
        let reloaded_store = WorkspaceTrustStore::with_custom_file(store_file);
        assert!(reloaded_store.is_trusted(&test_repo));
    }

    #[test]
    fn test_trust_revocation() {
        let dir = tempdir().unwrap();
        let store_file = dir.path().join("trusted.json");
        let mut store = WorkspaceTrustStore::with_custom_file(store_file.clone());

        let test_repo = dir.path().join("temporary_project");
        fs::create_dir_all(&test_repo).unwrap();

        store.trust_workspace(&test_repo).unwrap();
        assert!(store.is_trusted(&test_repo));

        store.revoke_trust(&test_repo).unwrap();
        assert!(!store.is_trusted(&test_repo));

        let reloaded = WorkspaceTrustStore::with_custom_file(store_file);
        assert!(!reloaded.is_trusted(&test_repo));
    }
}
