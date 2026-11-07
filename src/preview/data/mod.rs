//! Structured data file preview (CSV, TSV, JSON, JSONL, XML, YAML).

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub enum DataPreviewKind {
    Table {
        delimiter: char,
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        column_widths: Vec<usize>,
        total_rows: usize,
    },
    Json {
        pretty_lines: Vec<String>,
        is_valid: bool,
        error_message: Option<String>,
        total_keys: usize,
    },
    JsonLines {
        records: Vec<(usize, bool, String)>, // (line_num, valid, preview)
        total_records: usize,
    },
}

#[derive(Debug, Clone)]
pub struct DataPreview {
    pub kind: DataPreviewKind,
    pub format_title: &'static str,
}

impl DataPreview {
    pub fn load(path: &Path, max_lines: usize) -> Self {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        if ext == "csv" || ext == "tsv" {
            Self::load_delimited(path, if ext == "tsv" { '\t' } else { ',' }, max_lines)
        } else if ext == "json" {
            Self::load_json(path, max_lines)
        } else if ext == "jsonl" || ext == "ndjson" {
            Self::load_jsonl(path, max_lines)
        } else {
            Self::load_delimited(path, ',', max_lines)
        }
    }

    fn load_delimited(path: &Path, default_delim: char, max_lines: usize) -> Self {
        let Ok(file) = File::open(path) else {
            return Self {
                kind: DataPreviewKind::Table {
                    delimiter: default_delim,
                    headers: vec!["Error".to_string()],
                    rows: vec![vec!["Could not open file".to_string()]],
                    column_widths: vec![20],
                    total_rows: 0,
                },
                format_title: "Delimited Data",
            };
        };

        let reader = BufReader::new(file);
        let mut lines = Vec::new();
        let mut total_rows = 0;

        for line_res in reader.lines() {
            total_rows += 1;
            if let Ok(line) = line_res {
                if lines.len() < max_lines {
                    lines.push(line);
                }
            }
        }

        // Sniff delimiter from first line if not strictly TSV
        let delimiter = if default_delim == '\t' {
            '\t'
        } else if let Some(first) = lines.first() {
            let comma_count = first.matches(',').count();
            let semi_count = first.matches(';').count();
            let tab_count = first.matches('\t').count();
            let pipe_count = first.matches('|').count();
            if tab_count > comma_count && tab_count > semi_count {
                '\t'
            } else if semi_count > comma_count {
                ';'
            } else if pipe_count > comma_count {
                '|'
            } else {
                ','
            }
        } else {
            default_delim
        };

        let mut parsed_rows: Vec<Vec<String>> = lines
            .into_iter()
            .map(|l| l.split(delimiter).map(|s| s.trim().to_string()).collect())
            .collect();

        let headers = if !parsed_rows.is_empty() {
            parsed_rows.remove(0)
        } else {
            Vec::new()
        };

        let num_cols = headers
            .len()
            .max(parsed_rows.iter().map(|r| r.len()).max().unwrap_or(0));
        let mut column_widths = vec![4usize; num_cols];

        for (i, h) in headers.iter().enumerate() {
            column_widths[i] = column_widths[i].max(h.len());
        }

        for row in &parsed_rows {
            for (i, val) in row.iter().enumerate() {
                if i < column_widths.len() {
                    column_widths[i] = column_widths[i].max(val.len());
                }
            }
        }

        // Cap column width to 30 chars for readability
        for w in &mut column_widths {
            *w = (*w).min(30);
        }

        Self {
            kind: DataPreviewKind::Table {
                delimiter,
                headers,
                rows: parsed_rows,
                column_widths,
                total_rows,
            },
            format_title: if delimiter == '\t' {
                "Tab-Separated Values (TSV)"
            } else {
                "Comma-Separated Values (CSV)"
            },
        }
    }

    fn load_json(path: &Path, max_lines: usize) -> Self {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(val) => {
                let pretty = serde_json::to_string_pretty(&val).unwrap_or_default();
                let pretty_lines: Vec<String> = pretty
                    .lines()
                    .take(max_lines)
                    .map(|s| s.to_string())
                    .collect();
                let total_keys = if let Some(map) = val.as_object() {
                    map.len()
                } else if let Some(arr) = val.as_array() {
                    arr.len()
                } else {
                    1
                };

                Self {
                    kind: DataPreviewKind::Json {
                        pretty_lines,
                        is_valid: true,
                        error_message: None,
                        total_keys,
                    },
                    format_title: "JSON Document",
                }
            }
            Err(e) => Self {
                kind: DataPreviewKind::Json {
                    pretty_lines: content
                        .lines()
                        .take(max_lines)
                        .map(|s| s.to_string())
                        .collect(),
                    is_valid: false,
                    error_message: Some(format!(
                        "Syntax error at line {}, col {}: {}",
                        e.line(),
                        e.column(),
                        e
                    )),
                    total_keys: 0,
                },
                format_title: "Invalid JSON",
            },
        }
    }

    fn load_jsonl(path: &Path, max_lines: usize) -> Self {
        let Ok(file) = File::open(path) else {
            return Self {
                kind: DataPreviewKind::JsonLines {
                    records: Vec::new(),
                    total_records: 0,
                },
                format_title: "JSON Lines (JSONL)",
            };
        };

        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut total_records = 0;

        for (idx, line_res) in reader.lines().enumerate() {
            total_records += 1;
            if let Ok(line) = line_res {
                if records.len() < max_lines {
                    let is_valid = serde_json::from_str::<serde_json::Value>(&line).is_ok();
                    records.push((idx + 1, is_valid, line));
                }
            }
        }

        Self {
            kind: DataPreviewKind::JsonLines {
                records,
                total_records,
            },
            format_title: "JSON Lines (JSONL)",
        }
    }
}
