//! In-directory file search with fuzzy matching.

use crate::filesystem::browser::DirEntry;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

/// Filter a list of entries using a fuzzy query string.
/// Returns entries together with their match score, sorted descending.
pub fn fuzzy_filter(entries: &[DirEntry], query: &str) -> Vec<(i64, usize)> {
    if query.is_empty() {
        return entries.iter().enumerate().map(|(i, _)| (0i64, i)).collect();
    }

    let matcher = SkimMatcherV2::default();
    let mut scored: Vec<(i64, usize)> = entries
        .iter()
        .enumerate()
        .filter_map(|(i, e)| matcher.fuzzy_match(&e.name, query).map(|score| (score, i)))
        .collect();

    scored.sort_by_key(|a| std::cmp::Reverse(a.0));
    scored
}

/// Simple case-insensitive substring filter.
#[allow(dead_code)]
pub fn substring_filter<'a>(entries: &'a [DirEntry], query: &str) -> Vec<&'a DirEntry> {
    let q = query.to_lowercase();
    entries
        .iter()
        .filter(|e| e.name.to_lowercase().contains(&q))
        .collect()
}
