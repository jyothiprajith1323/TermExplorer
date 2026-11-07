//! Word-completion engine (file-local + workspace words).

use std::collections::BTreeSet;

/// A single completion candidate.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Completion {
    pub label: String,
    pub detail: Option<String>,
    pub kind: CompletionKind,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompletionKind {
    Word,
    Path,
    Snippet,
    Lsp,
}

/// Gather word-completions from the buffer lines for `prefix`.
pub fn word_completions(lines: &[String], prefix: &str, max: usize) -> Vec<Completion> {
    if prefix.is_empty() {
        return Vec::new();
    }
    let mut seen = BTreeSet::new();
    let prefix_lower = prefix.to_lowercase();
    for line in lines {
        for word in line.split(|c: char| !c.is_alphanumeric() && c != '_') {
            if word.len() > prefix.len()
                && word.to_lowercase().starts_with(&prefix_lower)
                && word != prefix
            {
                seen.insert(word.to_string());
            }
        }
    }
    seen.into_iter()
        .take(max)
        .map(|label| Completion {
            label,
            detail: Some("buffer".to_string()),
            kind: CompletionKind::Word,
        })
        .collect()
}

/// Extract the word immediately before the cursor column in a line.
pub fn prefix_at(line: &str, col: usize) -> &str {
    let bytes = line.as_bytes();
    let end = col.min(bytes.len());
    let start = bytes[..end]
        .iter()
        .rposition(|&b| !b.is_ascii_alphanumeric() && b != b'_')
        .map(|i| i + 1)
        .unwrap_or(0);
    &line[start..end]
}
