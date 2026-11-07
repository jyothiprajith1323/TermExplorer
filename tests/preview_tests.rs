use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tempfile::tempdir;
use termexplorer::preview::archive::is_safe_archive_path;
use termexplorer::preview::cache::PreviewLruCache;
use termexplorer::preview::data::DataPreview;
use termexplorer::preview::detector::{detect_file_format, FileCategory};
use termexplorer::preview::hex::HexPreview;
use termexplorer::preview::metadata::DetailedMetadata;
use termexplorer::preview::rendering::GraphicsCapability;
use termexplorer::preview::PreviewManager;

#[test]
fn test_preview_detector_categories() {
    let dir = tempdir().unwrap();

    // 1. Rust file -> SourceCode
    let rs_path = dir.path().join("test.rs");
    File::create(&rs_path)
        .unwrap()
        .write_all(b"fn main() {}")
        .unwrap();
    let fmt = detect_file_format(&rs_path);
    assert_eq!(fmt.category, FileCategory::SourceCode);
    assert!(!fmt.is_binary);

    // 2. Markdown -> Markdown
    let md_path = dir.path().join("README.md");
    File::create(&md_path)
        .unwrap()
        .write_all(b"# Title\n- [ ] Task")
        .unwrap();
    let fmt = detect_file_format(&md_path);
    assert_eq!(fmt.category, FileCategory::Markdown);

    // 3. CSV -> Data
    let csv_path = dir.path().join("data.csv");
    File::create(&csv_path)
        .unwrap()
        .write_all(b"name,age,city\nAlice,30,Berlin")
        .unwrap();
    let fmt = detect_file_format(&csv_path);
    assert_eq!(fmt.category, FileCategory::Data);

    // 4. JSON -> Data
    let json_path = dir.path().join("config.json");
    File::create(&json_path)
        .unwrap()
        .write_all(b"{\"key\": \"value\"}")
        .unwrap();
    let fmt = detect_file_format(&json_path);
    assert_eq!(fmt.category, FileCategory::Data);

    // 5. ZIP -> Archive
    let zip_path = dir.path().join("archive.zip");
    File::create(&zip_path)
        .unwrap()
        .write_all(b"PK\x03\x04empty")
        .unwrap();
    let fmt = detect_file_format(&zip_path);
    assert_eq!(fmt.category, FileCategory::Archive);

    // 6. SQLite -> Database
    let db_path = dir.path().join("test.db");
    File::create(&db_path)
        .unwrap()
        .write_all(b"SQLite format 3\0")
        .unwrap();
    let fmt = detect_file_format(&db_path);
    assert_eq!(fmt.category, FileCategory::Database);

    // 7. Binary Null Bytes -> BinaryHex
    let bin_path = dir.path().join("binary.bin");
    File::create(&bin_path)
        .unwrap()
        .write_all(b"\x7fELF\x02\x01\x01\x00\x00\x00")
        .unwrap();
    let fmt = detect_file_format(&bin_path);
    assert_eq!(fmt.category, FileCategory::BinaryHex);
    assert!(fmt.is_binary);
}

#[test]
fn test_archive_path_traversal_safety() {
    assert!(is_safe_archive_path("valid/path/file.txt"));
    assert!(is_safe_archive_path("file.txt"));
    assert!(!is_safe_archive_path("../etc/passwd"));
    assert!(!is_safe_archive_path("foo/../../bar"));
    assert!(!is_safe_archive_path("/absolute/path/file.txt"));
}

#[test]
fn test_csv_tsv_data_preview() {
    let dir = tempdir().unwrap();

    let csv_path = dir.path().join("test.csv");
    File::create(&csv_path)
        .unwrap()
        .write_all(b"Col1,Col2,Col3\nVal1,Val2,Val3\nRow2_1,Row2_2,Row2_3")
        .unwrap();

    let data_prev = DataPreview::load(&csv_path, 10);
    match data_prev.kind {
        termexplorer::preview::data::DataPreviewKind::Table {
            headers,
            rows,
            total_rows,
            delimiter,
            ..
        } => {
            assert_eq!(delimiter, ',');
            assert_eq!(headers, vec!["Col1", "Col2", "Col3"]);
            assert_eq!(rows.len(), 2);
            assert_eq!(total_rows, 3);
        }
        _ => panic!("Expected Table preview for CSV"),
    }
}

#[test]
fn test_hex_viewer_rows_and_ascii() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.bin");
    File::create(&path)
        .unwrap()
        .write_all(b"Hello World 1234\x00\x01\x02\xFF")
        .unwrap();

    let hex_prev = HexPreview::load(&path, 10);
    assert!(!hex_prev.rows.is_empty());
    let first_row = &hex_prev.rows[0];
    assert_eq!(first_row.offset, 0);
    assert!(first_row.ascii.contains("Hello World"));
}

#[test]
fn test_detailed_metadata() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("sample.txt");
    File::create(&path)
        .unwrap()
        .write_all(b"Metadata content sample")
        .unwrap();

    let meta = DetailedMetadata::from_path(&path).expect("Metadata extraction");
    assert_eq!(meta.file_name, "sample.txt");
    assert_eq!(meta.size_bytes, 23);
    assert!(!meta.is_dir);
    assert!(meta.is_file);
    assert!(meta.permissions_str.starts_with('r'));
}

#[test]
fn test_preview_lru_cache() {
    let mut cache = PreviewLruCache::<String>::new(2);
    let p1 = PathBuf::from("/tmp/f1.txt");
    let p2 = PathBuf::from("/tmp/f2.txt");
    let p3 = PathBuf::from("/tmp/f3.txt");

    cache.insert(p1.clone(), None, "Content 1".into());
    cache.insert(p2.clone(), None, "Content 2".into());

    assert_eq!(cache.get(&p1, None).as_deref(), Some("Content 1"));

    // Insert 3rd, should evict p2 (since p1 was just touched)
    cache.insert(p3.clone(), None, "Content 3".into());
    assert!(cache.get(&p1, None).is_some());
    assert!(cache.get(&p3, None).is_some());
    assert!(cache.get(&p2, None).is_none());
}

#[test]
fn test_image_halfblock_rendering() {
    // Create an in-memory 20x20 RGB image
    let img = image::DynamicImage::new_rgb8(20, 20);
    let grid = GraphicsCapability::render_halfblocks(&img, 10, 5);
    assert!(!grid.is_empty());
    assert_eq!(grid[0][0].2, '▀');
}

#[test]
fn test_preview_manager_end_to_end() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("main.rs");
    File::create(&path)
        .unwrap()
        .write_all(b"fn hello() -> i32 { 42 }")
        .unwrap();

    let mut manager = PreviewManager::new(512 * 1024);
    let full = manager.load_file(&path, termexplorer::preview::PreviewTab::Preview, 80, 24);

    assert_eq!(full.format_info.category, FileCategory::SourceCode);
    assert!(full.source.is_some());
    assert!(full.hex.is_some());
    assert!(!full.actions.is_empty());
}
