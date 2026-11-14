//! Bounded memory and disk cache for file previews, thumbnails, and render outputs.

use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct CachedPreview<T> {
    pub path: PathBuf,
    pub modified: Option<SystemTime>,
    pub data: T,
}

/// In-memory LRU cache for fast preview lookups.
pub struct PreviewLruCache<T> {
    capacity: usize,
    entries: VecDeque<CachedPreview<T>>,
}

impl<T: Clone> PreviewLruCache<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            entries: VecDeque::with_capacity(capacity),
        }
    }

    pub fn get(&mut self, path: &Path, modified: Option<SystemTime>) -> Option<T> {
        let idx = self
            .entries
            .iter()
            .position(|e| e.path == path && (modified.is_none() || e.modified == modified))?;

        // Move to front (LRU)
        if let Some(entry) = self.entries.remove(idx) {
            let data = entry.data.clone();
            self.entries.push_front(entry);
            Some(data)
        } else {
            None
        }
    }

    pub fn insert(&mut self, path: PathBuf, modified: Option<SystemTime>, data: T) {
        // Remove existing if present
        self.entries.retain(|e| e.path != path);

        if self.entries.len() >= self.capacity {
            self.entries.pop_back();
        }

        self.entries.push_front(CachedPreview {
            path,
            modified,
            data,
        });
    }

    pub fn invalidate(&mut self, path: &Path) {
        self.entries.retain(|e| e.path != path);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// Persistent on-disk preview cache in `$XDG_CACHE_HOME/termexplorer/`.
/// Uses BLAKE3 fingerprinting and atomic writes.
pub struct DiskPreviewCache {
    cache_dir: PathBuf,
    max_bytes: u64,
}

impl DiskPreviewCache {
    pub fn new(max_bytes: u64) -> Self {
        let cache_base = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("termexplorer")
            .join("previews");

        let _ = fs::create_dir_all(&cache_base);

        Self {
            cache_dir: cache_base,
            max_bytes,
        }
    }

    /// Compute a BLAKE3 fingerprint for a file, sampling up to 64 KiB for large files.
    pub fn compute_fingerprint(path: &Path) -> Option<String> {
        let mut file = File::open(path).ok()?;
        let mut hasher = blake3::Hasher::new();
        let mut buf = [0u8; 65536];

        let bytes_read = file.read(&mut buf).ok()?;
        hasher.update(&buf[..bytes_read]);

        // Incorporate metadata (size + mtime) to guard against identical sampled headers
        if let Ok(meta) = file.metadata() {
            hasher.update(&meta.len().to_le_bytes());
            if let Ok(mtime) = meta.modified() {
                if let Ok(dur) = mtime.duration_since(SystemTime::UNIX_EPOCH) {
                    hasher.update(&dur.as_nanos().to_le_bytes());
                }
            }
        }

        Some(hasher.finalize().to_hex().to_string())
    }

    /// Read cached binary payload (e.g. rasterized thumbnail or SVG render) by fingerprint.
    pub fn read_cached(&self, fingerprint: &str, extension: &str) -> Option<Vec<u8>> {
        let filename = format!("{fingerprint}.{extension}");
        let target = self.cache_dir.join(filename);
        if target.exists() {
            // Touch mtime for LRU eviction
            let _ = fs::File::open(&target).and_then(|f| f.set_modified(SystemTime::now()));
            fs::read(&target).ok()
        } else {
            None
        }
    }

    /// Atomically write cached payload to disk and maintain max size limits.
    pub fn write_cached(
        &self,
        fingerprint: &str,
        extension: &str,
        bytes: &[u8],
    ) -> std::io::Result<PathBuf> {
        let target = self.cache_dir.join(format!("{fingerprint}.{extension}"));

        // Atomic write via tempfile in cache directory
        let temp_dir = &self.cache_dir;
        let mut temp_file = tempfile::NamedTempFile::new_in(temp_dir)?;
        temp_file.write_all(bytes)?;
        temp_file.persist(&target).map_err(|e| e.error)?;

        // Check and enforce total cache size bounds
        self.enforce_capacity();

        Ok(target)
    }

    /// Enforce maximum disk cache size through LRU file deletion.
    fn enforce_capacity(&self) {
        let mut entries = Vec::new();
        let mut total_size = 0u64;

        if let Ok(dir_entries) = fs::read_dir(&self.cache_dir) {
            for entry in dir_entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                    let size = meta.len();
                    total_size += size;
                    entries.push((entry.path(), mtime, size));
                }
            }
        }

        if total_size > self.max_bytes {
            // Sort oldest mtime first
            entries.sort_by_key(|e| e.1);
            for (path, _, size) in entries {
                let _ = fs::remove_file(path);
                total_size = total_size.saturating_sub(size);
                if total_size <= self.max_bytes * 8 / 10 {
                    // Evicted down to 80% threshold
                    break;
                }
            }
        }
    }

    pub fn clear(&self) {
        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}
