//! Asynchronous Git operations manager ensuring zero TUI thread freezing.

use crate::git::{
    branch, commit, diff, history, status,
    types::{GitBlameLine, GitBranch, GitCommitInfo, GitRepoStatus},
};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::timeout;

/// Timeout for local Git CLI operations to prevent any UI lockups.
pub const GIT_COMMAND_TIMEOUT: Duration = Duration::from_secs(5);

/// Coordinates asynchronous execution of Git commands via Tokio blocking threads.
pub struct GitOperationManager;

impl GitOperationManager {
    /// Query status asynchronously with bounded timeout.
    pub async fn query_status(dir: PathBuf) -> GitRepoStatus {
        let task = tokio::task::spawn_blocking(move || status::query_status(&dir));
        match timeout(GIT_COMMAND_TIMEOUT, task).await {
            Ok(Ok(st)) => st,
            _ => GitRepoStatus::empty(),
        }
    }

    /// Stage a file asynchronously.
    pub async fn stage_file(dir: PathBuf, file: String) -> bool {
        let task = tokio::task::spawn_blocking(move || status::stage_file(&dir, &file));
        match timeout(GIT_COMMAND_TIMEOUT, task).await {
            Ok(Ok(success)) => success,
            _ => false,
        }
    }

    /// Unstage a file asynchronously.
    pub async fn unstage_file(dir: PathBuf, file: String) -> bool {
        let task = tokio::task::spawn_blocking(move || status::unstage_file(&dir, &file));
        match timeout(GIT_COMMAND_TIMEOUT, task).await {
            Ok(Ok(success)) => success,
            _ => false,
        }
    }

    /// Stage all modified & untracked files asynchronously.
    pub async fn stage_all(dir: PathBuf) -> bool {
        let task = tokio::task::spawn_blocking(move || status::stage_all(&dir));
        match timeout(GIT_COMMAND_TIMEOUT, task).await {
            Ok(Ok(success)) => success,
            _ => false,
        }
    }

    /// Unstage all staged files asynchronously.
    pub async fn unstage_all(dir: PathBuf) -> bool {
        let task = tokio::task::spawn_blocking(move || status::unstage_all(&dir));
        match timeout(GIT_COMMAND_TIMEOUT, task).await {
            Ok(Ok(success)) => success,
            _ => false,
        }
    }

    /// Commit staged changes asynchronously.
    pub async fn create_commit(dir: PathBuf, message: String) -> Result<String, String> {
        let task = tokio::task::spawn_blocking(move || commit::create_commit(&dir, &message));
        match timeout(GIT_COMMAND_TIMEOUT, task).await {
            Ok(Ok(res)) => res,
            Ok(Err(join_err)) => Err(format!("Task execution panicked: {join_err}")),
            Err(_) => Err("Git commit timed out after 5 seconds".to_string()),
        }
    }

    /// Fetch unified diff asynchronously.
    pub async fn get_diff(
        dir: PathBuf,
        file: Option<String>,
        staged: bool,
    ) -> Result<String, String> {
        let task = tokio::task::spawn_blocking(move || match file {
            Some(f) => diff::get_file_diff(&dir, &f, staged),
            None => diff::get_working_tree_diff(&dir, staged),
        });
        match timeout(GIT_COMMAND_TIMEOUT, task).await {
            Ok(Ok(res)) => res,
            Ok(Err(join_err)) => Err(format!("Task execution panicked: {join_err}")),
            Err(_) => Err("Git diff timed out".to_string()),
        }
    }

    /// Discover branches asynchronously.
    pub async fn list_branches(dir: PathBuf) -> Result<Vec<GitBranch>, String> {
        let task = tokio::task::spawn_blocking(move || branch::list_branches(&dir));
        match timeout(GIT_COMMAND_TIMEOUT, task).await {
            Ok(Ok(res)) => res,
            Ok(Err(join_err)) => Err(format!("Task execution panicked: {join_err}")),
            Err(_) => Err("Git branch listing timed out".to_string()),
        }
    }

    /// Switch branch asynchronously.
    pub async fn switch_branch(dir: PathBuf, branch_name: String) -> Result<String, String> {
        let task = tokio::task::spawn_blocking(move || branch::switch_branch(&dir, &branch_name));
        match timeout(GIT_COMMAND_TIMEOUT, task).await {
            Ok(Ok(res)) => res,
            Ok(Err(join_err)) => Err(format!("Task execution panicked: {join_err}")),
            Err(_) => Err("Git switch branch timed out".to_string()),
        }
    }

    /// Create and switch branch asynchronously.
    pub async fn create_branch(dir: PathBuf, branch_name: String) -> Result<String, String> {
        let task = tokio::task::spawn_blocking(move || {
            branch::create_and_switch_branch(&dir, &branch_name)
        });
        match timeout(GIT_COMMAND_TIMEOUT, task).await {
            Ok(Ok(res)) => res,
            Ok(Err(join_err)) => Err(format!("Task execution panicked: {join_err}")),
            Err(_) => Err("Git create branch timed out".to_string()),
        }
    }

    /// Fetch commit history asynchronously.
    pub async fn get_history(
        dir: PathBuf,
        max_count: usize,
        skip: usize,
    ) -> Result<Vec<GitCommitInfo>, String> {
        let task =
            tokio::task::spawn_blocking(move || history::get_commit_history(&dir, max_count, skip));
        match timeout(GIT_COMMAND_TIMEOUT, task).await {
            Ok(Ok(res)) => res,
            Ok(Err(join_err)) => Err(format!("Task execution panicked: {join_err}")),
            Err(_) => Err("Git log timed out".to_string()),
        }
    }

    /// Fetch file blame annotations asynchronously.
    pub async fn get_blame(dir: PathBuf, file_path: PathBuf) -> Result<Vec<GitBlameLine>, String> {
        let task = tokio::task::spawn_blocking(move || history::get_file_blame(&dir, &file_path));
        match timeout(GIT_COMMAND_TIMEOUT, task).await {
            Ok(Ok(res)) => res,
            Ok(Err(join_err)) => Err(format!("Task execution panicked: {join_err}")),
            Err(_) => Err("Git blame timed out".to_string()),
        }
    }
}
