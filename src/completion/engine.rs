//! Central completion orchestrator uniting all 4 layers:
//! Layer 1: Buffer words (< 10ms)
//! Layer 2: Workspace symbols (< 50ms)
//! Layer 3: Snippet expander (instant)
//! Layer 4: Async non-blocking LSP provider

use super::buffer_words::BufferWordProvider;
use super::cache::CompletionCache;
use super::lsp_provider::LspProvider;
use super::provider::{CompletionContext, CompletionItem, CompletionProvider};
use super::ranking::rank_completions;
use super::snippets::SnippetProvider;
use super::workspace_symbols::WorkspaceIndexer;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct CentralCompletionEngine {
    pub buffer_words: Arc<BufferWordProvider>,
    pub workspace_indexer: Arc<WorkspaceIndexer>,
    pub snippets: Arc<SnippetProvider>,
    pub lsp_provider: Arc<LspProvider>,
    pub cache: Arc<CompletionCache>,
}

impl CentralCompletionEngine {
    pub fn new() -> Self {
        Self {
            buffer_words: Arc::new(BufferWordProvider::new()),
            workspace_indexer: Arc::new(WorkspaceIndexer::new()),
            snippets: Arc::new(SnippetProvider::new()),
            lsp_provider: Arc::new(LspProvider::new()),
            cache: Arc::new(CompletionCache::new()),
        }
    }

    /// Background index a workspace directory.
    pub fn index_workspace(&self, root: &Path) {
        self.workspace_indexer.index_directory(root);
    }

    /// Query unified 4-layer completions at cursor.
    pub fn complete(&self, ctx: &CompletionContext, max: usize) -> Vec<CompletionItem> {
        if ctx.prefix.len() < 2 {
            return Vec::new();
        }

        // Check cache for instant lookup
        let cache_key = format!(
            "{}:{}:{}",
            ctx.file_path.map_or("", |p| p.to_str().unwrap_or("")),
            ctx.prefix,
            ctx.cursor_row
        );
        if let Some(cached) = self.cache.get(&cache_key) {
            return cached;
        }

        let mut all = Vec::new();

        // 1. Layer 3: Snippets
        let snips = self.snippets.complete(ctx);
        all.extend(snips);

        // 2. Layer 4: LSP (high priority contextual items)
        let lsp_items = self.lsp_provider.complete(ctx);
        all.extend(lsp_items);

        // 3. Layer 1: Buffer words
        let buf_words = self.buffer_words.complete(ctx);
        all.extend(buf_words);

        // 4. Layer 2: Workspace symbols
        let syms = self.workspace_indexer.complete(ctx);
        all.extend(syms);

        // Rank and deduplicate
        let ranked = rank_completions(all, max);
        self.cache.insert(cache_key, ranked.clone());
        ranked
    }
}
