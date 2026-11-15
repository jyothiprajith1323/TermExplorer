//! Unified diff generation, parsing, and syntax-highlighted Ratatui rendering.

use crate::git::types::{DiffLineKind, GitDiffFile, GitDiffHunk, GitDiffLine};
use crate::theme::Theme;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use std::path::Path;
use std::process::Command;

/// Fetch unified diff for a single file (staged or unstaged).
pub fn get_file_diff(dir: &Path, file_path: &str, staged: bool) -> Result<String, String> {
    let mut args = vec!["diff", "--no-color"];
    if staged {
        args.push("--cached");
    }
    args.push("--");
    args.push(file_path);

    let out = Command::new("git")
        .current_dir(dir)
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to run git diff: {e}"))?;

    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).to_string())
    }
}

/// Fetch complete working tree diff (staged or unstaged).
pub fn get_working_tree_diff(dir: &Path, staged: bool) -> Result<String, String> {
    let mut args = vec!["diff", "--no-color"];
    if staged {
        args.push("--cached");
    }

    let out = Command::new("git")
        .current_dir(dir)
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to run git diff: {e}"))?;

    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).to_string())
    }
}

/// Fetch diff introduced by a specific commit.
pub fn get_commit_diff(dir: &Path, commit_hash: &str) -> Result<String, String> {
    let out = Command::new("git")
        .current_dir(dir)
        .args(["show", "--no-color", "--format=", commit_hash])
        .output()
        .map_err(|e| format!("Failed to run git show: {e}"))?;

    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).to_string())
    }
}

/// Parse raw unified diff text into structured `GitDiffFile` objects.
pub fn parse_unified_diff(raw: &str) -> Vec<GitDiffFile> {
    let mut files = Vec::new();
    let mut current_file: Option<GitDiffFile> = None;
    let mut current_hunk: Option<GitDiffHunk> = None;

    let mut old_lineno = 0;
    let mut new_lineno = 0;

    for line in raw.lines() {
        if line.starts_with("diff --git ") {
            if let Some(hunk) = current_hunk.take() {
                if let Some(ref mut file) = current_file {
                    file.hunks.push(hunk);
                }
            }
            if let Some(file) = current_file.take() {
                files.push(file);
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            let (old_p, new_p) = if parts.len() >= 4 {
                (
                    parts[2].strip_prefix("a/").unwrap_or(parts[2]).to_string(),
                    parts[3].strip_prefix("b/").unwrap_or(parts[3]).to_string(),
                )
            } else {
                (String::new(), String::new())
            };

            current_file = Some(GitDiffFile {
                old_path: old_p,
                new_path: new_p,
                is_binary: false,
                hunks: Vec::new(),
            });
        } else if line.starts_with("Binary files ") {
            if let Some(ref mut file) = current_file {
                file.is_binary = true;
            }
        } else if line.starts_with("@@ ") {
            if let Some(hunk) = current_hunk.take() {
                if let Some(ref mut file) = current_file {
                    file.hunks.push(hunk);
                }
            }

            // Parse @@ -old_start,old_lines +new_start,new_lines @@
            let (o_start, o_lines, n_start, n_lines) = parse_hunk_header(line);
            old_lineno = o_start;
            new_lineno = n_start;

            current_hunk = Some(GitDiffHunk {
                old_start: o_start,
                old_lines: o_lines,
                new_start: n_start,
                new_lines: n_lines,
                header: line.to_string(),
                lines: Vec::new(),
            });
        } else if let Some(ref mut hunk) = current_hunk {
            if let Some(rest) = line.strip_prefix('+') {
                hunk.lines.push(GitDiffLine {
                    kind: DiffLineKind::Addition,
                    old_lineno: None,
                    new_lineno: Some(new_lineno),
                    text: rest.to_string(),
                });
                new_lineno += 1;
            } else if let Some(rest) = line.strip_prefix('-') {
                hunk.lines.push(GitDiffLine {
                    kind: DiffLineKind::Deletion,
                    old_lineno: Some(old_lineno),
                    new_lineno: None,
                    text: rest.to_string(),
                });
                old_lineno += 1;
            } else if let Some(rest) = line.strip_prefix(' ') {
                hunk.lines.push(GitDiffLine {
                    kind: DiffLineKind::Context,
                    old_lineno: Some(old_lineno),
                    new_lineno: Some(new_lineno),
                    text: rest.to_string(),
                });
                old_lineno += 1;
                new_lineno += 1;
            } else if line.starts_with('\\') {
                // "\ No newline at end of file"
                hunk.lines.push(GitDiffLine {
                    kind: DiffLineKind::Header,
                    old_lineno: None,
                    new_lineno: None,
                    text: line.to_string(),
                });
            }
        }
    }

    if let Some(hunk) = current_hunk.take() {
        if let Some(ref mut file) = current_file {
            file.hunks.push(hunk);
        }
    }
    if let Some(file) = current_file.take() {
        files.push(file);
    }

    files
}

/// Helper to parse `@@ -a,b +c,d @@` hunk header.
fn parse_hunk_header(line: &str) -> (usize, usize, usize, usize) {
    let mut old_start = 1;
    let mut old_lines = 1;
    let mut new_start = 1;
    let mut new_lines = 1;

    let parts: Vec<&str> = line.split("@@").collect();
    if parts.len() >= 2 {
        let coords: Vec<&str> = parts[1].split_whitespace().collect();
        for c in coords {
            if let Some(old_part) = c.strip_prefix('-') {
                if let Some((s, l)) = old_part.split_once(',') {
                    old_start = s.parse().unwrap_or(1);
                    old_lines = l.parse().unwrap_or(1);
                } else {
                    old_start = old_part.parse().unwrap_or(1);
                }
            } else if let Some(new_part) = c.strip_prefix('+') {
                if let Some((s, l)) = new_part.split_once(',') {
                    new_start = s.parse().unwrap_or(1);
                    new_lines = l.parse().unwrap_or(1);
                } else {
                    new_start = new_part.parse().unwrap_or(1);
                }
            }
        }
    }

    (old_start, old_lines, new_start, new_lines)
}

/// Render diff text to styled Ratatui `Line`s using Catppuccin Mocha colors.
pub fn render_diff_lines(diff_text: &str, theme: &Theme) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    if diff_text.trim().is_empty() {
        lines.push(Line::from(Span::styled(
            "  (No changes detected)",
            Style::default().fg(theme.hidden_color),
        )));
        return lines;
    }

    let cyan_teal = Color::Rgb(148, 226, 213); // Catppuccin Teal
    let mauve_blue = Color::Rgb(203, 166, 247); // Catppuccin Mauve

    for line_str in diff_text.lines() {
        if line_str.starts_with("diff --git")
            || line_str.starts_with("--- ")
            || line_str.starts_with("+++ ")
            || line_str.starts_with("index ")
        {
            lines.push(Line::from(Span::styled(
                line_str.to_string(),
                Style::default().fg(mauve_blue).add_modifier(Modifier::BOLD),
            )));
        } else if line_str.starts_with("@@ ") {
            lines.push(Line::from(Span::styled(
                line_str.to_string(),
                Style::default().fg(cyan_teal).add_modifier(Modifier::BOLD),
            )));
        } else if line_str.starts_with('+') {
            lines.push(Line::from(Span::styled(
                line_str.to_string(),
                Style::default().fg(theme.success),
            )));
        } else if line_str.starts_with('-') {
            lines.push(Line::from(Span::styled(
                line_str.to_string(),
                Style::default().fg(theme.error),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                line_str.to_string(),
                Style::default().fg(theme.fg),
            )));
        }
    }

    lines
}
