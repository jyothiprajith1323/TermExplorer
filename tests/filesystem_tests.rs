use tempfile::tempdir;
use termexplorer::filesystem::{
    browser::{list_dir, SortOrder},
    metadata::{get_metadata, is_binary},
    operations::{
        copy_entry, copy_file, create_dir, create_file, delete_dir, delete_entry, delete_file,
        rename_entry, save_file_atomic,
    },
    search::{fuzzy_filter, substring_filter},
};

#[test]
fn test_create_and_delete_file() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");

    assert!(!file_path.exists());
    create_file(&file_path).unwrap();
    assert!(file_path.exists());

    // Creating already existing file should fail
    assert!(create_file(&file_path).is_err());

    delete_file(&file_path).unwrap();
    assert!(!file_path.exists());
}

#[test]
fn test_create_and_delete_dir() {
    let dir = tempdir().unwrap();
    let sub_dir = dir.path().join("subdir");

    assert!(!sub_dir.exists());
    create_dir(&sub_dir).unwrap();
    assert!(sub_dir.exists());

    // Nested delete
    let nested_file = sub_dir.join("inner.txt");
    create_file(&nested_file).unwrap();
    assert!(nested_file.exists());

    delete_dir(&sub_dir).unwrap();
    assert!(!sub_dir.exists());
}

#[test]
fn test_rename_entry() {
    let dir = tempdir().unwrap();
    let old_path = dir.path().join("old.txt");
    let new_path = dir.path().join("new.txt");

    create_file(&old_path).unwrap();
    assert!(old_path.exists());

    rename_entry(&old_path, &new_path).unwrap();
    assert!(!old_path.exists());
    assert!(new_path.exists());

    // Destination collision
    let another = dir.path().join("another.txt");
    create_file(&another).unwrap();
    assert!(rename_entry(&new_path, &another).is_err());
}

#[test]
fn test_copy_file_and_entry() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let dst = dir.path().join("dst.txt");

    save_file_atomic(&src, b"Hello World").unwrap();
    copy_file(&src, &dst).unwrap();

    assert_eq!(std::fs::read_to_string(&dst).unwrap(), "Hello World");

    // Test recursive copy_entry
    let src_dir = dir.path().join("src_folder");
    create_dir(&src_dir).unwrap();
    let file1 = src_dir.join("f1.txt");
    save_file_atomic(&file1, b"File 1").unwrap();

    let dst_dir = dir.path().join("dst_folder");
    copy_entry(&src_dir, &dst_dir).unwrap();

    assert!(dst_dir.join("f1.txt").exists());
    assert_eq!(
        std::fs::read_to_string(dst_dir.join("f1.txt")).unwrap(),
        "File 1"
    );
}

#[test]
fn test_delete_entry() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("del_file.txt");
    create_file(&file).unwrap();
    delete_entry(&file).unwrap();
    assert!(!file.exists());

    let folder = dir.path().join("del_folder");
    create_dir(&folder).unwrap();
    let inner = folder.join("inner.txt");
    create_file(&inner).unwrap();
    delete_entry(&folder).unwrap();
    assert!(!folder.exists());
}

#[test]
fn test_list_dir_and_hidden_files() {
    let dir = tempdir().unwrap();
    create_file(&dir.path().join("visible.txt")).unwrap();
    create_file(&dir.path().join(".hidden.txt")).unwrap();
    create_dir(&dir.path().join("subdir")).unwrap();

    // Hidden = false
    let entries = list_dir(dir.path(), false, SortOrder::Name).unwrap();
    assert_eq!(entries.len(), 2); // subdir and visible.txt
    assert!(entries[0].is_dir()); // Dirs first

    // Hidden = true
    let entries_all = list_dir(dir.path(), true, SortOrder::Name).unwrap();
    assert_eq!(entries_all.len(), 3);
}

#[test]
fn test_sorting_orders() {
    let dir = tempdir().unwrap();
    let a = dir.path().join("a.txt");
    let b = dir.path().join("b.txt");

    save_file_atomic(&a, b"short").unwrap();
    save_file_atomic(&b, b"a very long content in file b").unwrap();

    let by_name = list_dir(dir.path(), false, SortOrder::Name).unwrap();
    assert_eq!(by_name[0].name, "a.txt");

    let by_name_desc = list_dir(dir.path(), false, SortOrder::NameDesc).unwrap();
    assert_eq!(by_name_desc[0].name, "b.txt");

    let by_size = list_dir(dir.path(), false, SortOrder::Size).unwrap();
    assert_eq!(by_size[0].name, "a.txt");

    let by_size_desc = list_dir(dir.path(), false, SortOrder::SizeDesc).unwrap();
    assert_eq!(by_size_desc[0].name, "b.txt");
}

#[test]
fn test_metadata_and_binary_detection() {
    let dir = tempdir().unwrap();
    let text_file = dir.path().join("doc.md");
    save_file_atomic(&text_file, b"# Heading\nSome text content.").unwrap();

    let meta = get_metadata(&text_file).unwrap();
    assert_eq!(meta.extension.as_deref(), Some("md"));
    assert_eq!(meta.mime_guess, "text/markdown");
    assert!(!meta.is_dir);

    assert!(!is_binary(b"Just simple text lines\nWith normal chars."));
    // Binary bytes with null bytes
    assert!(is_binary(&[
        0x7F, 0x45, 0x4C, 0x46, 0x02, 0x01, 0x01, 0x00, 0x00, 0x00
    ]));
}

#[test]
fn test_fuzzy_and_substring_filter() {
    let dir = tempdir().unwrap();
    create_file(&dir.path().join("main.rs")).unwrap();
    create_file(&dir.path().join("cargo.toml")).unwrap();
    create_file(&dir.path().join("readme.md")).unwrap();

    let entries = list_dir(dir.path(), false, SortOrder::Name).unwrap();

    let filtered = fuzzy_filter(&entries, "main");
    assert!(!filtered.is_empty());
    assert_eq!(entries[filtered[0].1].name, "main.rs");

    let sub = substring_filter(&entries, "cargo");
    assert_eq!(sub.len(), 1);
    assert_eq!(sub[0].name, "cargo.toml");
}
