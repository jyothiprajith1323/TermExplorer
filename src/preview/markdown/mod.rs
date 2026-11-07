//! Markdown document parser and preview generator.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct TocItem {
    pub level: usize,
    pub title: String,
    pub line_number: usize,
}

#[derive(Debug, Clone)]
pub struct MarkdownPreview {
    pub formatted_lines: Vec<String>,
    pub toc: Vec<TocItem>,
    pub total_lines: usize,
}

impl MarkdownPreview {
    pub fn load(path: &Path, max_lines: usize) -> Self {
        let Ok(file) = File::open(path) else {
            return Self {
                formatted_lines: vec!["[Error: Could not open markdown file]".to_string()],
                toc: Vec::new(),
                total_lines: 0,
            };
        };

        let reader = BufReader::new(file);
        let mut formatted_lines = Vec::new();
        let mut toc = Vec::new();
        let mut in_code_block = false;
        let mut total_lines = 0;

        for (idx, line_res) in reader.lines().enumerate() {
            if formatted_lines.len() >= max_lines {
                break;
            }
            total_lines += 1;
            let line = line_res.unwrap_or_default();

            if line.trim_start().starts_with("```") {
                in_code_block = !in_code_block;
                if formatted_lines.len() < max_lines {
                    formatted_lines.push(format!("  ─── {} ───", line.trim()));
                }
                continue;
            }

            if in_code_block {
                if formatted_lines.len() < max_lines {
                    formatted_lines.push(format!("  │ {}", line));
                }
                continue;
            }

            // Heading detection
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                let hashes = trimmed.chars().take_while(|&c| c == '#').count();
                if hashes <= 6 && trimmed.chars().nth(hashes) == Some(' ') {
                    let title = trimmed[hashes + 1..].trim().to_string();
                    toc.push(TocItem {
                        level: hashes,
                        title: title.clone(),
                        line_number: idx + 1,
                    });

                    if formatted_lines.len() < max_lines {
                        let prefix = match hashes {
                            1 => "█ ",
                            2 => "▓▓ ",
                            3 => "▒▒▒ ",
                            _ => "• ",
                        };
                        formatted_lines.push(String::new());
                        formatted_lines.push(format!("{}{}", prefix, title));
                        formatted_lines.push(String::new());
                        continue;
                    }
                }
            }

            // Checkbox detection
            if trimmed.starts_with("- [ ] ") || trimmed.starts_with("* [ ] ") {
                if formatted_lines.len() < max_lines {
                    formatted_lines.push(format!("  ☐ {}", &trimmed[6..]));
                }
                continue;
            } else if trimmed.starts_with("- [x] ")
                || trimmed.starts_with("* [x] ")
                || trimmed.starts_with("- [X] ")
            {
                if formatted_lines.len() < max_lines {
                    formatted_lines.push(format!("  ☑ {}", &trimmed[6..]));
                }
                continue;
            }

            // Unordered list
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                if formatted_lines.len() < max_lines {
                    formatted_lines.push(format!("  • {}", &trimmed[2..]));
                }
                continue;
            }

            if formatted_lines.len() < max_lines {
                formatted_lines.push(line);
            }
        }

        Self {
            formatted_lines,
            toc,
            total_lines,
        }
    }
}
