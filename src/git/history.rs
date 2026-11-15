//! Git commit history retrieval and file blame annotations.

use crate::git::types::{GitBlameLine, GitCommitInfo};
use std::path::Path;
use std::process::Command;

/// Fetch a paginated slice of commit history from the repository.
pub fn get_commit_history(
    dir: &Path,
    max_count: usize,
    skip: usize,
) -> Result<Vec<GitCommitInfo>, String> {
    let count_str = max_count.to_string();
    let skip_str = skip.to_string();
    let format = "%H\x1f%h\x1f%an\x1f%ae\x1f%ad\x1f%s\x1e";

    let out = Command::new("git")
        .current_dir(dir)
        .args([
            "log",
            "-n",
            &count_str,
            &format!("--skip={}", skip_str),
            &format!("--format={}", format),
            "--date=short",
        ])
        .output()
        .map_err(|e| format!("Failed to read git log: {e}"))?;

    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string());
    }

    let text = String::from_utf8_lossy(&out.stdout);
    let mut commits = Vec::new();

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

    Ok(commits)
}

/// Retrieve line-by-line blame annotations for a file.
pub fn get_file_blame(dir: &Path, file_path: &Path) -> Result<Vec<GitBlameLine>, String> {
    let rel_path = file_path
        .strip_prefix(dir)
        .unwrap_or(file_path)
        .to_string_lossy();

    let out = Command::new("git")
        .current_dir(dir)
        .args(["blame", "-s", "--date=short", "--", rel_path.as_ref()])
        .output()
        .map_err(|e| format!("Failed to run git blame: {e}"))?;

    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string());
    }

    let text = String::from_utf8_lossy(&out.stdout);
    let mut lines = Vec::new();

    for (idx, line_str) in text.lines().enumerate() {
        let line_num = idx + 1;
        // Format of `git blame -s --date=short`:
        // <hash> (<author> <date> <lineno>) <content>
        if let Some((meta, content)) = line_str.split_once(')') {
            let parts: Vec<&str> = meta.split_whitespace().collect();
            let hash = parts.first().copied().unwrap_or("???????").to_string();
            let author = if parts.len() >= 2 {
                parts[1].trim_start_matches('(').to_string()
            } else {
                "Unknown".to_string()
            };
            let date = if parts.len() >= 3 {
                parts[2].to_string()
            } else {
                "Unknown".to_string()
            };

            lines.push(GitBlameLine {
                line_num,
                commit_hash: hash,
                author,
                date,
                content: content.to_string(),
            });
        } else {
            lines.push(GitBlameLine {
                line_num,
                commit_hash: "???????".to_string(),
                author: "Unknown".to_string(),
                date: "".to_string(),
                content: line_str.to_string(),
            });
        }
    }

    Ok(lines)
}
