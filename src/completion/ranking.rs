//! Ranking and deduplication engine for multi-layer completion candidates.

use super::provider::CompletionItem;
use std::collections::BTreeMap;

/// Rank and deduplicate completion items, keeping the highest-scoring candidate for each label.
pub fn rank_completions(items: Vec<CompletionItem>, max: usize) -> Vec<CompletionItem> {
    if items.is_empty() {
        return Vec::new();
    }

    let mut deduped: BTreeMap<String, CompletionItem> = BTreeMap::new();
    for item in items {
        if let Some(existing) = deduped.get_mut(&item.label) {
            if item.score > existing.score {
                *existing = item;
            }
        } else {
            deduped.insert(item.label.clone(), item);
        }
    }

    let mut list: Vec<CompletionItem> = deduped.into_values().collect();
    list.sort_by_key(|a| std::cmp::Reverse(a.score));
    list.truncate(max);
    list
}
