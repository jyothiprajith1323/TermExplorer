use std::fs::File;
use std::io::Write;
use tempfile::tempdir;
use termexplorer::preview::capabilities::TerminalGraphicsProtocol;
use termexplorer::preview::data::{DataPreview, DataPreviewKind};
use termexplorer::preview::database::DatabasePreview;
use termexplorer::preview::hex::HexPreview;
use termexplorer::preview::image::ImagePreview;
use termexplorer::preview::markdown::MarkdownPreview;

#[test]
fn test_svg_in_process_rasterization() {
    let dir = tempdir().expect("tempdir");
    let svg_path = dir.path().join("test.svg");
    let svg_data = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
        <rect width="100" height="100" fill="#ff0000"/>
    </svg>"##;
    std::fs::write(&svg_path, svg_data).expect("write svg");

    let preview = ImagePreview::load(&svg_path, 40, 20);
    assert_eq!(preview.format_name, "Scalable Vector Graphics (SVG)");
    assert_eq!(preview.width, 100);
    assert_eq!(preview.height, 100);
    assert!(
        !preview.halfblock_grid.is_empty(),
        "Halfblock cells must be generated"
    );
}

#[test]
fn test_image_zoom_pan_fit() {
    let mut preview = ImagePreview {
        format_name: "PNG".to_string(),
        width: 800,
        height: 600,
        color_type: "RGBA 8-bit".to_string(),
        has_alpha: true,
        halfblock_grid: vec![vec![(
            ratatui::style::Color::Reset,
            ratatui::style::Color::Reset,
            ' ',
        )]],
        zoom_level: 1.0,
        pan_x: 0,
        pan_y: 0,
        fit_to_panel: true,
        aspect_ratio: "4:3".to_string(),
        is_animated: false,
        frame_count: Some(1),
        protocol: TerminalGraphicsProtocol::HalfBlocks,
    };

    assert_eq!(preview.zoom_level, 1.0);
    preview.zoom_in();
    assert_eq!(preview.zoom_level, 1.25);
    preview.zoom_out();
    assert_eq!(preview.zoom_level, 1.0);

    preview.pan(15, -20);
    assert_eq!(preview.pan_x, 15);
    assert_eq!(preview.pan_y, -20);

    preview.reset_zoom();
    assert_eq!(preview.zoom_level, 1.0);
    assert_eq!(preview.pan_x, 0);
    assert_eq!(preview.pan_y, 0);

    assert!(preview.fit_to_panel);
    preview.toggle_fit();
    assert!(!preview.fit_to_panel);
}

#[test]
fn test_sqlite_pure_rust_inspection() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().join("test.db");

    // Initialize SQLite database
    let conn = rusqlite::Connection::open(&db_path).expect("open sqlite db");
    conn.execute(
        "CREATE TABLE developer (id INTEGER PRIMARY KEY, username TEXT NOT NULL, role TEXT);",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO developer (id, username, role) VALUES (1, 'prajith', 'maintainer'), (2, 'agent', 'systems');",
        [],
    )
    .unwrap();
    drop(conn);

    let preview = DatabasePreview::load(&db_path, 20);
    assert!(preview.is_sqlite, "Database must be detected as SQLite");
    assert_eq!(preview.table_count, 1);
    assert_eq!(preview.tables.len(), 1);

    let dev_table = &preview.tables[0];
    assert_eq!(dev_table.name, "developer");
    assert_eq!(dev_table.columns.len(), 3);
    assert_eq!(dev_table.columns[0].name, "id");
    assert_eq!(dev_table.columns[1].name, "username");
    assert_eq!(dev_table.columns[2].name, "role");
    assert!(!preview.sample_rows.is_empty());
}

#[test]
fn test_markdown_ast_rendering() {
    let dir = tempdir().expect("tempdir");
    let md_path = dir.path().join("doc.md");
    let content = r#"# Architecture Overview

This is **bold** and *italic* text.

```rust
pub fn main() {
    println!("Hello TermExplorer");
}
```

- [x] High performance
- [ ] Todo item
"#;
    std::fs::write(&md_path, content).unwrap();

    let md = MarkdownPreview::load(&md_path, 80);
    assert!(md.heading_count >= 1);
    assert!(!md.toc.is_empty());
    assert_eq!(md.toc[0].title, "Architecture Overview");
    assert_eq!(md.toc[0].level, 1);
    assert!(!md.formatted_lines.is_empty());
}

#[test]
fn test_hex_viewer_virtualized_seeking() {
    let dir = tempdir().expect("tempdir");
    let bin_path = dir.path().join("binary.bin");
    let mut file = File::create(&bin_path).unwrap();

    // 1 KiB binary with magic header
    let mut data = vec![0u8; 1024];
    data[0] = 0x7F;
    data[1] = b'E';
    data[2] = b'L';
    data[3] = b'F';
    data[512] = 0xDE;
    data[513] = 0xAD;
    data[514] = 0xBE;
    data[515] = 0xEF;
    file.write_all(&data).unwrap();

    let hex = HexPreview::load(&bin_path, 256);
    assert_eq!(hex.detected_format, "ELF 32-bit Executable");
    assert_eq!(hex.file_size, 1024);

    // Virtualized seek to offset 512
    let hex_seek = HexPreview::load_at_offset(&bin_path, 512, 16);
    assert_eq!(hex_seek.rows.len(), 16);
    assert!(hex_seek.rows[0].hex_left.starts_with("de ad be ef"));

    // Pattern search
    let found_offsets = HexPreview::search_pattern(&bin_path, &[0xDE, 0xAD, 0xBE, 0xEF], 10);
    assert_eq!(found_offsets, vec![512]);
}

#[test]
fn test_data_preview_yaml_and_json_tree() {
    let dir = tempdir().expect("tempdir");
    let yaml_path = dir.path().join("config.yaml");
    let yaml_content = r#"
server:
  host: 127.0.0.1
  port: 8080
users:
  - admin
  - dev
"#;
    std::fs::write(&yaml_path, yaml_content).unwrap();

    let data = DataPreview::load(&yaml_path, 100);
    match data.kind {
        DataPreviewKind::Yaml { ref tree_nodes, .. } => {
            assert!(!tree_nodes.is_empty());
            // Tree paths
            assert!(tree_nodes.iter().any(|n| n.key_path.contains("server")));
        }
        _ => panic!("Expected DataPreviewKind::Yaml"),
    }
}
