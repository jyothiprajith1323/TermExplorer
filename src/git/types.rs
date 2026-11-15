//! Git domain types, status enums, branch metadata, diff abstractions, and blame models.

use std::path::PathBuf;

/// Status of an individual file in the Git working tree or index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitFileStatus {
    Staged,
    Modified,
    Untracked,
    Deleted,
    Renamed,
    Copied,
    Conflicted,
    TypeChanged,
}

impl GitFileStatus {
    pub fn badge(&self) -> &'static str {
        match self {
            Self::Staged => "[A]",
            Self::Modified => "[M]",
            Self::Untracked => "[?]",
            Self::Deleted => "[D]",
            Self::Renamed => "[R]",
            Self::Copied => "[C]",
            Self::Conflicted => "[!]",
            Self::TypeChanged => "[T]",
        }
    }
}

/// A changed file entry in the Git working tree or index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitFileEntry {
    pub path: String,
    pub status: GitFileStatus,
    pub raw_code: String,
}

impl GitFileEntry {
    pub fn new(path: String, status: GitFileStatus, raw_code: String) -> Self {
        Self {
            path,
            status,
            raw_code,
        }
    }
}

/// Metadata for a Git branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitBranch {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    pub commit_hash: String,
    pub commit_summary: String,
}

/// Commit summary information from git log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitCommitInfo {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub author_email: String,
    pub date: String,
    pub message: String,
}

/// Single line annotation from git blame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitBlameLine {
    pub line_num: usize,
    pub commit_hash: String,
    pub author: String,
    pub date: String,
    pub content: String,
}

/// Classification of a line in a unified diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineKind {
    Header,
    HunkHeader,
    Context,
    Addition,
    Deletion,
}

/// A line in a unified diff with optional old and new line numbers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitDiffLine {
    pub kind: DiffLineKind,
    pub old_lineno: Option<usize>,
    pub new_lineno: Option<usize>,
    pub text: String,
}

/// A diff hunk within a file diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitDiffHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub header: String,
    pub lines: Vec<GitDiffLine>,
}

/// Diff representation for a single modified file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitDiffFile {
    pub old_path: String,
    pub new_path: String,
    pub is_binary: bool,
    pub hunks: Vec<GitDiffHunk>,
}

/// Complete repository status snapshot.
#[derive(Debug, Clone, Default)]
pub struct GitRepoStatus {
    pub is_repo: bool,
    pub repo_root: Option<PathBuf>,
    pub branch: String,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    pub is_dirty: bool,
    pub staged_files: Vec<GitFileEntry>,
    pub unstaged_files: Vec<GitFileEntry>,
    pub untracked_files: Vec<GitFileEntry>,
    pub conflicted_files: Vec<GitFileEntry>,
    pub recent_commits: Vec<GitCommitInfo>,
}

impl GitRepoStatus {
    /// Return an empty, non-repository status.
    pub fn empty() -> Self {
        Self {
            is_repo: false,
            repo_root: None,
            branch: String::new(),
            upstream: None,
            ahead: 0,
            behind: 0,
            is_dirty: false,
            staged_files: Vec::new(),
            unstaged_files: Vec::new(),
            untracked_files: Vec::new(),
            conflicted_files: Vec::new(),
            recent_commits: Vec::new(),
        }
    }
}
