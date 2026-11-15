//! Git branch listing, discovery, switching, and safe branch lifecycle operations.

use crate::git::types::GitBranch;
use std::path::Path;
use std::process::Command;

/// Discover all local and remote branches in the repository.
pub fn list_branches(dir: &Path) -> Result<Vec<GitBranch>, String> {
    let out = Command::new("git")
        .current_dir(dir)
        .args(["branch", "--list", "--all", "-v", "--no-color"])
        .output()
        .map_err(|e| format!("Failed to list branches: {e}"))?;

    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string());
    }

    let text = String::from_utf8_lossy(&out.stdout);
    let mut branches = Vec::new();

    for line in text.lines() {
        let is_current = line.starts_with('*');
        let line_trimmed = line.trim_start_matches('*').trim();
        if line_trimmed.is_empty() {
            continue;
        }

        // Check if HEAD pointer alias (e.g. remotes/origin/HEAD -> origin/master)
        if line_trimmed.contains("->") {
            continue;
        }

        let mut parts = line_trimmed.split_whitespace();
        let name = parts.next().unwrap_or("").to_string();
        let is_remote = name.starts_with("remotes/");

        let commit_hash = parts.next().unwrap_or("").to_string();

        let remaining: Vec<&str> = parts.collect();
        let mut upstream = None;
        let mut ahead = 0;
        let mut behind = 0;
        let mut summary_start_idx = 0;

        // Check for upstream tracking info: [origin/master: ahead 1, behind 2]
        if let Some(first) = remaining.first() {
            if first.starts_with('[') {
                let mut bracket_content = String::new();
                for (idx, token) in remaining.iter().enumerate() {
                    bracket_content.push_str(token);
                    bracket_content.push(' ');
                    if token.ends_with(']') {
                        summary_start_idx = idx + 1;
                        break;
                    }
                }

                let clean = bracket_content
                    .trim()
                    .trim_start_matches('[')
                    .trim_end_matches(']');
                let mut bracket_parts = clean.split(':');
                if let Some(up) = bracket_parts.next() {
                    upstream = Some(up.trim().to_string());
                }
                if let Some(counts) = bracket_parts.next() {
                    for count_part in counts.split(',') {
                        let c = count_part.trim();
                        if let Some(n) = c.strip_prefix("ahead ") {
                            ahead = n.trim().parse().unwrap_or(0);
                        } else if let Some(n) = c.strip_prefix("behind ") {
                            behind = n.trim().parse().unwrap_or(0);
                        }
                    }
                }
            }
        }

        let commit_summary = remaining[summary_start_idx..].join(" ");

        branches.push(GitBranch {
            name,
            is_current,
            is_remote,
            upstream,
            ahead,
            behind,
            commit_hash,
            commit_summary,
        });
    }

    Ok(branches)
}

/// Switch to an existing local branch safely.
pub fn switch_branch(dir: &Path, branch_name: &str) -> Result<String, String> {
    let clean_name = branch_name
        .strip_prefix("remotes/origin/")
        .unwrap_or(branch_name);

    let out = Command::new("git")
        .current_dir(dir)
        .args(["switch", clean_name])
        .output()
        .map_err(|e| format!("Failed to switch branch: {e}"))?;

    if out.status.success() {
        let msg = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        if !msg.is_empty() {
            Ok(msg)
        } else {
            Ok(stderr)
        }
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

/// Create a new branch and switch to it immediately.
pub fn create_and_switch_branch(dir: &Path, branch_name: &str) -> Result<String, String> {
    let trimmed = branch_name.trim();
    if trimmed.is_empty() {
        return Err("Branch name cannot be empty".to_string());
    }

    let out = Command::new("git")
        .current_dir(dir)
        .args(["switch", "-c", trimmed])
        .output()
        .map_err(|e| format!("Failed to create branch: {e}"))?;

    if out.status.success() {
        let msg = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        if !msg.is_empty() {
            Ok(msg)
        } else {
            Ok(stderr)
        }
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

/// Delete a local branch safely.
pub fn delete_branch(dir: &Path, branch_name: &str, force: bool) -> Result<String, String> {
    let flag = if force { "-D" } else { "-d" };
    let out = Command::new("git")
        .current_dir(dir)
        .args(["branch", flag, branch_name])
        .output()
        .map_err(|e| format!("Failed to delete branch: {e}"))?;

    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}
