//! Explorer and Path Autocompletion Engine.
//! Provides fast, fuzzy path completion, directory expansion (~/ and relative),
//! and pinned/recent folder suggestions with < 50ms latency.

use crate::config::PinnedFolder;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::path::{Path, PathBuf};

/// A path completion candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathCompletion {
    /// The full completed path string to replace or insert.
    pub completed_path: String,
    /// Human-friendly display label (e.g., "src/main.rs" or "📁 Projects/").
    pub display_label: String,
    /// Detailed hint (e.g. "directory", "file", "pinned").
    pub detail: String,
    /// Whether this target is a directory.
    pub is_dir: bool,
    /// Ranking score (higher is better).
    pub score: i64,
}

/// Expand tilde `~` to the user's home directory.
pub fn expand_tilde(path_str: &str) -> PathBuf {
    let expanded = shellexpand::tilde(path_str);
    PathBuf::from(expanded.as_ref())
}

/// Autocomplete a filesystem path given a user input query and reference directory.
/// Target latency: < 50ms (typical: 0.1ms - 2ms).
pub fn complete_path(input: &str, current_dir: &Path, max: usize) -> Vec<PathCompletion> {
    if input.trim().is_empty() {
        return Vec::new();
    }

    let input_trimmed = input.trim();
    let has_tilde = input_trimmed.starts_with('~');
    let has_slash = input_trimmed.contains('/') || input_trimmed.contains('\\');

    let (parent_dir, file_prefix, is_absolute) = if has_tilde {
        let expanded = expand_tilde(input_trimmed);
        if input_trimmed.ends_with('/') || input_trimmed.ends_with('\\') {
            (expanded, String::new(), true)
        } else {
            let parent = expanded.parent().unwrap_or(&expanded).to_path_buf();
            let prefix = expanded
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default();
            (parent, prefix, true)
        }
    } else if has_slash {
        let p = Path::new(input_trimmed);
        let is_abs = p.is_absolute();
        if input_trimmed.ends_with('/') || input_trimmed.ends_with('\\') {
            let dir = if is_abs {
                p.to_path_buf()
            } else {
                current_dir.join(p)
            };
            (dir, String::new(), is_abs)
        } else {
            let parent_rel = p.parent().unwrap_or_else(|| Path::new(""));
            let dir = if is_abs {
                parent_rel.to_path_buf()
            } else {
                current_dir.join(parent_rel)
            };
            let prefix = p
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default();
            (dir, prefix, is_abs)
        }
    } else {
        (current_dir.to_path_buf(), input_trimmed.to_string(), false)
    };

    if !parent_dir.is_dir() {
        return Vec::new();
    }

    let entries = match std::fs::read_dir(&parent_dir) {
        Ok(rd) => rd,
        Err(_) => return Vec::new(),
    };

    let matcher = SkimMatcherV2::default();
    let prefix_lower = file_prefix.to_lowercase();
    let mut candidates = Vec::new();

    for entry in entries.flatten() {
        let file_name = entry.file_name().to_string_lossy().to_string();
        let file_name_lower = file_name.to_lowercase();
        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

        // Compute score
        let score = if file_prefix.is_empty() {
            if is_dir {
                100
            } else {
                50
            }
        } else if file_name_lower == prefix_lower {
            1000 + if is_dir { 50 } else { 0 }
        } else if file_name_lower.starts_with(&prefix_lower) {
            500 + (file_name.len() as i64).min(50) + if is_dir { 30 } else { 0 }
        } else if let Some(fuzzy_score) = matcher.fuzzy_match(&file_name, &file_prefix) {
            fuzzy_score + if is_dir { 20 } else { 0 }
        } else {
            continue;
        };

        // Construct completed path string
        let completed = if has_tilde {
            let mut s = input_trimmed.to_string();
            if let Some(last_slash) = s.rfind('/') {
                s.truncate(last_slash + 1);
            } else {
                s.clear();
            }
            s.push_str(&file_name);
            if is_dir {
                s.push('/');
            }
            s
        } else if is_absolute {
            let mut s = parent_dir.join(&file_name).to_string_lossy().to_string();
            if is_dir && !s.ends_with('/') {
                s.push('/');
            }
            s
        } else if has_slash {
            let mut s = input_trimmed.to_string();
            if let Some(last_slash) = s.rfind('/') {
                s.truncate(last_slash + 1);
            } else {
                s.clear();
            }
            s.push_str(&file_name);
            if is_dir {
                s.push('/');
            }
            s
        } else {
            let mut s = file_name.clone();
            if is_dir {
                s.push('/');
            }
            s
        };

        let display_label = if is_dir {
            format!("📁 {}/", file_name)
        } else {
            format!("📄 {}", file_name)
        };

        candidates.push(PathCompletion {
            completed_path: completed,
            display_label,
            detail: if is_dir {
                "directory".to_string()
            } else {
                "file".to_string()
            },
            is_dir,
            score,
        });
    }

    candidates.sort_by_key(|a| std::cmp::Reverse(a.score));
    candidates.truncate(max);
    candidates
}

/// Suggest pinned folders and recent directories matching a search query.
pub fn suggest_locations(
    query: &str,
    pinned: &[PinnedFolder],
    recent: &[PathBuf],
    max: usize,
) -> Vec<PathCompletion> {
    let mut results = Vec::new();
    let query_lower = query.trim().to_lowercase();
    let matcher = SkimMatcherV2::default();

    // 1. Pinned folders
    for pf in pinned {
        let name_lower = pf.name.to_lowercase();
        let path_str = pf.path.to_string_lossy().to_string();
        let score = if query_lower.is_empty() {
            200
        } else if name_lower == query_lower {
            1000
        } else if name_lower.starts_with(&query_lower) {
            600
        } else if let Some(s) = matcher.fuzzy_match(&pf.name, query) {
            s + 200
        } else if let Some(s) = matcher.fuzzy_match(&path_str, query) {
            s + 100
        } else {
            continue;
        };

        results.push(PathCompletion {
            completed_path: path_str.clone(),
            display_label: format!("📌 {}", pf.name),
            detail: format!("Pinned • {}", path_str),
            is_dir: true,
            score,
        });
    }

    // 2. Recent directories
    for rd in recent {
        let path_str = rd.to_string_lossy().to_string();
        let name = rd
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| path_str.clone());
        let name_lower = name.to_lowercase();

        let score = if query_lower.is_empty() {
            150
        } else if name_lower == query_lower {
            900
        } else if name_lower.starts_with(&query_lower) {
            500
        } else if let Some(s) = matcher.fuzzy_match(&name, query) {
            s + 150
        } else if let Some(s) = matcher.fuzzy_match(&path_str, query) {
            s + 50
        } else {
            continue;
        };

        results.push(PathCompletion {
            completed_path: path_str.clone(),
            display_label: format!("🕒 {}", name),
            detail: format!("Recent • {}", path_str),
            is_dir: true,
            score,
        });
    }

    results.sort_by_key(|a| std::cmp::Reverse(a.score));
    results.truncate(max);
    results
}
