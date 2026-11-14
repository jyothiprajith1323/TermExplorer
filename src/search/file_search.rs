//! High-performance workspace fuzzy filename search using ignore::WalkBuilder and SkimMatcherV2.

use crate::search::cancellation::SearchCancellation;
use crate::search::filters::SearchFilters;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FileSearchResult {
    pub path: PathBuf,
    pub relative_path: String,
    pub file_name: String,
    pub is_dir: bool,
    pub score: i64,
}

pub fn search_files_fuzzy(
    root: &Path,
    query: &str,
    filters: &SearchFilters,
    cancellation: &SearchCancellation,
    generation: u64,
) -> Vec<FileSearchResult> {
    if cancellation.is_cancelled(generation) {
        return Vec::new();
    }

    let matcher = SkimMatcherV2::default();
    let q = query.trim();

    let mut builder = WalkBuilder::new(root);
    builder
        .hidden(!filters.include_hidden)
        .git_ignore(filters.respect_gitignore)
        .git_global(filters.respect_gitignore)
        .git_exclude(filters.respect_gitignore)
        .parents(false)
        .filter_entry(|entry| {
            let name = entry.file_name().to_string_lossy();
            // Prune heavy directories early
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                && (name == ".git"
                    || name == "node_modules"
                    || name == "target"
                    || name == ".cargo")
            {
                return false;
            }
            true
        });

    let mut results = Vec::new();
    let mut count = 0;

    for result in builder.build() {
        count += 1;
        if count % 100 == 0 && cancellation.is_cancelled(generation) {
            return Vec::new();
        }

        let Ok(entry) = result else { continue };
        let path = entry.path().to_path_buf();

        if path == root {
            continue;
        }

        // Filter by extension if requested
        if let Some(ref req_ext) = filters.file_extension {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if ext != req_ext.to_ascii_lowercase() {
                continue;
            }
        }

        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let relative_path = path
            .strip_prefix(root)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| path.display().to_string());

        let score = if q.is_empty() {
            0
        } else {
            // Match against filename with high weight, or relative path with normal weight
            if let Some(fn_score) = matcher.fuzzy_match(&file_name, q) {
                fn_score + 100
            } else if let Some(rel_score) = matcher.fuzzy_match(&relative_path, q) {
                rel_score
            } else {
                continue;
            }
        };

        results.push(FileSearchResult {
            path,
            relative_path,
            file_name,
            is_dir,
            score,
        });

        if results.len() >= filters.max_results * 2 {
            break;
        }
    }

    if !q.is_empty() {
        results.sort_by_key(|a| std::cmp::Reverse(a.score));
    } else {
        // Natural alphabetical ordering for empty queries
        results.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    }

    results.truncate(filters.max_results);
    results
}
