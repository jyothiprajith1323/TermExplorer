//! Asynchronous workspace live content search (ripgrep-style text and regex grep).

use crate::search::cancellation::SearchCancellation;
use crate::search::filters::SearchFilters;
use ignore::WalkBuilder;
use regex::RegexBuilder;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ContentMatch {
    pub path: PathBuf,
    pub relative_path: String,
    pub line_number: usize,
    pub line_text: String,
    pub match_start: usize,
    pub match_end: usize,
}

pub fn search_content(
    root: &Path,
    query: &str,
    filters: &SearchFilters,
    cancellation: &SearchCancellation,
    generation: u64,
) -> Vec<ContentMatch> {
    if query.trim().is_empty() || cancellation.is_cancelled(generation) {
        return Vec::new();
    }

    // Build regex pattern according to filters
    let pattern = if filters.is_regex {
        if filters.whole_word {
            format!(r"\b{}\b", query)
        } else {
            query.to_string()
        }
    } else {
        let escaped = regex::escape(query);
        if filters.whole_word {
            format!(r"\b{}\b", escaped)
        } else {
            escaped
        }
    };

    let regex = match RegexBuilder::new(&pattern)
        .case_insensitive(!filters.case_sensitive)
        .multi_line(false)
        .build()
    {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let mut builder = WalkBuilder::new(root);
    builder
        .hidden(!filters.include_hidden)
        .git_ignore(filters.respect_gitignore)
        .git_global(filters.respect_gitignore)
        .git_exclude(filters.respect_gitignore)
        .filter_entry(|entry| {
            let name = entry.file_name().to_string_lossy();
            !(entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                && matches!(name.as_ref(), ".git" | "node_modules" | "target" | ".cargo"))
        });

    let mut matches = Vec::new();
    let mut files_scanned = 0;

    for result in builder.build() {
        files_scanned += 1;
        if files_scanned % 50 == 0 && cancellation.is_cancelled(generation) {
            return Vec::new();
        }

        let Ok(entry) = result else { continue };
        if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            continue;
        }

        let path = entry.path().to_path_buf();

        // Optional extension filter
        if let Some(ref req_ext) = filters.file_extension {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if ext != req_ext.to_ascii_lowercase() {
                continue;
            }
        }

        // Fast binary check: skip if file contains null bytes in initial 512 bytes
        let Ok(file) = File::open(&path) else {
            continue;
        };
        let mut header = [0u8; 512];
        let mut check_file = match File::open(&path) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let n = check_file.read(&mut header).unwrap_or(0);
        if header[..n].contains(&0u8) {
            continue; // Skip binary files
        }

        let relative_path = path
            .strip_prefix(root)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| path.display().to_string());

        let reader = BufReader::new(file);

        for (line_idx, line_res) in reader.lines().enumerate() {
            if cancellation.is_cancelled(generation) {
                return Vec::new();
            }

            let Ok(line) = line_res else { continue };
            if line.len() > 2000 {
                continue; // Skip minified or oversized lines
            }

            if let Some(mat) = regex.find(&line) {
                let match_start = mat.start();
                let match_end = mat.end();
                matches.push(ContentMatch {
                    path: path.clone(),
                    relative_path: relative_path.clone(),
                    line_number: line_idx + 1,
                    line_text: line,
                    match_start,
                    match_end,
                });

                if matches.len() >= filters.max_results {
                    return matches;
                }
            }
        }
    }

    matches
}
