//! Bounded LRU cache for file previews.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct CachedPreview<T> {
    pub path: PathBuf,
    pub modified: Option<SystemTime>,
    pub data: T,
}

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
