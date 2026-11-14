//! Central unified search engine orchestrating file and content queries.

use crate::search::cancellation::SearchCancellation;
use crate::search::content_search::{search_content, ContentMatch};
use crate::search::file_search::{search_files_fuzzy, FileSearchResult};
use crate::search::filters::SearchFilters;
use crate::search::history::SearchHistory;
use std::path::PathBuf;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchMode {
    #[default]
    Files,
    Content,
}

#[derive(Debug, Clone)]
pub enum SearchResultEvent {
    FileResults {
        generation: u64,
        results: Vec<FileSearchResult>,
    },
    ContentResults {
        generation: u64,
        results: Vec<ContentMatch>,
    },
}

pub struct SearchEngine {
    pub mode: SearchMode,
    pub query: String,
    pub filters: SearchFilters,
    pub file_results: Vec<FileSearchResult>,
    pub content_results: Vec<ContentMatch>,
    pub selected_index: usize,
    pub is_searching: bool,
    pub cancellation: SearchCancellation,
    pub history: SearchHistory,
    pub sender: mpsc::UnboundedSender<SearchResultEvent>,
    pub receiver: mpsc::UnboundedReceiver<SearchResultEvent>,
}

impl SearchEngine {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            mode: SearchMode::Files,
            query: String::new(),
            filters: SearchFilters::default(),
            file_results: Vec::new(),
            content_results: Vec::new(),
            selected_index: 0,
            is_searching: false,
            cancellation: SearchCancellation::new(),
            history: SearchHistory::load(),
            sender: tx,
            receiver: rx,
        }
    }

    /// Trigger asynchronous search across the workspace for the current query.
    pub fn trigger_search(&mut self, root: PathBuf) {
        let generation = self.cancellation.advance();
        let query = self.query.clone();
        let mode = self.mode;
        let filters = self.filters.clone();
        let cancellation = self.cancellation.clone();
        let sender = self.sender.clone();

        self.is_searching = true;
        self.selected_index = 0;

        tokio::task::spawn_blocking(move || match mode {
            SearchMode::Files => {
                let results =
                    search_files_fuzzy(&root, &query, &filters, &cancellation, generation);
                let _ = sender.send(SearchResultEvent::FileResults {
                    generation,
                    results,
                });
            }
            SearchMode::Content => {
                let results = search_content(&root, &query, &filters, &cancellation, generation);
                let _ = sender.send(SearchResultEvent::ContentResults {
                    generation,
                    results,
                });
            }
        });
    }

    /// Poll incoming search events from the background thread.
    pub fn poll_events(&mut self) {
        while let Ok(event) = self.receiver.try_recv() {
            match event {
                SearchResultEvent::FileResults {
                    generation,
                    results,
                } => {
                    if !self.cancellation.is_cancelled(generation) {
                        self.file_results = results;
                        self.is_searching = false;
                        self.clamp_selection();
                    }
                }
                SearchResultEvent::ContentResults {
                    generation,
                    results,
                } => {
                    if !self.cancellation.is_cancelled(generation) {
                        self.content_results = results;
                        self.is_searching = false;
                        self.clamp_selection();
                    }
                }
            }
        }
    }

    pub fn select_next(&mut self) {
        let len = self.current_len();
        if len > 0 && self.selected_index + 1 < len {
            self.selected_index += 1;
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn clamp_selection(&mut self) {
        let len = self.current_len();
        if len == 0 {
            self.selected_index = 0;
        } else if self.selected_index >= len {
            self.selected_index = len.saturating_sub(1);
        }
    }

    pub fn current_len(&self) -> usize {
        match self.mode {
            SearchMode::Files => self.file_results.len(),
            SearchMode::Content => self.content_results.len(),
        }
    }

    pub fn selected_target(&self) -> Option<(PathBuf, Option<usize>)> {
        match self.mode {
            SearchMode::Files => self
                .file_results
                .get(self.selected_index)
                .map(|r| (r.path.clone(), None)),
            SearchMode::Content => self
                .content_results
                .get(self.selected_index)
                .map(|m| (m.path.clone(), Some(m.line_number))),
        }
    }

    pub fn toggle_mode(&mut self, root: PathBuf) {
        self.mode = match self.mode {
            SearchMode::Files => SearchMode::Content,
            SearchMode::Content => SearchMode::Files,
        };
        self.trigger_search(root);
    }

    pub fn commit_history(&mut self) {
        match self.mode {
            SearchMode::Files => self.history.push_file_query(&self.query),
            SearchMode::Content => self.history.push_content_query(&self.query),
        }
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}
