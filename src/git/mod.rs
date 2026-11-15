//! Git integration: repository detection, branch status, staging, unified diffs, history, and commits.

pub mod branch;
pub mod commit;
pub mod diff;
pub mod history;
pub mod manager;
pub mod status;
pub mod types;

pub use branch::{create_and_switch_branch, delete_branch, list_branches, switch_branch};
pub use commit::{amend_commit, create_commit};
pub use diff::{
    get_commit_diff, get_file_diff, get_working_tree_diff, parse_unified_diff, render_diff_lines,
};
pub use history::{get_commit_history, get_file_blame};
pub use manager::GitOperationManager;
pub use status::{
    query_recent_commits, query_status, stage_all, stage_file, unstage_all, unstage_file,
};
pub use types::{
    DiffLineKind, GitBlameLine, GitBranch, GitCommitInfo, GitDiffFile, GitDiffHunk, GitDiffLine,
    GitFileEntry, GitFileStatus, GitRepoStatus,
};

impl GitRepoStatus {
    /// Query Git status for directory synchronously.
    pub fn query(dir: &std::path::Path) -> Self {
        status::query_status(dir)
    }

    /// Stage a file (`git add <file>`).
    pub fn stage_file(dir: &std::path::Path, file: &str) -> bool {
        status::stage_file(dir, file)
    }

    /// Unstage a file (`git restore --staged <file>`).
    pub fn unstage_file(dir: &std::path::Path, file: &str) -> bool {
        status::unstage_file(dir, file)
    }

    /// Commit staged changes with message.
    pub fn commit(dir: &std::path::Path, message: &str) -> Result<String, String> {
        commit::create_commit(dir, message)
    }
}
