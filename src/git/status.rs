//! Git status extraction via porcelain v2 parsing with NUL delimiter support.

use crate::git::types::{GitCommitInfo, GitFileEntry, GitFileStatus, GitRepoStatus};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Query the Git CLI for a complete status snapshot at the specified directory.
pub fn query_status(dir: &Path) -> GitRepoStatus {
    // 1. Verify inside Git work tree
    let Ok(out) = Command::new("git")
        .current_dir(dir)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
    else {
        return GitRepoStatus::empty();
    };

    if !out.status.success()
        || !String::from_utf8_lossy(&out.stdout)
            .trim()
            .starts_with("true")
    {
        return GitRepoStatus::empty();
    }

    // 2. Discover repository root directory
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

    // 3. Query porcelain v2 status with branch info and NUL separation
    let mut parsed = ParsedPorcelainStatus {
        branch: "HEAD".to_string(),
        ..Default::default()
    };

    if let Ok(out) = Command::new("git")
        .current_dir(dir)
        .args(["status", "--porcelain=v2", "--branch", "-z"])
        .output()
    {
        if out.status.success() {
            parse_porcelain_v2_nul(&out.stdout, &mut parsed);
        } else {
            // Fallback to porcelain v1 if v2 fails
            fallback_porcelain_v1(dir, &mut parsed);
        }
    }

    let is_dirty = !parsed.staged.is_empty()
        || !parsed.unstaged.is_empty()
        || !parsed.untracked.is_empty()
        || !parsed.conflicted.is_empty();

    // 4. Query recent commits log
    let recent_commits = query_recent_commits(dir, 10);

    GitRepoStatus {
        is_repo: true,
        repo_root,
        branch: parsed.branch,
        upstream: parsed.upstream,
        ahead: parsed.ahead,
        behind: parsed.behind,
        is_dirty,
        staged_files: parsed.staged,
        unstaged_files: parsed.unstaged,
        untracked_files: parsed.untracked,
        conflicted_files: parsed.conflicted,
        recent_commits,
    }
}

#[derive(Default)]
struct ParsedPorcelainStatus {
    branch: String,
    upstream: Option<String>,
    ahead: usize,
    behind: usize,
    staged: Vec<GitFileEntry>,
    unstaged: Vec<GitFileEntry>,
    untracked: Vec<GitFileEntry>,
    conflicted: Vec<GitFileEntry>,
}

/// Parse `git status --porcelain=v2 --branch -z` raw bytes.
fn parse_porcelain_v2_nul(stdout: &[u8], parsed: &mut ParsedPorcelainStatus) {
    let mut entries = stdout.split(|&b| b == 0);

    while let Some(chunk) = entries.next() {
        if chunk.is_empty() {
            continue;
        }

        let line = String::from_utf8_lossy(chunk);

        if let Some(stripped) = line.strip_prefix("# branch.head ") {
            let b = stripped.trim();
            parsed.branch = if b == "(detached)" {
                "detached".to_string()
            } else {
                b.to_string()
            };
        } else if let Some(stripped) = line.strip_prefix("# branch.upstream ") {
            parsed.upstream = Some(stripped.trim().to_string());
        } else if let Some(stripped) = line.strip_prefix("# branch.ab ") {
            let ab = stripped.trim();
            for part in ab.split_whitespace() {
                if let Some(num_str) = part.strip_prefix('+') {
                    if let Ok(n) = num_str.parse::<usize>() {
                        parsed.ahead = n;
                    }
                } else if let Some(num_str) = part.strip_prefix('-') {
                    if let Ok(n) = num_str.parse::<usize>() {
                        parsed.behind = n;
                    }
                }
            }
        } else if line.starts_with("1 ") {
            // Ordinary change: 1 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <path>
            let parts: Vec<&str> = line.splitn(9, ' ').collect();
            if parts.len() >= 9 {
                let xy = parts[1];
                let path = parts[8].to_string();
                let x = xy.chars().next().unwrap_or('.');
                let y = xy.chars().nth(1).unwrap_or('.');

                // Staged entry
                if x != '.' {
                    parsed.staged.push(GitFileEntry {
                        path: path.clone(),
                        status: match x {
                            'A' => GitFileStatus::Staged,
                            'D' => GitFileStatus::Deleted,
                            'R' => GitFileStatus::Renamed,
                            'C' => GitFileStatus::Copied,
                            'T' => GitFileStatus::TypeChanged,
                            _ => GitFileStatus::Modified,
                        },
                        raw_code: format!("{} ", x),
                    });
                }

                // Unstaged entry
                if y != '.' {
                    parsed.unstaged.push(GitFileEntry {
                        path,
                        status: match y {
                            'D' => GitFileStatus::Deleted,
                            'T' => GitFileStatus::TypeChanged,
                            _ => GitFileStatus::Modified,
                        },
                        raw_code: format!(" {}", y),
                    });
                }
            }
        } else if line.starts_with("2 ") {
            // Renamed/copied: 2 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path>\0<origPath>
            let parts: Vec<&str> = line.splitn(10, ' ').collect();
            if parts.len() >= 10 {
                let xy = parts[1];
                let path = parts[9].to_string();
                let orig_path = entries
                    .next()
                    .map(|o| String::from_utf8_lossy(o).to_string());
                let x = xy.chars().next().unwrap_or('.');
                let y = xy.chars().nth(1).unwrap_or('.');

                let display_path = if let Some(orig) = orig_path {
                    format!("{} -> {}", orig, path)
                } else {
                    path.clone()
                };

                if x != '.' {
                    parsed.staged.push(GitFileEntry {
                        path: display_path.clone(),
                        status: if x == 'C' {
                            GitFileStatus::Copied
                        } else {
                            GitFileStatus::Renamed
                        },
                        raw_code: format!("{} ", x),
                    });
                }

                if y != '.' {
                    parsed.unstaged.push(GitFileEntry {
                        path: display_path,
                        status: GitFileStatus::Modified,
                        raw_code: format!(" {}", y),
                    });
                }
            }
        } else if line.starts_with("u ") {
            // Unmerged / conflict entry
            let parts: Vec<&str> = line.splitn(11, ' ').collect();
            if parts.len() >= 11 {
                let path = parts[10].to_string();
                parsed.conflicted.push(GitFileEntry {
                    path,
                    status: GitFileStatus::Conflicted,
                    raw_code: "UU".to_string(),
                });
            }
        } else if let Some(path) = line.strip_prefix("? ") {
            parsed.untracked.push(GitFileEntry {
                path: path.to_string(),
                status: GitFileStatus::Untracked,
                raw_code: "??".to_string(),
            });
        }
    }
}

/// Fallback parser using porcelain v1 when v2 is unavailable.
fn fallback_porcelain_v1(dir: &Path, parsed: &mut ParsedPorcelainStatus) {
    if let Ok(o) = Command::new("git")
        .current_dir(dir)
        .args(["branch", "--show-current"])
        .output()
    {
        let b = String::from_utf8_lossy(&o.stdout).trim().to_string();
        if !b.is_empty() {
            parsed.branch = b;
        }
    }

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
                parsed.untracked.push(GitFileEntry {
                    path: file_path,
                    status: GitFileStatus::Untracked,
                    raw_code: "??".to_string(),
                });
            } else {
                if x != ' ' && x != '?' {
                    parsed.staged.push(GitFileEntry {
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
                if y != ' ' && y != '?' {
                    parsed.unstaged.push(GitFileEntry {
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
}

/// Fetch recent commits log with structured delimited fields.
pub fn query_recent_commits(dir: &Path, max_count: usize) -> Vec<GitCommitInfo> {
    let mut commits = Vec::new();
    let format = "%H\x1f%h\x1f%an\x1f%ae\x1f%ad\x1f%s\x1e";
    let n_str = max_count.to_string();

    if let Ok(out) = Command::new("git")
        .current_dir(dir)
        .args([
            "log",
            "-n",
            &n_str,
            &format!("--format={}", format),
            "--date=short",
        ])
        .output()
    {
        let text = String::from_utf8_lossy(&out.stdout);
        for record in text.split('\x1e') {
            let record = record.trim();
            if record.is_empty() {
                continue;
            }
            let fields: Vec<&str> = record.split('\x1f').collect();
            if fields.len() >= 6 {
                commits.push(GitCommitInfo {
                    hash: fields[0].to_string(),
                    short_hash: fields[1].to_string(),
                    author: fields[2].to_string(),
                    author_email: fields[3].to_string(),
                    date: fields[4].to_string(),
                    message: fields[5].to_string(),
                });
            }
        }
    }
    commits
}

/// Stage a single file (`git add <file>`).
pub fn stage_file(dir: &Path, file: &str) -> bool {
    Command::new("git")
        .current_dir(dir)
        .args(["add", "--", file])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Unstage a single file (`git restore --staged <file>` with fallback to `git rm --cached` on unborn HEAD).
pub fn unstage_file(dir: &Path, file: &str) -> bool {
    if let Ok(o) = Command::new("git")
        .current_dir(dir)
        .args(["restore", "--staged", "--", file])
        .output()
    {
        if o.status.success() {
            return true;
        }
    }

    // Fallback for unborn HEAD (initial commit before any revisions exist)
    Command::new("git")
        .current_dir(dir)
        .args(["rm", "--cached", "-r", "--", file])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Stage all modified and untracked files (`git add -A`).
pub fn stage_all(dir: &Path) -> bool {
    Command::new("git")
        .current_dir(dir)
        .args(["add", "-A"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Unstage all staged files (`git restore --staged .` with fallback to `git rm --cached -r .` on unborn HEAD).
pub fn unstage_all(dir: &Path) -> bool {
    if let Ok(o) = Command::new("git")
        .current_dir(dir)
        .args(["restore", "--staged", "."])
        .output()
    {
        if o.status.success() {
            return true;
        }
    }

    // Fallback for unborn HEAD (initial commit before any revisions exist)
    Command::new("git")
        .current_dir(dir)
        .args(["rm", "--cached", "-r", "."])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
