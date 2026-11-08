//! Intelligent Autocomplete & Suggestions Engine for TermExplorer.
//!
//! Provides unified multi-layer autocompletion across:
//! - File Explorer path completion and fuzzy search (`explorer`)
//! - Command Palette intelligent ranking and location navigation (`palette`)
//! - Editor layered code autocompletion: buffer, workspace symbols, snippets, LSP (`editor`)
//! - Latency and performance benchmarking harness (`benchmark`)
//! - Modular 4-layer engine architecture (`engine`, `provider`, `buffer_words`, `workspace_symbols`, `snippets`, `lsp_provider`)

pub mod benchmark;
pub mod buffer_words;
pub mod cache;
pub mod cancellation;
pub mod editor;
pub mod engine;
pub mod explorer;
pub mod lsp_provider;
pub mod palette;
pub mod provider;
pub mod ranking;
pub mod snippets;
pub mod workspace_symbols;

pub use benchmark::CompletionBenchmarkMetrics;
pub use editor::{
    buffer_word_completions, get_language_snippets, get_layered_completions, CodeSnippet,
    EditorCompletion, EditorCompletionKind, LspClientHook, WorkspaceIndexer, WorkspaceSymbol,
};
pub use engine::CentralCompletionEngine;
pub use explorer::{complete_path, expand_tilde, suggest_locations, PathCompletion};
pub use lsp_provider::LspProvider;
pub use palette::{rank_palette_commands, PaletteSuggestion};
pub use provider::{CompletionContext, CompletionItem, CompletionItemKind, CompletionProvider};
pub use snippets::SnippetProvider;

use crate::config::PinnedFolder;
use crate::state::FocusedPanel;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Central completion coordinator for the application.
#[derive(Debug, Clone, Default)]
pub struct CompletionEngine {
    pub central: CentralCompletionEngine,
    pub workspace_indexer: Arc<WorkspaceIndexer>,
    pub lsp_hook: Arc<LspClientHook>,
}

impl CompletionEngine {
    pub fn new() -> Self {
        let central = CentralCompletionEngine::new();
        let workspace_indexer = central.workspace_indexer.clone();
        Self {
            central,
            workspace_indexer,
            lsp_hook: Arc::new(LspClientHook::new()),
        }
    }

    /// Background index a workspace directory.
    pub fn index_workspace(&self, root: &Path) {
        self.workspace_indexer.index_directory(root);
    }

    /// Complete editor text at cursor.
    pub fn complete_editor(
        &self,
        lines: &[String],
        prefix: &str,
        file_path: Option<&Path>,
        max: usize,
    ) -> Vec<EditorCompletion> {
        get_layered_completions(lines, prefix, file_path, Some(&self.workspace_indexer), max)
    }

    /// Complete filesystem path.
    pub fn complete_path(
        &self,
        input: &str,
        current_dir: &Path,
        max: usize,
    ) -> Vec<PathCompletion> {
        complete_path(input, current_dir, max)
    }

    /// Rank command palette suggestions.
    pub fn complete_palette(
        &self,
        query: &str,
        pinned: &[PinnedFolder],
        recent: &[PathBuf],
        current_panel: FocusedPanel,
    ) -> Vec<PaletteSuggestion> {
        rank_palette_commands(query, pinned, recent, current_panel)
    }
}
