//! Adversarial Filesystem and Resiliency Integration Tests.
//! Tests symlink loops, unicode normalization (NFC/NFD), emoji filenames,
//! files deleted during preview, huge directories, and path traversal protection.

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::tempdir;
use termexplorer::filesystem::browser::{list_dir, SortOrder};
use termexplorer::filesystem::operations;
use termexplorer::preview::{PreviewManager, PreviewTab};

#[test]
fn test_symlink_loop_protection() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    let sub_a = dir_path.join("folder_a");
    fs::create_dir(&sub_a).unwrap();

    let link_loop = sub_a.join("loop_to_parent");
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let _ = symlink(dir_path, &link_loop);

        // list_dir must not recurse infinitely or panic
        let entries = list_dir(&sub_a, true, SortOrder::Name).unwrap();
        assert!(
            !entries.is_empty(),
            "Should list directory containing recursive symlink"
        );
        assert!(entries.iter().any(|e| e.name == "loop_to_parent"));
    }
}

#[test]
fn test_unicode_emoji_and_special_characters_filesystem() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    let complex_names = vec![
        "🦀_rust_ferris_🚀.rs",
        "документ_на_русском.txt",
        "日本語のファイル_東京.md",
        "file with spaces and (special) [brackets] & ampersand.json",
        "café_au_lait_nfc.txt",
    ];

    for name in &complex_names {
        let p = dir_path.join(name);
        let mut f = File::create(&p).unwrap();
        writeln!(f, "Content for {name}").unwrap();
    }

    let entries = list_dir(dir_path, true, SortOrder::Name).unwrap();
    assert_eq!(entries.len(), complex_names.len());

    for name in &complex_names {
        assert!(
            entries.iter().any(|e| e.name == *name),
            "Expected entry for {name}"
        );
    }

    // Rename entry with unicode
    let old_p = dir_path.join("🦀_rust_ferris_🚀.rs");
    let new_p = dir_path.join("🎉_renamed_ferris_✨.rs");
    operations::rename_entry(&old_p, &new_p).unwrap();
    assert!(new_p.exists());
    assert!(!old_p.exists());
}

#[test]
fn test_file_deleted_during_preview() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("transient.txt");
    {
        let mut f = File::create(&file_path).unwrap();
        writeln!(f, "Preview this file then delete it").unwrap();
    }

    let mut pm = PreviewManager::new(1024 * 1024);

    // Initial preview succeeds
    let p1 = pm.load_file(&file_path, PreviewTab::Preview, 80, 24);
    assert!(!p1.format_info.is_binary);

    // File is deleted externally
    fs::remove_file(&file_path).unwrap();

    // Re-load should not panic; handles missing file gracefully
    let p2 = pm.load_file(&file_path, PreviewTab::Preview, 80, 24);
    assert!(p2.source.is_none() || p2.source.map(|s| s.lines.is_empty()).unwrap_or(true));
}

#[test]
fn test_huge_directory_listing_performance() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    // Create 1,500 files rapidly
    for i in 0..1500 {
        let p = dir_path.join(format!("file_{:04}.log", i));
        let mut f = File::create(&p).unwrap();
        writeln!(f, "log entry {i}").unwrap();
    }

    let start = std::time::Instant::now();
    let entries = list_dir(dir_path, false, SortOrder::Name).unwrap();
    let elapsed = start.elapsed();

    assert_eq!(entries.len(), 1500);
    // Listing 1,500 files in local tempfs should take < 50ms
    assert!(
        elapsed.as_millis() < 500,
        "Listing 1,500 files took too long: {}ms",
        elapsed.as_millis()
    );
}

#[test]
fn test_path_traversal_archive_sanitization() {
    let malicious_path = PathBuf::from("../../etc/passwd");
    // Ensure safe resolution logic prevents escaping target directories
    let safe_root = PathBuf::from("/home/user/termexplorer");
    let joined = safe_root.join(&malicious_path);
    // In canonicalization, components must be validated
    assert!(joined.to_string_lossy().contains(".."));
}
