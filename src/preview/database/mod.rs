//! SQLite database preview and schema inspection.

use crate::preview::registry::CapabilityRegistry;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct TableSchemaInfo {
    pub name: String,
    pub sql: String,
    pub row_count_est: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct DatabasePreview {
    pub is_sqlite: bool,
    pub page_size: u32,
    pub table_count: usize,
    pub tables: Vec<TableSchemaInfo>,
    pub sample_rows: Vec<String>,
    pub sqlite3_cli_available: bool,
}

impl DatabasePreview {
    pub fn load(path: &Path, max_tables: usize) -> Self {
        let sqlite3_cli_available = CapabilityRegistry::is_tool_available("sqlite3");

        let mut is_sqlite = false;
        let mut page_size = 0u32;

        // 1. Check SQLite 3 header (100 bytes)
        if let Ok(mut f) = File::open(path) {
            let mut header = [0u8; 100];
            if f.read_exact(&mut header).is_ok() && header.starts_with(b"SQLite format 3\0") {
                is_sqlite = true;
                page_size = u16::from_be_bytes([header[16], header[17]]) as u32;
                if page_size == 1 {
                    page_size = 65536;
                }
            }
        }

        let mut tables = Vec::new();
        let mut sample_rows = Vec::new();

        // 2. Query schema via sqlite3 CLI if available
        if is_sqlite && sqlite3_cli_available {
            if let Ok(out) = Command::new("sqlite3")
                .arg("-readonly")
                .arg(path)
                .arg("SELECT name, sql FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' LIMIT 20;")
                .output()
            {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines().take(max_tables) {
                    let mut parts = line.splitn(2, '|');
                    if let (Some(name), Some(sql)) = (parts.next(), parts.next()) {
                        tables.push(TableSchemaInfo {
                            name: name.to_string(),
                            sql: sql.to_string(),
                            row_count_est: None,
                        });
                    }
                }
            }

            // Preview first table sample data
            if let Some(first_table) = tables.first() {
                let query = format!("SELECT * FROM \"{}\" LIMIT 5;", first_table.name);
                if let Ok(out) = Command::new("sqlite3")
                    .arg("-readonly")
                    .arg("-header")
                    .arg("-column")
                    .arg(path)
                    .arg(&query)
                    .output()
                {
                    let text = String::from_utf8_lossy(&out.stdout);
                    for l in text.lines() {
                        sample_rows.push(l.to_string());
                    }
                }
            }
        } else if is_sqlite {
            // Pure-rust header-only fallback
            tables.push(TableSchemaInfo {
                name: "Database header validated".to_string(),
                sql: format!("SQLite 3 database (Page size: {} bytes). Install sqlite3 CLI for interactive table browsing.", page_size),
                row_count_est: None,
            });
        }

        let table_count = tables.len();

        Self {
            is_sqlite,
            page_size,
            table_count,
            tables,
            sample_rows,
            sqlite3_cli_available,
        }
    }
}
