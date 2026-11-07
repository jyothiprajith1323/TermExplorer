use std::fs::File;
use tempfile::tempdir;
use termexplorer::commands::CommandRegistry;
use termexplorer::git::GitRepoStatus;
use termexplorer::workspace::{detect_project_kind, ProjectKind, Workspace};

#[test]
fn test_project_kind_detection() {
    let dir = tempdir().unwrap();

    // 1. Rust Cargo project
    File::create(dir.path().join("Cargo.toml")).unwrap();
    assert_eq!(detect_project_kind(dir.path()), ProjectKind::Rust);

    // 2. Python project
    let py_dir = tempdir().unwrap();
    File::create(py_dir.path().join("pyproject.toml")).unwrap();
    assert_eq!(detect_project_kind(py_dir.path()), ProjectKind::Python);

    // 3. Node.js project
    let node_dir = tempdir().unwrap();
    File::create(node_dir.path().join("package.json")).unwrap();
    assert_eq!(detect_project_kind(node_dir.path()), ProjectKind::NodeJs);

    // 4. Go project
    let go_dir = tempdir().unwrap();
    File::create(go_dir.path().join("go.mod")).unwrap();
    assert_eq!(detect_project_kind(go_dir.path()), ProjectKind::Go);
}

#[test]
fn test_workspace_quick_actions() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("Cargo.toml")).unwrap();

    let ws = Workspace::from_dir(dir.path());
    assert_eq!(ws.project_kind, ProjectKind::Rust);

    let actions = ws.quick_actions();
    assert!(!actions.is_empty());
    assert!(actions.iter().any(|a| a.command.contains("cargo test")));
    assert!(actions.iter().any(|a| a.command.contains("cargo build")));
}

#[test]
fn test_command_palette_search() {
    let all = CommandRegistry::all_commands();
    assert!(all.len() >= 15);

    // Exact and fuzzy search
    let results = CommandRegistry::search("dual");
    assert!(!results.is_empty());
    assert!(results.iter().any(|c| c.id == "toggle_dual_pane"));

    let theme_results = CommandRegistry::search("dracula");
    assert!(!theme_results.is_empty());
    assert_eq!(theme_results[0].id, "theme_dracula");
}

#[test]
fn test_git_repo_query() {
    // Current workspace /home/prajith/TermExplorer is a valid git repository
    let curr_dir = std::path::Path::new("/home/prajith/TermExplorer");
    let status = GitRepoStatus::query(curr_dir);

    assert!(status.is_repo);
    assert!(!status.branch.is_empty());
    assert!(status.repo_root.is_some());
}
