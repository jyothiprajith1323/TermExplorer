//! Completion provider traits and shared models.

use std::path::Path;

/// Kind of editor completion item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompletionItemKind {
    BufferWord,
    WorkspaceSymbol,
    Snippet,
    Lsp,
}

impl CompletionItemKind {
    pub fn badge(&self) -> &'static str {
        match self {
            Self::BufferWord => "buf",
            Self::WorkspaceSymbol => "sym",
            Self::Snippet => "snip",
            Self::Lsp => "lsp",
        }
    }
}

/// A completion candidate returned by any provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionItem {
    pub label: String,
    pub detail: String,
    pub snippet_body: Option<String>,
    pub documentation: Option<String>,
    pub kind: CompletionItemKind,
    pub score: i64,
}

/// Context passed to completion providers.
pub struct CompletionContext<'a> {
    pub file_path: Option<&'a Path>,
    pub lines: &'a [String],
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub prefix: &'a str,
}

/// Trait implemented by all completion layer providers.
pub trait CompletionProvider: Send + Sync {
    fn complete(&self, ctx: &CompletionContext) -> Vec<CompletionItem>;
}
