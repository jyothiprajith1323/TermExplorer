//! Unified search engine for workspace fuzzy filename discovery and live content grep.

pub mod cancellation;
pub mod content_search;
pub mod engine;
pub mod file_search;
pub mod filters;
pub mod history;

pub use cancellation::SearchCancellation;
pub use content_search::{search_content, ContentMatch};
pub use engine::{SearchEngine, SearchMode};
pub use file_search::{search_files_fuzzy, FileSearchResult};
pub use filters::SearchFilters;
pub use history::SearchHistory;
