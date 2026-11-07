//! Source code and configuration preview with syntax highlighting.

use crate::editor::syntax::highlight_file;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct SourcePreview {
    pub lines: Vec<String>,
    pub highlighted_spans: Vec<Vec<(u8, u8, u8, String)>>,
    pub total_lines: usize,
    pub truncated: bool,
}

impl SourcePreview {
    pub fn load(path: &Path, max_bytes: u64, max_lines: usize) -> Self {
        let Ok(file) = File::open(path) else {
            return Self {
                lines: vec!["[Error: Could not open file for preview]".to_string()],
                highlighted_spans: Vec::new(),
                total_lines: 0,
                truncated: false,
            };
        };

        let reader = BufReader::new(file);
        let mut lines = Vec::new();
        let mut bytes_read = 0u64;
        let mut truncated = false;

        // Stop reading immediately once max_lines or max_bytes is reached to prevent
        // allocating thousands of unused Strings on large files and blocking the UI.
        for line_result in reader.lines() {
            match line_result {
                Ok(line) => {
                    bytes_read += line.len() as u64 + 1;
                    lines.push(line);
                    if lines.len() >= max_lines || bytes_read >= max_bytes {
                        truncated = true;
                        break;
                    }
                }
                Err(_) => {
                    lines.push("[Invalid UTF-8 sequence]".to_string());
                    break;
                }
            }
        }

        let total_lines = lines.len();
        let full_text = lines.join("\n");
        let highlighted_spans = highlight_file(path, &full_text);

        Self {
            lines,
            highlighted_spans,
            total_lines,
            truncated,
        }
    }
}
