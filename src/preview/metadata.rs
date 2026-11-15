//! Detailed file metadata extraction.

use bytesize::ByteSize;
use chrono::{DateTime, Local};
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DetailedMetadata {
    pub file_name: String,
    pub path: String,
    pub size_bytes: u64,
    pub size_formatted: String,
    pub permissions_str: String,
    pub permissions_octal: String,
    pub uid: u32,
    pub gid: u32,
    pub modified_time: String,
    pub accessed_time: String,
    pub created_time: String,
    pub is_symlink: bool,
    pub symlink_target: Option<String>,
    pub mime_type: String,
    pub is_dir: bool,
    pub is_file: bool,
    pub git_branch: Option<String>,
    pub git_file_status: Option<String>,
    pub git_last_commit: Option<String>,
}

impl DetailedMetadata {
    pub fn from_path(path: &Path) -> Option<Self> {
        let symlink_meta = fs::symlink_metadata(path).ok()?;
        let is_symlink = symlink_meta.file_type().is_symlink();
        let symlink_target = if is_symlink {
            fs::read_link(path)
                .ok()
                .map(|p| p.to_string_lossy().to_string())
        } else {
            None
        };

        let meta = fs::metadata(path).unwrap_or(symlink_meta.clone());
        let size_bytes = meta.len();
        let size_formatted = ByteSize(size_bytes).to_string();

        let mode = meta.permissions().mode();
        let permissions_octal = format!("{:04o}", mode & 0o7777);
        let permissions_str = format_mode(mode);

        let modified_time = meta
            .modified()
            .ok()
            .map(|t| {
                DateTime::<Local>::from(t)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            })
            .unwrap_or_else(|| "Unknown".to_string());

        let accessed_time = meta
            .accessed()
            .ok()
            .map(|t| {
                DateTime::<Local>::from(t)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            })
            .unwrap_or_else(|| "Unknown".to_string());

        let created_time = meta
            .created()
            .ok()
            .map(|t| {
                DateTime::<Local>::from(t)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            })
            .unwrap_or_else(|| "Unknown".to_string());

        let mime_type = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        let (git_branch, git_file_status, git_last_commit) = extract_git_info(path);

        Some(Self {
            file_name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            path: path.display().to_string(),
            size_bytes,
            size_formatted,
            permissions_str,
            permissions_octal,
            uid: meta.uid(),
            gid: meta.gid(),
            modified_time,
            accessed_time,
            created_time,
            is_symlink,
            symlink_target,
            mime_type,
            is_dir: meta.is_dir(),
            is_file: meta.is_file(),
            git_branch,
            git_file_status,
            git_last_commit,
        })
    }
}

fn extract_git_info(path: &Path) -> (Option<String>, Option<String>, Option<String>) {
    let parent = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or(path)
    };

    let branch = std::process::Command::new("git")
        .current_dir(parent)
        .args(["branch", "--show-current"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            } else {
                None
            }
        });

    let status = if branch.is_some() {
        std::process::Command::new("git")
            .current_dir(parent)
            .args([
                "status",
                "--porcelain=v1",
                "--",
                path.to_str().unwrap_or(""),
            ])
            .output()
            .ok()
            .map(|o| {
                if o.status.success() {
                    let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if s.is_empty() {
                        "Clean".to_string()
                    } else {
                        s
                    }
                } else {
                    "Unknown".to_string()
                }
            })
    } else {
        None
    };

    let commit = if branch.is_some() {
        std::process::Command::new("git")
            .current_dir(parent)
            .args([
                "log",
                "-n",
                "1",
                "--oneline",
                "--",
                path.to_str().unwrap_or(""),
            ])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if s.is_empty() {
                        None
                    } else {
                        Some(s)
                    }
                } else {
                    None
                }
            })
    } else {
        None
    };

    (branch, status, commit)
}

fn format_mode(mode: u32) -> String {
    let mut s = String::with_capacity(10);
    // User
    s.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o100 != 0 { 'x' } else { '-' });
    // Group
    s.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o010 != 0 { 'x' } else { '-' });
    // Others
    s.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o001 != 0 { 'x' } else { '-' });
    s
}
