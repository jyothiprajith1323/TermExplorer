//! Word and layered completion engine (file-local + workspace symbols + snippets).

pub use crate::completion::editor::{
    buffer_word_completions, get_language_snippets, get_layered_completions, EditorCompletion,
    EditorCompletionKind,
};
use std::path::Path;

use unicode_segmentation::UnicodeSegmentation;

/// A single completion candidate.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Completion {
    pub label: String,
    pub detail: Option<String>,
    pub snippet_body: Option<String>,
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
            snippet_body: it.snippet_body,
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
                snippet_body: it.snippet_body,
                kind,
            }
        })
        .collect()
}

/// Extract the word immediately before the grapheme cursor column in a line.
/// Unicode- and grapheme-safe for emoji, combining marks, and multibyte scripts.
pub fn prefix_at(line: &str, col: usize) -> &str {
    let graphemes: Vec<(usize, &str)> = line.grapheme_indices(true).collect();
    let col = col.min(graphemes.len());
    let mut start_grapheme = col;
    while start_grapheme > 0 {
        let g = graphemes[start_grapheme - 1].1;
        if g.chars().all(|c| c.is_alphanumeric() || c == '_') {
            start_grapheme -= 1;
        } else {
            break;
        }
    }
    let byte_start = if start_grapheme < graphemes.len() {
        graphemes[start_grapheme].0
    } else {
        line.len()
    };
    let byte_end = if col < graphemes.len() {
        graphemes[col].0
    } else {
        line.len()
    };
    &line[byte_start..byte_end]
}
