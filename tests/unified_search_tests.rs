use std::fs::{self, File};
use std::io::Write;
use std::time::Instant;
use tempfile::tempdir;
use termexplorer::search::cancellation::SearchCancellation;
use termexplorer::search::content_search::search_content;
use termexplorer::search::file_search::search_files_fuzzy;
use termexplorer::search::filters::SearchFilters;
use termexplorer::search::history::SearchHistory;

#[test]
fn test_fuzzy_file_search() {
    let dir = tempdir().expect("create temp dir");
    let root = dir.path();

    // Create test file structure
    fs::create_dir_all(root.join("src/preview")).unwrap();
    File::create(root.join("src/main.rs")).unwrap();
    File::create(root.join("src/lib.rs")).unwrap();
    File::create(root.join("src/preview/detector.rs")).unwrap();
    File::create(root.join("Cargo.toml")).unwrap();
    File::create(root.join("README.md")).unwrap();

    let cancellation = SearchCancellation::new();
    let filters = SearchFilters::default();

    // Search for "detect"
    let results = search_files_fuzzy(root, "detect", &filters, &cancellation, 0);
    assert!(!results.is_empty(), "Expected results for 'detect'");
    assert_eq!(results[0].file_name, "detector.rs");

    // Search for "cargo"
    let results = search_files_fuzzy(root, "cargo", &filters, &cancellation, 0);
    assert!(!results.is_empty());
    assert_eq!(results[0].file_name, "Cargo.toml");
}

#[test]
fn test_content_search_grep() {
    let dir = tempdir().expect("create temp dir");
    let root = dir.path();

    let rust_file = root.join("test.rs");
    let mut f = File::create(&rust_file).unwrap();
    writeln!(f, "pub fn calculate_hash() -> u64 {{").unwrap();
    writeln!(f, "    let blake3_token = 0xDEADBEEF;").unwrap();
    writeln!(f, "    blake3_token").unwrap();
    writeln!(f, "}}").unwrap();

    let cancellation = SearchCancellation::new();
    let mut filters = SearchFilters::default();

    // 1. Literal substring match
    let matches = search_content(root, "blake3_token", &filters, &cancellation, 0);
    assert_eq!(matches.len(), 2, "Expected 2 matches for blake3_token");
    assert_eq!(matches[0].line_number, 2);
    assert_eq!(matches[1].line_number, 3);

    // 2. Case sensitive search
    filters.case_sensitive = true;
    let matches_case = search_content(root, "BLAKE3_TOKEN", &filters, &cancellation, 0);
    assert!(
        matches_case.is_empty(),
        "Should not match uppercase when case_sensitive=true"
    );

    // 3. Regex search
    filters.is_regex = true;
    let regex_matches = search_content(root, r"0x[0-9A-F]+", &filters, &cancellation, 0);
    assert_eq!(regex_matches.len(), 1, "Expected 1 regex match");
    assert_eq!(regex_matches[0].line_number, 2);

    // 4. Whole word search
    filters.is_regex = false;
    filters.whole_word = true;
    let whole_matches = search_content(root, "blake3", &filters, &cancellation, 0);
    assert!(
        whole_matches.is_empty(),
        "'blake3' is not a whole word in blake3_token"
    );
}

#[test]
fn test_search_monotonic_cancellation_under_2ms() {
    let cancellation = SearchCancellation::new();
    let start_gen = cancellation.current();
    assert_eq!(start_gen, 0);

    let start = Instant::now();
    let next_gen = cancellation.advance();
    let elapsed = start.elapsed();

    assert_eq!(next_gen, 1);
    assert!(cancellation.is_cancelled(0));
    assert!(!cancellation.is_cancelled(1));
    assert!(
        elapsed.as_micros() < 2000,
        "Cancellation advance took {} µs, expected < 2000 µs",
        elapsed.as_micros()
    );
}

#[test]
fn test_search_history_persistence() {
    let mut history = SearchHistory::default();

    history.push_file_query("main.rs");
    history.push_file_query("Cargo.toml");
    history.push_file_query("main.rs"); // duplicate should move to front

    assert_eq!(history.file_queries.len(), 2);
    assert_eq!(history.file_queries[0], "main.rs");
    assert_eq!(history.file_queries[1], "Cargo.toml");

    history.push_content_query("fn run");
    history.push_content_query("struct State");
    assert_eq!(history.content_queries.len(), 2);
    assert_eq!(history.content_queries[0], "struct State");
    assert_eq!(history.content_queries[1], "fn run");

    history.clear();
    assert!(history.file_queries.is_empty());
    assert!(history.content_queries.is_empty());
}
