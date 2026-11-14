//! Structured data file inspector (CSV, TSV, JSON, JSONL, YAML).
//!
//! Provides interactive spreadsheet-like table navigation for delimited data
//! and collapsible hierarchical tree navigation with jq-style path extraction for JSON/YAML.

use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct JsonTreeNode {
    pub key_path: String,
    pub display_text: String,
    pub depth: usize,
    pub is_collapsible: bool,
    pub is_collapsed: bool,
    pub value_str: String,
}

#[derive(Debug, Clone)]
pub enum DataPreviewKind {
    Table {
        delimiter: char,
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        column_widths: Vec<usize>,
        total_rows: usize,
        selected_row: usize,
        selected_col: usize,
        sort_col: Option<usize>,
        sort_descending: bool,
        filter_query: String,
    },
    Json {
        pretty_lines: Vec<String>,
        tree_nodes: Vec<JsonTreeNode>,
        is_valid: bool,
        error_message: Option<String>,
        total_keys: usize,
        selected_node: usize,
    },
    Yaml {
        pretty_lines: Vec<String>,
        tree_nodes: Vec<JsonTreeNode>,
        is_valid: bool,
        error_message: Option<String>,
        selected_node: usize,
    },
    JsonLines {
        records: Vec<(usize, bool, String)>,
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
        } else if ext == "yaml" || ext == "yml" {
            Self::load_yaml(path, max_lines)
        } else if ext == "jsonl" || ext == "ndjson" {
            Self::load_jsonl(path, max_lines)
        } else {
            Self::load_delimited(path, ',', max_lines)
        }
    }

    /// Load CSV / TSV with RFC 4180 compliance via the `csv` crate.
    fn load_delimited(path: &Path, default_delim: char, max_lines: usize) -> Self {
        let Ok(file) = File::open(path) else {
            return Self {
                kind: DataPreviewKind::Table {
                    delimiter: default_delim,
                    headers: vec!["Error".to_string()],
                    rows: vec![vec!["Could not open file".to_string()]],
                    column_widths: vec![20],
                    total_rows: 0,
                    selected_row: 0,
                    selected_col: 0,
                    sort_col: None,
                    sort_descending: false,
                    filter_query: String::new(),
                },
                format_title: "Delimited Data",
            };
        };

        // Sniff delimiter from first 1024 bytes
        let mut sample = [0u8; 1024];
        let bytes_read = File::open(path)
            .and_then(|mut f| f.read(&mut sample))
            .unwrap_or(0);
        let sample_str = String::from_utf8_lossy(&sample[..bytes_read]);

        let delimiter = if default_delim == '\t' {
            b'\t'
        } else {
            let comma_count = sample_str.matches(',').count();
            let semi_count = sample_str.matches(';').count();
            let tab_count = sample_str.matches('\t').count();
            let pipe_count = sample_str.matches('|').count();

            if tab_count > comma_count && tab_count > semi_count {
                b'\t'
            } else if semi_count > comma_count {
                b';'
            } else if pipe_count > comma_count {
                b'|'
            } else {
                b','
            }
        };

        let mut reader = csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .has_headers(true)
            .flexible(true)
            .from_reader(BufReader::new(file));

        let headers: Vec<String> = reader
            .headers()
            .map(|h| h.iter().map(str::to_string).collect())
            .unwrap_or_default();

        let has_headers = !headers.is_empty();
        let mut rows = Vec::new();
        let mut total_rows = if has_headers { 1 } else { 0 };

        for record_res in reader.records() {
            total_rows += 1;
            if let Ok(rec) = record_res {
                if rows.len() < max_lines {
                    rows.push(rec.iter().map(str::to_string).collect::<Vec<_>>());
                }
            }
        }

        let num_cols = headers
            .len()
            .max(rows.iter().map(|r| r.len()).max().unwrap_or(0));
        let mut column_widths = vec![4usize; num_cols];

        for (i, h) in headers.iter().enumerate() {
            if i < column_widths.len() {
                column_widths[i] = column_widths[i].max(h.len());
            }
        }

        for row in &rows {
            for (i, val) in row.iter().enumerate() {
                if i < column_widths.len() {
                    column_widths[i] = column_widths[i].max(val.len());
                }
            }
        }

        // Cap column widths to 35 chars
        for w in &mut column_widths {
            *w = (*w).min(35);
        }

        let delim_char = delimiter as char;

        Self {
            kind: DataPreviewKind::Table {
                delimiter: delim_char,
                headers,
                rows,
                column_widths,
                total_rows,
                selected_row: 0,
                selected_col: 0,
                sort_col: None,
                sort_descending: false,
                filter_query: String::new(),
            },
            format_title: if delim_char == '\t' {
                "Tab-Separated Values (TSV)"
            } else {
                "Comma-Separated Values (CSV)"
            },
        }
    }

    /// Load and parse JSON document with pretty-printing and tree construction.
    fn load_json(path: &Path, max_lines: usize) -> Self {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(val) => {
                let pretty = serde_json::to_string_pretty(&val).unwrap_or_default();
                let pretty_lines: Vec<String> =
                    pretty.lines().take(max_lines).map(str::to_string).collect();

                let mut tree_nodes = Vec::new();
                build_json_tree(&val, "", 0, &mut tree_nodes);

                let total_keys = tree_nodes.len();

                Self {
                    kind: DataPreviewKind::Json {
                        pretty_lines,
                        tree_nodes,
                        is_valid: true,
                        error_message: None,
                        total_keys,
                        selected_node: 0,
                    },
                    format_title: "JSON Document",
                }
            }
            Err(e) => Self {
                kind: DataPreviewKind::Json {
                    pretty_lines: content
                        .lines()
                        .take(max_lines)
                        .map(str::to_string)
                        .collect(),
                    tree_nodes: Vec::new(),
                    is_valid: false,
                    error_message: Some(format!(
                        "Syntax error at line {}, col {}: {}",
                        e.line(),
                        e.column(),
                        e
                    )),
                    total_keys: 0,
                    selected_node: 0,
                },
                format_title: "Invalid JSON",
            },
        }
    }

    /// Load and parse YAML document with tree construction.
    fn load_yaml(path: &Path, max_lines: usize) -> Self {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        match serde_yaml::from_str::<serde_yaml::Value>(&content) {
            Ok(val) => {
                let pretty_lines: Vec<String> = content
                    .lines()
                    .take(max_lines)
                    .map(str::to_string)
                    .collect();

                let mut tree_nodes = Vec::new();
                build_yaml_tree(&val, "", 0, &mut tree_nodes);

                Self {
                    kind: DataPreviewKind::Yaml {
                        pretty_lines,
                        tree_nodes,
                        is_valid: true,
                        error_message: None,
                        selected_node: 0,
                    },
                    format_title: "YAML Document",
                }
            }
            Err(e) => Self {
                kind: DataPreviewKind::Yaml {
                    pretty_lines: content
                        .lines()
                        .take(max_lines)
                        .map(str::to_string)
                        .collect(),
                    tree_nodes: Vec::new(),
                    is_valid: false,
                    error_message: Some(format!("YAML parse error: {e}")),
                    selected_node: 0,
                },
                format_title: "Invalid YAML",
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

/// Recursively build JSON tree nodes with jq-style key paths (e.g. `.users[0].address.city`).
fn build_json_tree(val: &serde_json::Value, path: &str, depth: usize, out: &mut Vec<JsonTreeNode>) {
    if out.len() >= 500 {
        return; // Bounded tree size
    }

    match val {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                let sub_path = if path.is_empty() {
                    format!(".{k}")
                } else {
                    format!("{path}.{k}")
                };

                let is_collapsible = v.is_object() || v.is_array();
                let preview = match v {
                    serde_json::Value::Object(m) => format!("{{...}} ({} keys)", m.len()),
                    serde_json::Value::Array(a) => format!("[...] ({} items)", a.len()),
                    serde_json::Value::String(s) => format!("\"{s}\""),
                    _ => v.to_string(),
                };

                out.push(JsonTreeNode {
                    key_path: sub_path.clone(),
                    display_text: format!("{k}: {preview}"),
                    depth,
                    is_collapsible,
                    is_collapsed: false,
                    value_str: v.to_string(),
                });

                if is_collapsible && depth < 4 {
                    build_json_tree(v, &sub_path, depth + 1, out);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for (idx, v) in arr.iter().enumerate() {
                let sub_path = format!("{path}[{idx}]");
                let is_collapsible = v.is_object() || v.is_array();
                let preview = match v {
                    serde_json::Value::Object(m) => format!("{{...}} ({} keys)", m.len()),
                    serde_json::Value::Array(a) => format!("[...] ({} items)", a.len()),
                    serde_json::Value::String(s) => format!("\"{s}\""),
                    _ => v.to_string(),
                };

                out.push(JsonTreeNode {
                    key_path: sub_path.clone(),
                    display_text: format!("[{idx}]: {preview}"),
                    depth,
                    is_collapsible,
                    is_collapsed: false,
                    value_str: v.to_string(),
                });

                if is_collapsible && depth < 4 {
                    build_json_tree(v, &sub_path, depth + 1, out);
                }
            }
        }
        _ => {}
    }
}

/// Recursively build YAML tree nodes with jq-style key paths.
fn build_yaml_tree(val: &serde_yaml::Value, path: &str, depth: usize, out: &mut Vec<JsonTreeNode>) {
    if out.len() >= 500 {
        return;
    }

    match val {
        serde_yaml::Value::Mapping(map) => {
            for (k, v) in map {
                let key_str = k.as_str().unwrap_or("key");
                let sub_path = if path.is_empty() {
                    format!(".{key_str}")
                } else {
                    format!("{path}.{key_str}")
                };

                let is_collapsible = v.is_mapping() || v.is_sequence();
                let preview = match v {
                    serde_yaml::Value::Mapping(m) => format!("(mapping, {} entries)", m.len()),
                    serde_yaml::Value::Sequence(s) => format!("(sequence, {} items)", s.len()),
                    serde_yaml::Value::String(s) => format!("\"{s}\""),
                    _ => format!("{v:?}"),
                };

                out.push(JsonTreeNode {
                    key_path: sub_path.clone(),
                    display_text: format!("{key_str}: {preview}"),
                    depth,
                    is_collapsible,
                    is_collapsed: false,
                    value_str: format!("{v:?}"),
                });

                if is_collapsible && depth < 4 {
                    build_yaml_tree(v, &sub_path, depth + 1, out);
                }
            }
        }
        serde_yaml::Value::Sequence(seq) => {
            for (idx, v) in seq.iter().enumerate() {
                let sub_path = format!("{path}[{idx}]");
                let is_collapsible = v.is_mapping() || v.is_sequence();
                let preview = match v {
                    serde_yaml::Value::Mapping(m) => format!("(mapping, {} entries)", m.len()),
                    serde_yaml::Value::Sequence(s) => format!("(sequence, {} items)", s.len()),
                    _ => format!("{v:?}"),
                };

                out.push(JsonTreeNode {
                    key_path: sub_path.clone(),
                    display_text: format!("[{idx}]: {preview}"),
                    depth,
                    is_collapsible,
                    is_collapsed: false,
                    value_str: format!("{v:?}"),
                });

                if is_collapsible && depth < 4 {
                    build_yaml_tree(v, &sub_path, depth + 1, out);
                }
            }
        }
        _ => {}
    }
}
