//! Word and layered completion engine (file-local + workspace symbols + snippets).

pub use crate::completion::editor::{
    buffer_word_completions, get_language_snippets, get_layered_completions, EditorCompletion,
    EditorCompletionKind,
};
use std::path::Path;

/// A single completion candidate.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Completion {
    pub label: String,
    pub detail: Option<String>,
    pub kind: CompletionKind,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompletionKind {
    Word,
    Path,
    Snippet,
    Lsp,
    WorkspaceSymbol,
}

/// Gather word-completions from the buffer lines for `prefix`.
pub fn word_completions(lines: &[String], prefix: &str, max: usize) -> Vec<Completion> {
    let items = buffer_word_completions(lines, prefix, max);
    items
        .into_iter()
        .map(|it| Completion {
            label: it.label,
            detail: Some(it.detail),
            kind: CompletionKind::Word,
        })
        .collect()
}

/// Gather layered completions (buffer words + snippets + workspace symbols).
pub fn layered_completions(
    lines: &[String],
    prefix: &str,
    file_path: Option<&Path>,
    max: usize,
) -> Vec<Completion> {
    let items = get_layered_completions(lines, prefix, file_path, None, max);
    items
        .into_iter()
        .map(|it| {
            let kind = match it.kind {
                EditorCompletionKind::BufferWord => CompletionKind::Word,
                EditorCompletionKind::WorkspaceSymbol => CompletionKind::WorkspaceSymbol,
                EditorCompletionKind::Snippet => CompletionKind::Snippet,
                EditorCompletionKind::Lsp => CompletionKind::Lsp,
            };
            Completion {
                label: it.label,
                detail: Some(it.detail),
                kind,
            }
        })
        .collect()
}

/// Extract the word immediately before the cursor column in a line (char-safe).
pub fn prefix_at(line: &str, col: usize) -> &str {
    let chars: Vec<char> = line.chars().collect();
    let end = col.min(chars.len());
    let start = (0..end)
        .rev()
        .find(|&i| !chars[i].is_alphanumeric() && chars[i] != '_')
        .map(|i| i + 1)
        .unwrap_or(0);

    let byte_start = line
        .char_indices()
        .nth(start)
        .map(|(b, _)| b)
        .unwrap_or(line.len());
    let byte_end = line
        .char_indices()
        .nth(end)
        .map(|(b, _)| b)
        .unwrap_or(line.len());
    &line[byte_start..byte_end]
}
