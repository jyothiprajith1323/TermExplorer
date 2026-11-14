//! Search history management and XDG state directory persistence.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchHistory {
    pub file_queries: Vec<String>,
    pub content_queries: Vec<String>,
}

impl SearchHistory {
    pub fn load() -> Self {
        let path = Self::history_path();
        if let Ok(content) = fs::read_to_string(&path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let path = Self::history_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&path, json);
        }
    }

    pub fn push_file_query(&mut self, query: &str) {
        let q = query.trim().to_string();
        if q.is_empty() {
            return;
        }
        self.file_queries.retain(|x| x != &q);
        self.file_queries.insert(0, q);
        if self.file_queries.len() > 50 {
            self.file_queries.truncate(50);
        }
        self.save();
    }

    pub fn push_content_query(&mut self, query: &str) {
        let q = query.trim().to_string();
        if q.is_empty() {
            return;
        }
        self.content_queries.retain(|x| x != &q);
        self.content_queries.insert(0, q);
        if self.content_queries.len() > 50 {
            self.content_queries.truncate(50);
        }
        self.save();
    }

    pub fn clear(&mut self) {
        self.file_queries.clear();
        self.content_queries.clear();
        self.save();
    }

    fn history_path() -> PathBuf {
        let state_dir = dirs::state_dir()
            .unwrap_or_else(|| dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("/tmp")))
            .join("termexplorer");
        state_dir.join("search_history.json")
    }
}
