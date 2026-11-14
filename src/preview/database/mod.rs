//! Pure-Rust SQLite database inspector using rusqlite with strictly read-only connections.
//!
//! Extracts table and view schemas, column definitions, indexes, and sample row previews.

use rusqlite::{Connection, OpenFlags};
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub col_type: String,
    pub is_pk: bool,
    pub not_null: bool,
}

#[derive(Debug, Clone)]
pub struct TableSchemaInfo {
    pub name: String,
    pub sql: String,
    pub columns: Vec<ColumnInfo>,
    pub row_count_est: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct DatabasePreview {
    pub is_sqlite: bool,
    pub page_size: u32,
    pub table_count: usize,
    pub view_count: usize,
    pub tables: Vec<TableSchemaInfo>,
    pub sample_rows: Vec<String>,
    pub is_read_only: bool,
    pub error_message: Option<String>,
}

impl DatabasePreview {
    pub fn load(path: &Path, max_tables: usize) -> Self {
        let mut is_sqlite = false;
        let mut page_size = 0u32;

        // 1. Validate SQLite 3 magic header (100 bytes)
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

        if !is_sqlite {
            return Self {
                is_sqlite: false,
                page_size: 0,
                table_count: 0,
                view_count: 0,
                tables: Vec::new(),
                sample_rows: Vec::new(),
                is_read_only: true,
                error_message: Some("Not a valid SQLite 3 database header".to_string()),
            };
        }

        let mut tables = Vec::new();
        let mut sample_rows = Vec::new();
        let mut view_count = 0;
        let mut error_message = None;

        // 2. Open read-only SQLite connection via rusqlite
        match Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
            Ok(conn) => {
                // Query table & view names
                if let Ok(mut stmt) = conn.prepare(
                    "SELECT type, name, sql FROM sqlite_master WHERE name NOT LIKE 'sqlite_%' ORDER BY type, name LIMIT ?;"
                ) {
                    if let Ok(rows) = stmt.query_map([max_tables as i64], |row| {
                        let obj_type: String = row.get(0)?;
                        let name: String = row.get(1)?;
                        let sql: String = row.get(2).unwrap_or_default();
                        Ok((obj_type, name, sql))
                    }) {
                        for item in rows.flatten() {
                            let (obj_type, name, sql) = item;
                            if obj_type == "view" {
                                view_count += 1;
                            }

                            // Query table pragma table_info(name)
                            let mut columns = Vec::new();
                            let pragma_query = format!("PRAGMA table_info(\"{name}\");");
                            if let Ok(mut p_stmt) = conn.prepare(&pragma_query) {
                                if let Ok(p_rows) = p_stmt.query_map([], |p_row| {
                                    let col_name: String = p_row.get(1)?;
                                    let col_type: String = p_row.get(2)?;
                                    let not_null: bool = p_row.get(3)?;
                                    let is_pk: bool = p_row.get(5)?;
                                    Ok(ColumnInfo {
                                        name: col_name,
                                        col_type,
                                        is_pk,
                                        not_null,
                                    })
                                }) {
                                    for col in p_rows.flatten() {
                                        columns.push(col);
                                    }
                                }
                            }

                            // Estimate row count
                            let count_query = format!("SELECT COUNT(*) FROM \"{name}\";");
                            let count: Option<i64> = conn.query_row(&count_query, [], |r| r.get(0)).ok();

                            tables.push(TableSchemaInfo {
                                name,
                                sql,
                                columns,
                                row_count_est: count,
                            });
                        }
                    }
                }

                // Query sample rows from first table
                if let Some(first_table) = tables.first() {
                    let sample_query = format!("SELECT * FROM \"{}\" LIMIT 8;", first_table.name);
                    if let Ok(mut s_stmt) = conn.prepare(&sample_query) {
                        let col_count = s_stmt.column_count();
                        let col_names: Vec<String> =
                            s_stmt.column_names().map(str::to_string).collect();

                        sample_rows.push(col_names.join(" │ "));

                        if let Ok(s_rows) = s_stmt.query_map([], |row| {
                            let mut vals = Vec::new();
                            for i in 0..col_count {
                                let val_str = match row.get_ref(i) {
                                    Ok(rusqlite::types::ValueRef::Null) => "NULL".to_string(),
                                    Ok(rusqlite::types::ValueRef::Integer(n)) => n.to_string(),
                                    Ok(rusqlite::types::ValueRef::Real(f)) => format!("{f:.2}"),
                                    Ok(rusqlite::types::ValueRef::Text(s)) => {
                                        let text = String::from_utf8_lossy(s);
                                        if text.len() > 25 {
                                            format!("{}…", &text[..24])
                                        } else {
                                            text.to_string()
                                        }
                                    }
                                    Ok(rusqlite::types::ValueRef::Blob(b)) => {
                                        format!("<blob {}B>", b.len())
                                    }
                                    Err(_) => "?".to_string(),
                                };
                                vals.push(val_str);
                            }
                            Ok(vals.join(" │ "))
                        }) {
                            for r in s_rows.flatten() {
                                sample_rows.push(r);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error_message = Some(format!("Could not read SQLite database: {e}"));
            }
        }

        let table_count = tables.len();

        Self {
            is_sqlite: true,
            page_size,
            table_count,
            view_count,
            tables,
            sample_rows,
            is_read_only: true,
            error_message,
        }
    }
}
