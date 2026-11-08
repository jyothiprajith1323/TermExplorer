//! Layer 2: Workspace Symbol Indexer (< 50ms query latency).
//! Scans project source files in the background, ignoring `.git`, `target`, `node_modules`, etc.

use super::provider::{CompletionContext, CompletionItem, CompletionItemKind, CompletionProvider};
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSymbol {
    pub name: String,
    pub kind_name: String,
    pub relative_path: String,
    pub line_number: usize,
}

#[derive(Debug, Clone, Default)]
pub struct WorkspaceIndexer {
    cache: Arc<RwLock<BTreeMap<String, Vec<WorkspaceSymbol>>>>,
    indexed_root: Arc<RwLock<Option<PathBuf>>>,
}

impl WorkspaceIndexer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_ignored_dir(dir_name: &str) -> bool {
        matches!(
            dir_name,
            ".git"
                | "node_modules"
                | "target"
                | "dist"
                | "build"
                | ".cache"
                | ".vscode"
                | "vendor"
                | ".cargo"
                | ".gemini"
                | "website"
                | "screenshots"
                | "docs"
                | "tests"
                | ".venv"
                | "__pycache__"
                | "bin"
                | "obj"
                | "packaging"
                | "plugins"
        )
    }

    pub fn is_indexable_extension(ext: &str) -> bool {
        matches!(
            ext,
            "rs" | "py"
                | "c"
                | "h"
                | "cpp"
                | "hpp"
                | "java"
                | "go"
                | "js"
                | "ts"
                | "jsx"
                | "tsx"
                | "sql"
                | "toml"
                | "sh"
        )
    }

    pub fn index_directory(&self, root: &Path) {
        let mut symbols_map: BTreeMap<String, Vec<WorkspaceSymbol>> = BTreeMap::new();
        let mut files_indexed = 0;
        self.walk_and_index(root, root, &mut symbols_map, &mut files_indexed, 0, 250);

        if let Ok(mut lock) = self.cache.write() {
            *lock = symbols_map;
        }
        if let Ok(mut root_lock) = self.indexed_root.write() {
            *root_lock = Some(root.to_path_buf());
        }
    }

    fn walk_and_index(
        &self,
        current: &Path,
        root: &Path,
        map: &mut BTreeMap<String, Vec<WorkspaceSymbol>>,
        files_indexed: &mut usize,
        depth: usize,
        max_files: usize,
    ) {
        if depth > 5 || *files_indexed >= max_files {
            return;
        }

        let rd = match std::fs::read_dir(current) {
            Ok(r) => r,
            Err(_) => return,
        };

        for entry in rd.flatten() {
            let Ok(ft) = entry.file_type() else { continue };
            let file_name = entry.file_name().to_string_lossy().to_string();

            if ft.is_dir() {
                if !Self::is_ignored_dir(&file_name) && !file_name.starts_with('.') {
                    let path = entry.path();
                    self.walk_and_index(&path, root, map, files_indexed, depth + 1, max_files);
                    if *files_indexed >= max_files {
                        return;
                    }
                }
            } else if ft.is_file() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if Self::is_indexable_extension(ext) {
                        *files_indexed += 1;
                        self.index_file(&path, root, map);
                        if *files_indexed >= max_files {
                            return;
                        }
                    }
                }
            }
        }
    }

    fn index_file(
        &self,
        path: &Path,
        root: &Path,
        map: &mut BTreeMap<String, Vec<WorkspaceSymbol>>,
    ) {
        let file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(_) => return,
        };

        let rel_path = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        let reader = BufReader::new(file);

        for (idx, line_res) in reader.lines().take(600).enumerate() {
            let Ok(line) = line_res else { break };
            let trimmed = line.trim_start();
            if trimmed.starts_with("fn ")
                || trimmed.starts_with("pub fn ")
                || trimmed.starts_with("async fn ")
                || trimmed.starts_with("pub async fn ")
            {
                if let Some(name) = extract_name_after(trimmed, "fn ") {
                    map.entry(name.clone()).or_default().push(WorkspaceSymbol {
                        name,
                        kind_name: "fn".to_string(),
                        relative_path: rel_path.clone(),
                        line_number: idx + 1,
                    });
                }
            } else if trimmed.starts_with("struct ") || trimmed.starts_with("pub struct ") {
                if let Some(name) = extract_name_after(trimmed, "struct ") {
                    map.entry(name.clone()).or_default().push(WorkspaceSymbol {
                        name,
                        kind_name: "struct".to_string(),
                        relative_path: rel_path.clone(),
                        line_number: idx + 1,
                    });
                }
            } else if trimmed.starts_with("enum ") || trimmed.starts_with("pub enum ") {
                if let Some(name) = extract_name_after(trimmed, "enum ") {
                    map.entry(name.clone()).or_default().push(WorkspaceSymbol {
                        name,
                        kind_name: "enum".to_string(),
                        relative_path: rel_path.clone(),
                        line_number: idx + 1,
                    });
                }
            } else if trimmed.starts_with("trait ") || trimmed.starts_with("pub trait ") {
                if let Some(name) = extract_name_after(trimmed, "trait ") {
                    map.entry(name.clone()).or_default().push(WorkspaceSymbol {
                        name,
                        kind_name: "trait".to_string(),
                        relative_path: rel_path.clone(),
                        line_number: idx + 1,
                    });
                }
            } else if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
                if let Some(name) = extract_name_after(trimmed, "def ") {
                    map.entry(name.clone()).or_default().push(WorkspaceSymbol {
                        name,
                        kind_name: "def".to_string(),
                        relative_path: rel_path.clone(),
                        line_number: idx + 1,
                    });
                }
            } else if trimmed.starts_with("class ") {
                if let Some(name) = extract_name_after(trimmed, "class ") {
                    map.entry(name.clone()).or_default().push(WorkspaceSymbol {
                        name,
                        kind_name: "class".to_string(),
                        relative_path: rel_path.clone(),
                        line_number: idx + 1,
                    });
                }
            } else if trimmed.starts_with("type ") || trimmed.starts_with("interface ") {
                if let Some(name) = extract_name_after(trimmed, "type ")
                    .or_else(|| extract_name_after(trimmed, "interface "))
                {
                    map.entry(name.clone()).or_default().push(WorkspaceSymbol {
                        name,
                        kind_name: "type".to_string(),
                        relative_path: rel_path.clone(),
                        line_number: idx + 1,
                    });
                }
            }
        }
    }

    pub fn query_symbols(&self, prefix: &str, max: usize) -> Vec<CompletionItem> {
        if prefix.is_empty() {
            return Vec::new();
        }
        let prefix_lower = prefix.to_lowercase();
        let cache = match self.cache.read() {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        let mut results = Vec::new();

        for (sym_name, list) in cache.iter() {
            let sym_lower = sym_name.to_lowercase();
            if sym_lower.starts_with(&prefix_lower) && sym_name != prefix {
                if let Some(first) = list.first() {
                    let score = 420 + (sym_name.len() as i64).min(30);
                    results.push(CompletionItem {
                        label: sym_name.clone(),
                        detail: format!(
                            "{} ({}:{})",
                            first.kind_name, first.relative_path, first.line_number
                        ),
                        snippet_body: None,
                        documentation: Some(format!(
                            "Symbol {} defined in {}:{}",
                            sym_name, first.relative_path, first.line_number
                        )),
                        kind: CompletionItemKind::WorkspaceSymbol,
                        score,
                    });
                }
            }
            if results.len() >= max {
                break;
            }
        }

        results
    }
}

impl CompletionProvider for WorkspaceIndexer {
    fn complete(&self, ctx: &CompletionContext) -> Vec<CompletionItem> {
        self.query_symbols(ctx.prefix, 16)
    }
}

fn extract_name_after(line: &str, keyword: &str) -> Option<String> {
    let pos = line.find(keyword)?;
    let remainder = &line[pos + keyword.len()..];
    let name: String = remainder
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}
