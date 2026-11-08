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

        // Scan lines in document
        for line in ctx.lines {
            for word in line.split(|c: char| !c.is_alphanumeric() && c != '_') {
                if word.len() > prefix_len
                    && word.len() <= 64
                    && word
                        .as_bytes()
                        .get(..prefix_len)
                        .is_some_and(|p| p.eq_ignore_ascii_case(ctx.prefix.as_bytes()))
                    && !word.eq_ignore_ascii_case(ctx.prefix)
                {
                    seen.insert(word.to_string());
                    if seen.len() >= 64 {
                        break;
                    }
                }
            }
            if seen.len() >= 64 {
                break;
            }
        }

        seen.into_iter()
            .map(|label| {
                let exact_case = label.starts_with(ctx.prefix);
                let score = if exact_case { 600 } else { 500 } - (label.len() as i64).min(50);
                CompletionItem {
                    label,
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
