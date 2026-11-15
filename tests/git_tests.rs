//! Integration tests for Native Git Integration (Sprint 4B).

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;
use termexplorer::git::{branch, commit, diff, history, status, GitFileStatus};
use termexplorer::theme::Theme;

/// Helper to set up an isolated, temporary Git repository with author configuration.
fn setup_temp_repo() -> (TempDir, PathBuf) {
    let temp = TempDir::new().expect("Failed to create tempdir");
    let repo_dir = temp.path().to_path_buf();

    Command::new("git")
        .current_dir(&repo_dir)
        .args(["init", "-b", "master"])
        .output()
        .expect("git init failed");

    Command::new("git")
        .current_dir(&repo_dir)
        .args(["config", "user.name", "TermExplorer Tester"])
        .output()
        .expect("git config user.name failed");

    Command::new("git")
        .current_dir(&repo_dir)
        .args(["config", "user.email", "tester@termexplorer.local"])
        .output()
        .expect("git config user.email failed");

    (temp, repo_dir)
}

#[test]
fn test_git_status_porcelain_v2_parsing_and_lifecycle() {
    let (_temp, repo_dir) = setup_temp_repo();

    // Initially clean repository
    let st = status::query_status(&repo_dir);
    assert!(st.is_repo);
    assert_eq!(st.branch, "master");
    assert!(!st.is_dirty);
    assert!(st.staged_files.is_empty());
    assert!(st.unstaged_files.is_empty());
    assert!(st.untracked_files.is_empty());

    // 1. Create untracked file
    let file1 = repo_dir.join("hello.txt");
    fs::write(&file1, "Line 1\nLine 2\n").unwrap();

    let st = status::query_status(&repo_dir);
    assert!(st.is_dirty);
    assert_eq!(st.untracked_files.len(), 1);
    assert_eq!(st.untracked_files[0].path, "hello.txt");
    assert_eq!(st.untracked_files[0].status, GitFileStatus::Untracked);

    // 2. Stage file
    assert!(status::stage_file(&repo_dir, "hello.txt"));
    let st = status::query_status(&repo_dir);
    assert_eq!(st.staged_files.len(), 1);
    assert_eq!(st.staged_files[0].path, "hello.txt");
    assert_eq!(st.staged_files[0].status, GitFileStatus::Staged);
    assert!(st.untracked_files.is_empty());

    // 3. Unstage file
    assert!(status::unstage_file(&repo_dir, "hello.txt"));
    let st = status::query_status(&repo_dir);
    assert!(st.staged_files.is_empty());
    assert_eq!(st.untracked_files.len(), 1);

    // 4. Stage all and commit
    assert!(status::stage_all(&repo_dir));
    let commit_res = commit::create_commit(&repo_dir, "Initial commit");
    assert!(commit_res.is_ok());

    let st = status::query_status(&repo_dir);
    assert!(!st.is_dirty);
    assert_eq!(st.recent_commits.len(), 1);
    assert_eq!(st.recent_commits[0].message, "Initial commit");

    // 5. Modify committed file
    fs::write(&file1, "Line 1 modified\nLine 2\nLine 3 added\n").unwrap();
    let st = status::query_status(&repo_dir);
    assert!(st.is_dirty);
    assert_eq!(st.unstaged_files.len(), 1);
    assert_eq!(st.unstaged_files[0].status, GitFileStatus::Modified);
}

#[test]
fn test_git_unified_diff_generation_and_parsing() {
    let (_temp, repo_dir) = setup_temp_repo();

    let file = repo_dir.join("code.rs");
    fs::write(&file, "fn main() {\n    println!(\"hello\");\n}\n").unwrap();
    status::stage_all(&repo_dir);
    commit::create_commit(&repo_dir, "Initial code").unwrap();

    // Modify file
    fs::write(
        &file,
        "fn main() {\n    println!(\"hello world\");\n    let x = 42;\n}\n",
    )
    .unwrap();

    // Get working tree diff
    let raw_diff = diff::get_file_diff(&repo_dir, "code.rs", false).unwrap();
    assert!(raw_diff.contains("println!(\"hello world\");"));
    assert!(raw_diff.contains("let x = 42;"));

    // Parse diff into structured model
    let parsed_files = diff::parse_unified_diff(&raw_diff);
    assert_eq!(parsed_files.len(), 1);
    assert_eq!(parsed_files[0].new_path, "code.rs");
    assert!(!parsed_files[0].hunks.is_empty());

    // Render diff to Ratatui lines
    let theme = Theme::default_dark();
    let rendered_lines = diff::render_diff_lines(&raw_diff, &theme);
    assert!(!rendered_lines.is_empty());

    // Check that addition lines contain '+' and deletion lines contain '-'
    let has_add = rendered_lines
        .iter()
        .any(|l| l.spans.iter().any(|s| s.content.starts_with('+')));
    let has_del = rendered_lines
        .iter()
        .any(|l| l.spans.iter().any(|s| s.content.starts_with('-')));
    assert!(has_add, "Diff rendering should include addition lines");
    assert!(has_del, "Diff rendering should include deletion lines");
}

#[test]
fn test_git_branch_operations() {
    let (_temp, repo_dir) = setup_temp_repo();

    // Commit a file so master has a HEAD
    let file = repo_dir.join("README.md");
    fs::write(&file, "# Test Repo\n").unwrap();
    status::stage_all(&repo_dir);
    commit::create_commit(&repo_dir, "Initial commit").unwrap();

    // List initial branches
    let branches = branch::list_branches(&repo_dir).unwrap();
    assert!(!branches.is_empty());
    assert!(branches.iter().any(|b| b.name == "master" && b.is_current));

    // Create and switch to new branch
    let switch_res = branch::create_and_switch_branch(&repo_dir, "feature/awesome");
    assert!(switch_res.is_ok());

    let branches = branch::list_branches(&repo_dir).unwrap();
    assert!(branches
        .iter()
        .any(|b| b.name == "feature/awesome" && b.is_current));

    // Switch back to master
    let switch_back = branch::switch_branch(&repo_dir, "master");
    assert!(switch_back.is_ok());

    let branches = branch::list_branches(&repo_dir).unwrap();
    assert!(branches.iter().any(|b| b.name == "master" && b.is_current));

    // Delete feature branch
    let del_res = branch::delete_branch(&repo_dir, "feature/awesome", false);
    assert!(del_res.is_ok());

    let branches = branch::list_branches(&repo_dir).unwrap();
    assert!(!branches.iter().any(|b| b.name == "feature/awesome"));
}

#[test]
fn test_git_history_and_file_blame() {
    let (_temp, repo_dir) = setup_temp_repo();

    let file = repo_dir.join("blame_test.txt");
    fs::write(&file, "First line\n").unwrap();
    status::stage_all(&repo_dir);
    commit::create_commit(&repo_dir, "Commit one").unwrap();

    fs::write(&file, "First line\nSecond line\n").unwrap();
    status::stage_all(&repo_dir);
    commit::create_commit(&repo_dir, "Commit two").unwrap();

    // Retrieve commit log
    let history_list = history::get_commit_history(&repo_dir, 10, 0).unwrap();
    assert_eq!(history_list.len(), 2);
    assert_eq!(history_list[0].message, "Commit two");
    assert_eq!(history_list[1].message, "Commit one");

    // Retrieve file blame
    let blame_lines = history::get_file_blame(&repo_dir, &file).unwrap();
    assert_eq!(blame_lines.len(), 2);
    assert_eq!(blame_lines[0].line_num, 1);
    assert!(blame_lines[0].content.contains("First line"));
    assert_eq!(blame_lines[1].line_num, 2);
    assert!(blame_lines[1].content.contains("Second line"));
}

#[tokio::test]
async fn test_git_manager_async_non_blocking_operations() {
    use termexplorer::git::GitOperationManager;

    let (_temp, repo_dir) = setup_temp_repo();

    let file = repo_dir.join("async_test.txt");
    fs::write(&file, "Async content\n").unwrap();

    // Stage asynchronously
    let staged = GitOperationManager::stage_file(repo_dir.clone(), "async_test.txt".into()).await;
    assert!(staged);

    // Commit asynchronously
    let commit_res =
        GitOperationManager::create_commit(repo_dir.clone(), "Async commit message".into()).await;
    assert!(commit_res.is_ok());

    // Query status asynchronously
    let st = GitOperationManager::query_status(repo_dir.clone()).await;
    assert!(st.is_repo);
    assert!(!st.is_dirty);
    assert_eq!(st.recent_commits[0].message, "Async commit message");

    // List branches asynchronously
    let branches = GitOperationManager::list_branches(repo_dir.clone()).await;
    assert!(branches.is_ok());
    assert_eq!(branches.unwrap()[0].name, "master");
}
