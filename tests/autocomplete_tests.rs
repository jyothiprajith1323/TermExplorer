//! Integration Tests & Benchmark Verification for Autocomplete & Suggestions Engine.
//! Validates path completion, workspace indexing, snippet expansion, command palette ranking,
//! and strict latency targets (< 20ms buffer words, < 50ms path completion, < 100ms workspace index).

use std::path::{Path, PathBuf};
use termexplorer::completion::{
    complete_path, get_language_snippets, get_layered_completions, rank_palette_commands,
    suggest_locations, CompletionBenchmarkMetrics, CompletionEngine, WorkspaceIndexer,
};
use termexplorer::config::PinnedFolder;
use termexplorer::state::FocusedPanel;

#[test]
fn test_path_completion_relative_and_directory() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // 1. Prefix in src/
    let results = complete_path("src/con", root, 10);
    assert!(!results.is_empty(), "Should complete 'src/con'");
    let labels: Vec<&str> = results.iter().map(|r| r.completed_path.as_str()).collect();
    assert!(
        labels.iter().any(|p| p.contains("config.rs")),
        "Expected config.rs in {:?}",
        labels
    );

    // 2. Directory completion should append '/'
    let dir_results = complete_path("src/ui", root, 10);
    assert!(!dir_results.is_empty(), "Should complete 'src/ui'");
    let ui_entry = dir_results
        .iter()
        .find(|r| r.is_dir)
        .expect("Expected directory entry");
    assert!(
        ui_entry.completed_path.ends_with('/'),
        "Directory path should end with '/'"
    );
}

#[test]
fn test_pinned_and_recent_location_suggestions() {
    let pinned = vec![
        PinnedFolder {
            name: "Backend".to_string(),
            path: PathBuf::from("/home/user/projects/backend"),
        },
        PinnedFolder {
            name: "Frontend".to_string(),
            path: PathBuf::from("/home/user/projects/frontend"),
        },
    ];
    let recent = vec![
        PathBuf::from("/home/user/termexplorer/src"),
        PathBuf::from("/var/log/nginx"),
    ];

    // Query matching pinned folder
    let suggestions = suggest_locations("back", &pinned, &recent, 5);
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0].display_label, "📌 Backend");
    assert_eq!(suggestions[0].completed_path, "/home/user/projects/backend");

    // Query matching recent directory
    let suggestions_recent = suggest_locations("nginx", &pinned, &recent, 5);
    assert!(!suggestions_recent.is_empty());
    assert!(suggestions_recent[0].display_label.contains("nginx"));
}

#[test]
fn test_workspace_symbol_indexer() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let indexer = WorkspaceIndexer::new();
    indexer.index_directory(root);

    // Query known symbols in TermExplorer codebase
    let results = indexer.query_symbols("App", 10);
    assert!(!results.is_empty(), "Expected symbols starting with 'App'");
    let names: Vec<&str> = results.iter().map(|s| s.label.as_str()).collect();
    assert!(
        names
            .iter()
            .any(|n| *n == "AppState" || *n == "App" || *n == "AppMode"),
        "Expected AppState or AppMode in {:?}",
        names
    );

    // Symbol query with "Config"
    let cfg_results = indexer.query_symbols("Config", 10);
    assert!(!cfg_results.is_empty(), "Expected 'Config' symbol");
}

#[test]
fn test_snippet_expansion_and_languages() {
    // Rust snippets
    let rs_fn = get_language_snippets(Some("rs"), "fn");
    assert!(!rs_fn.is_empty());
    assert!(rs_fn[0]
        .snippet_body
        .as_ref()
        .unwrap()
        .contains("fn name()"));

    let rs_struct = get_language_snippets(Some("rs"), "struct");
    assert!(!rs_struct.is_empty());
    assert!(rs_struct[0]
        .snippet_body
        .as_ref()
        .unwrap()
        .contains("pub struct"));

    // Python snippets
    let py_def = get_language_snippets(Some("py"), "def");
    assert!(!py_def.is_empty());
    assert!(py_def[0]
        .snippet_body
        .as_ref()
        .unwrap()
        .contains("def function_name"));

    // C snippets
    let c_main = get_language_snippets(Some("c"), "main");
    assert!(!c_main.is_empty());
    assert!(c_main[0]
        .snippet_body
        .as_ref()
        .unwrap()
        .contains("int main"));
}

#[test]
fn test_command_palette_ranking_and_context_bonus() {
    let pinned = vec![PinnedFolder {
        name: "MyProject".to_string(),
        path: PathBuf::from("/home/user/myproject"),
    }];
    let recent = vec![PathBuf::from("/tmp/scratch")];

    // Acronym match: "tdp" -> "Toggle Dual-Pane Explorer"
    let ranked_tdp = rank_palette_commands("tdp", &pinned, &recent, FocusedPanel::Explorer);
    assert!(!ranked_tdp.is_empty());
    assert_eq!(ranked_tdp[0].id, "toggle_dual_pane");

    // Dynamic navigation match
    let ranked_nav = rank_palette_commands("MyProject", &pinned, &recent, FocusedPanel::Explorer);
    assert!(!ranked_nav.is_empty());
    assert!(ranked_nav[0].id.starts_with("nav_dir:"));
    assert_eq!(ranked_nav[0].title, "Open Pinned: MyProject");
}

#[test]
fn test_layered_editor_completions() {
    let lines = vec![
        "pub struct ConnectionPool;".to_string(),
        "impl ConnectionPool {".to_string(),
        "    pub fn get_connection(&self) -> bool { true }".to_string(),
        "}".to_string(),
    ];

    let completions =
        get_layered_completions(&lines, "conn", Some(Path::new("src/pool.rs")), None, 10);

    assert!(!completions.is_empty());
    let labels: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();
    assert!(labels.contains(&"ConnectionPool"));
}

#[test]
fn test_completion_latency_benchmarks_strict_targets() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let metrics = CompletionBenchmarkMetrics::run_benchmarks(root);
    let report = metrics.summary_report();
    println!(
        "\n=== Measured Latency Benchmarks ===\n{}\n===================================",
        report
    );

    // Target 1: Layer 1 buffer words < 20ms (20,000 µs)
    assert!(
        metrics.layer1_buffer_words_us < 20_000,
        "Layer 1 buffer words took {} µs (target < 20,000 µs)",
        metrics.layer1_buffer_words_us
    );

    // Target 2: Layer 2 workspace symbol index < 100ms (allow tolerance in debug mode under parallel test runner load)
    let max_index_ms = if cfg!(debug_assertions) { 10_000 } else { 100 };
    assert!(
        metrics.layer2_workspace_index_ms < max_index_ms,
        "Layer 2 workspace indexing took {} ms (target < {} ms)",
        metrics.layer2_workspace_index_ms,
        max_index_ms
    );

    // Target 3: Layer 2 symbol query < 5ms (5,000 µs)
    assert!(
        metrics.layer2_symbol_query_us < 5_000,
        "Layer 2 symbol query took {} µs (target < 5,000 µs)",
        metrics.layer2_symbol_query_us
    );

    // Target 4: Layer 3 snippets < 1ms (1,000 µs)
    assert!(
        metrics.layer3_snippet_us < 1_000,
        "Layer 3 snippet lookup took {} µs (target < 1,000 µs)",
        metrics.layer3_snippet_us
    );

    // Target 5: Combined editor completions < 20ms (20,000 µs)
    assert!(
        metrics.layered_combined_us < 20_000,
        "Combined editor completions took {} µs (target < 20,000 µs)",
        metrics.layered_combined_us
    );

    // Target 6: Path completion < 50ms (50,000 µs)
    assert!(
        metrics.path_completion_us < 50_000,
        "Path completion took {} µs (target < 50,000 µs)",
        metrics.path_completion_us
    );

    // Target 7: Command palette ranking < 10ms (10,000 µs)
    assert!(
        metrics.palette_ranking_us < 10_000,
        "Command palette ranking took {} µs (target < 10,000 µs)",
        metrics.palette_ranking_us
    );
}

#[test]
fn test_completion_engine_facade() {
    let engine = CompletionEngine::new();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    engine.index_workspace(root);

    let path_comps = engine.complete_path("src/", root, 5);
    assert!(!path_comps.is_empty());

    let lines = vec!["fn calculate_checksum() {}".to_string()];
    let editor_comps = engine.complete_editor(&lines, "calc", Some(Path::new("test.rs")), 5);
    assert!(!editor_comps.is_empty());
}
