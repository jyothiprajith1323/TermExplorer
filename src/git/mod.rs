//! Git integration: repository detection, branch status, staging, and commits.

use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitFileStatus {
    Staged,
    Modified,
    Untracked,
    Deleted,
    Renamed,
}

#[derive(Debug, Clone)]
pub struct GitFileEntry {
    pub path: String,
    pub status: GitFileStatus,
    pub raw_code: String,
}

#[derive(Debug, Clone)]
pub struct GitCommitInfo {
    pub hash: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct GitRepoStatus {
    pub is_repo: bool,
    pub repo_root: Option<PathBuf>,
    pub branch: String,
    pub is_dirty: bool,
    pub staged_files: Vec<GitFileEntry>,
    pub unstaged_files: Vec<GitFileEntry>,
    pub untracked_files: Vec<GitFileEntry>,
    pub recent_commits: Vec<GitCommitInfo>,
}

impl GitRepoStatus {
    pub fn empty() -> Self {
        Self {
            is_repo: false,
            repo_root: None,
            branch: String::new(),
            is_dirty: false,
            staged_files: Vec::new(),
            unstaged_files: Vec::new(),
            untracked_files: Vec::new(),
            recent_commits: Vec::new(),
        }
    }

    /// Query git CLI for repository status at the given directory.
    pub fn query(dir: &Path) -> Self {
        // 1. Check if inside work tree
        let Ok(out) = Command::new("git")
            .current_dir(dir)
            .args(["rev-parse", "--is-inside-work-tree"])
            .output()
        else {
            return Self::empty();
        };

        if !out.status.success()
            || !String::from_utf8_lossy(&out.stdout)
                .trim()
                .starts_with("true")
        {
            return Self::empty();
        }

        // 2. Get repo root
        let repo_root = Command::new("git")
            .current_dir(dir)
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()))
                } else {
                    None
                }
            });

        // 3. Current branch
        let branch = Command::new("git")
            .current_dir(dir)
            .args(["branch", "--show-current"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|| "HEAD".to_string());

        let branch = if branch.is_empty() {
            "detached".to_string()
        } else {
            branch
        };

        // 4. Status porcelain
        let mut staged_files = Vec::new();
        let mut unstaged_files = Vec::new();
        let mut untracked_files = Vec::new();

        if let Ok(out) = Command::new("git")
            .current_dir(dir)
            .args(["status", "--porcelain=v1"])
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                if line.len() < 3 {
                    continue;
                }
                let x = line.chars().next().unwrap_or(' ');
                let y = line.chars().nth(1).unwrap_or(' ');
                let file_path = line[3..].trim().to_string();

                if x == '?' && y == '?' {
                    untracked_files.push(GitFileEntry {
                        path: file_path,
                        status: GitFileStatus::Untracked,
                        raw_code: "??".to_string(),
                    });
                } else {
                    // Staged index status
                    if x != ' ' && x != '?' {
                        staged_files.push(GitFileEntry {
                            path: file_path.clone(),
                            status: match x {
                                'A' => GitFileStatus::Staged,
                                'D' => GitFileStatus::Deleted,
                                'R' => GitFileStatus::Renamed,
                                _ => GitFileStatus::Modified,
                            },
                            raw_code: format!("{} ", x),
                        });
                    }
                    // Working tree status
                    if y != ' ' && y != '?' {
                        unstaged_files.push(GitFileEntry {
                            path: file_path,
                            status: match y {
                                'D' => GitFileStatus::Deleted,
                                _ => GitFileStatus::Modified,
                            },
                            raw_code: format!(" {}", y),
                        });
                    }
                }
            }
        }

        let is_dirty =
            !staged_files.is_empty() || !unstaged_files.is_empty() || !untracked_files.is_empty();

        // 5. Recent commits log
        let mut recent_commits = Vec::new();
        if let Ok(out) = Command::new("git")
            .current_dir(dir)
            .args(["log", "-n", "5", "--oneline"])
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                let mut parts = line.splitn(2, ' ');
                if let (Some(h), Some(msg)) = (parts.next(), parts.next()) {
                    recent_commits.push(GitCommitInfo {
                        hash: h.to_string(),
                        message: msg.to_string(),
                    });
                }
            }
        }

        Self {
            is_repo: true,
            repo_root,
            branch,
            is_dirty,
            staged_files,
            unstaged_files,
            untracked_files,
            recent_commits,
        }
    }

    /// Stage a file (`git add <file>`).
    pub fn stage_file(dir: &Path, file: &str) -> bool {
        Command::new("git")
            .current_dir(dir)
            .args(["add", "--", file])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Unstage a file (`git restore --staged <file>`).
    pub fn unstage_file(dir: &Path, file: &str) -> bool {
        Command::new("git")
            .current_dir(dir)
            .args(["restore", "--staged", "--", file])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Commit staged changes with message.
    pub fn commit(dir: &Path, message: &str) -> Result<String, String> {
        let out = Command::new("git")
            .current_dir(dir)
            .args(["commit", "-m", message])
            .output()
            .map_err(|e| e.to_string())?;

        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
        }
    }
}
