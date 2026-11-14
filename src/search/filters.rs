//! Search query filters and configuration options.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchFilters {
    pub case_sensitive: bool,
    pub is_regex: bool,
    pub whole_word: bool,
    pub include_hidden: bool,
    pub respect_gitignore: bool,
    pub file_extension: Option<String>,
    pub max_results: usize,
}

impl Default for SearchFilters {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            is_regex: false,
            whole_word: false,
            include_hidden: false,
            respect_gitignore: true,
            file_extension: None,
            max_results: 500,
        }
    }
}
