# TermExplorer: Structured Data Inspector Architecture

## 1. Overview

TermExplorer provides specialized native viewers for structured tabular and hierarchical datasets:
- **Tabular Data**: Delimiter-separated values (CSV, TSV) with RFC 4180 parsing, column autosizing, sorting, and cell navigation.
- **Hierarchical Trees**: Collapsible JSON and YAML tree navigation with jq-style key path derivation.
- **Relational Databases**: Pure-Rust read-only SQLite inspection (`rusqlite`) showing schema definitions, indexes, and row previews.

---

## 2. CSV / TSV Interactive Table Viewer

- **Strict RFC 4180 Parsing**: Built on the `csv` crate, safely handling quoted fields, embedded commas, and multiline cell strings without line splitting bugs.
- **Autosizing Column Widths**: Measures the maximum display width across headers and sampled rows (clamped between 4 and 40 characters) to optimize terminal real estate.
- **Table Statistics**: Reports total record count, column count, delimiter type, and header detection status.
- **Truncation and Pagination**: Streams large datasets with bounded row limits to ensure immediate rendering.

---

## 3. JSON & YAML Collapsible Tree Viewer

- **Tree Node Hierarchy**:
  ```rust
  pub struct JsonTreeNode {
      pub key_path: String, // e.g. ".users[0].address.city"
      pub display_key: String,
      pub value_preview: String,
      pub node_type: &'static str, // "Object", "Array", "String", "Number", "Boolean", "Null"
      pub depth: usize,
      pub is_expandable: bool,
      pub is_expanded: bool,
      pub child_count: usize,
  }
  ```
- **jq-Style Key Path Derivation**: As users navigate the tree, the exact jq-compatible query expression (e.g. `.dependencies["ratatui"].version`) is displayed in the status bar for immediate copy-pasting into terminal commands.
- **Syntax Validation & Error Locators**: If JSON or YAML is malformed, TermExplorer provides precise line and column syntax error diagnostics rather than failing silently.

---

## 4. Pure-Rust SQLite Read-Only Database Inspection

- **Safety Architecture**: Connections are strictly opened with `OpenFlags::SQLITE_OPEN_READ_ONLY`, preventing any schema or data modifications.
- **Catalog Inspection**: Queries `sqlite_master` for all tables, views, and indexes.
- **PRAGMA Column Extraction**: Executes `PRAGMA table_info(...)` to retrieve column names, types, primary key flags, and nullability constraints.
- **Sample Row Previews**: Queries the first 8 records of the primary table with neat ASCII column separators (` │ `).
