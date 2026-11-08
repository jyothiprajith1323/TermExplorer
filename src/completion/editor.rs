//! Editor Layered Autocomplete & Suggestions Engine (Compatibility Facade).
//!
//! Provides backwards-compatible bridge to modular 4-layer autocomplete architecture:
//! - Layer 1: Current buffer word completion (< 10ms)
//! - Layer 2: Workspace symbol indexer with background scanning (< 50ms)
//! - Layer 3: Multi-language snippet expander (instant)
//! - Layer 4: Async non-blocking LSP client hook with stale cancellation

pub use super::cancellation::CancellationToken;
pub use super::provider::{CompletionContext, CompletionItemKind as EditorCompletionKind};
pub use super::snippets::{all_builtin_snippets, CodeSnippet};
pub use super::workspace_symbols::{WorkspaceIndexer, WorkspaceSymbol};

use super::buffer_words::BufferWordProvider;
use super::provider::CompletionProvider;
use super::ranking::rank_completions;
use super::snippets::SnippetProvider;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

/// A single completion candidate for the editor popup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorCompletion {
    pub label: String,
    pub detail: String,
    pub snippet_body: Option<String>,
    pub kind: EditorCompletionKind,
    pub score: i64,
}

// ── Layer 1: Current Buffer Words (< 10ms) ───────────────────────────────────

pub fn buffer_word_completions(
    lines: &[String],
    prefix: &str,
    max: usize,
) -> Vec<EditorCompletion> {
    let provider = BufferWordProvider::new();
    let ctx = CompletionContext {
        file_path: None,
        lines,
        cursor_row: 0,
        cursor_col: 0,
        prefix,
    };
    provider
        .complete(&ctx)
        .into_iter()
        .take(max)
        .map(|it| EditorCompletion {
            label: it.label,
            detail: it.detail,
            snippet_body: it.snippet_body,
            kind: it.kind,
            score: it.score,
        })
        .collect()
}

// ── Layer 3: Language Snippet Expander ───────────────────────────────────────

pub fn get_language_snippets(ext: Option<&str>, prefix: &str) -> Vec<EditorCompletion> {
    let dummy_path = ext.map(|e| std::path::PathBuf::from(format!("file.{}", e)));
    let provider = SnippetProvider::new();
    let ctx = CompletionContext {
        file_path: dummy_path.as_deref(),
        lines: &[],
        cursor_row: 0,
        cursor_col: 0,
        prefix,
    };
    provider
        .complete(&ctx)
        .into_iter()
        .map(|it| EditorCompletion {
            label: it.label,
            detail: it.detail,
            snippet_body: it.snippet_body,
            kind: it.kind,
            score: it.score,
        })
        .collect()
}

// ── Layer 4: Async Non-Blocking LSP Hook ─────────────────────────────────────

#[derive(Debug, Default)]
pub struct LspClientHook {
    current_request_id: AtomicU64,
}

impl LspClientHook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn request_completions(&self, _prefix: &str) -> u64 {
        self.current_request_id.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn is_response_valid(&self, response_id: u64) -> bool {
        response_id == self.current_request_id.load(Ordering::SeqCst)
    }
}

// ── Unified Layered Completion Function ──────────────────────────────────────

pub fn get_layered_completions(
    lines: &[String],
    prefix: &str,
    file_path: Option<&Path>,
    indexer: Option<&WorkspaceIndexer>,
    max: usize,
) -> Vec<EditorCompletion> {
    if prefix.len() < 2 {
        return Vec::new();
    }

    let mut all = Vec::new();

    // 1. Layer 3: Snippets
    let ext = file_path
        .and_then(|p| p.extension())
        .and_then(|e| e.to_str());
    let snippets = get_language_snippets(ext, prefix);
    all.extend(
        snippets
            .into_iter()
            .map(|s| super::provider::CompletionItem {
                label: s.label,
                detail: s.detail,
                snippet_body: s.snippet_body,
                documentation: None,
                kind: s.kind,
                score: s.score,
            }),
    );

    // 2. Layer 1: Buffer words
    let buffer_words = buffer_word_completions(lines, prefix, max);
    all.extend(
        buffer_words
            .into_iter()
            .map(|b| super::provider::CompletionItem {
                label: b.label,
                detail: b.detail,
                snippet_body: b.snippet_body,
                documentation: None,
                kind: b.kind,
                score: b.score,
            }),
    );

    // 3. Layer 2: Workspace symbols
    if let Some(idx) = indexer {
        let symbols = idx.query_symbols(prefix, max);
        all.extend(symbols);
    }

    // Rank and deduplicate
    let ranked = rank_completions(all, max);
    ranked
        .into_iter()
        .map(|it| EditorCompletion {
            label: it.label,
            detail: it.detail,
            snippet_body: it.snippet_body,
            kind: it.kind,
            score: it.score,
        })
        .collect()
}
