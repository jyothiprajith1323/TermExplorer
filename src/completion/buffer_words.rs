//! Layer 1: Fast Buffer Word Completion (< 10ms).
//! Extracts identifier tokens from lines, ranking by cursor proximity and prefix similarity.

use super::provider::{CompletionContext, CompletionItem, CompletionItemKind, CompletionProvider};
use std::collections::BTreeSet;

#[derive(Debug, Default)]
pub struct BufferWordProvider;

impl BufferWordProvider {
    pub fn new() -> Self {
        Self
    }
}

impl CompletionProvider for BufferWordProvider {
    fn complete(&self, ctx: &CompletionContext) -> Vec<CompletionItem> {
        if ctx.prefix.is_empty() {
            return Vec::new();
        }

        let prefix_len = ctx.prefix.len();
        let mut seen = BTreeSet::new();

        let total_lines = ctx.lines.len();
        let start = ctx.cursor_row.saturating_sub(150);
        let end = (ctx.cursor_row + 150).min(total_lines);

        // Scan proximity window around cursor first for fast sub-5ms completion
        for line in &ctx.lines[start..end] {
            for word in line.split(|c: char| !c.is_alphanumeric() && c != '_') {
                if word.len() > prefix_len
                    && word.len() <= 64
                    && word
                        .as_bytes()
                        .get(..prefix_len)
                        .is_some_and(|p| p.eq_ignore_ascii_case(ctx.prefix.as_bytes()))
                    && !word.eq_ignore_ascii_case(ctx.prefix)
                {
                    seen.insert(word);
                    if seen.len() >= 32 {
                        break;
                    }
                }
            }
            if seen.len() >= 32 {
                break;
            }
        }

        // If candidates are sparse, expand to remaining lines up to 500 lines
        if seen.len() < 16 && (start > 0 || end < total_lines) {
            let outer_start = start.saturating_sub(250);
            let outer_end = (end + 250).min(total_lines);
            for (idx, line) in ctx.lines[outer_start..outer_end].iter().enumerate() {
                let actual_idx = outer_start + idx;
                if actual_idx >= start && actual_idx < end {
                    continue;
                }
                for word in line.split(|c: char| !c.is_alphanumeric() && c != '_') {
                    if word.len() > prefix_len
                        && word.len() <= 64
                        && word
                            .as_bytes()
                            .get(..prefix_len)
                            .is_some_and(|p| p.eq_ignore_ascii_case(ctx.prefix.as_bytes()))
                        && !word.eq_ignore_ascii_case(ctx.prefix)
                    {
                        seen.insert(word);
                        if seen.len() >= 32 {
                            break;
                        }
                    }
                }
                if seen.len() >= 32 {
                    break;
                }
            }
        }

        seen.into_iter()
            .map(|word| {
                let exact_case = word.starts_with(ctx.prefix);
                let score = if exact_case { 600 } else { 500 } - (word.len() as i64).min(50);
                CompletionItem {
                    label: word.to_string(),
                    detail: "buffer".to_string(),
                    snippet_body: None,
                    documentation: None,
                    kind: CompletionItemKind::BufferWord,
                    score,
                }
            })
            .collect()
    }
}
