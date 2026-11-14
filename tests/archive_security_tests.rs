use std::fs::File;
use std::io::Write;
use tempfile::tempdir;
use termexplorer::preview::archive::{
    is_safe_archive_path, ArchivePreview, MAX_EXTRACT_BYTES, MAX_EXTRACT_FILES,
};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

#[test]
fn test_archive_path_traversal_validation() {
    // Valid relative paths
    assert!(is_safe_archive_path("file.txt"));
    assert!(is_safe_archive_path("dir/subdir/file.rs"));
    assert!(is_safe_archive_path("assets/images/logo.png"));

    // Dangerous traversal paths (Zip Slip)
    assert!(!is_safe_archive_path("../etc/passwd"));
    assert!(!is_safe_archive_path("dir/../../secret.txt"));
    assert!(!is_safe_archive_path("/etc/shadow"));
    assert!(!is_safe_archive_path("/root/.ssh/authorized_keys"));
    assert!(!is_safe_archive_path(".."));
}

#[test]
fn test_archive_zip_slip_prevention_on_extraction() {
    let dir = tempdir().expect("tempdir");
    let zip_path = dir.path().join("malicious.zip");
    let target_extract_dir = dir.path().join("extracted");

    // Construct a zip file containing both a normal file and a Zip Slip traversal attempt
    let file = File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // Safe entry
    zip.start_file("safe.txt", options).unwrap();
    zip.write_all(b"Hello safe world").unwrap();

    // Malicious entry attempting to escape to parent directory
    zip.start_file("../slip.txt", options).unwrap();
    zip.write_all(b"Malicious payload").unwrap();

    zip.finish().unwrap();

    let preview = ArchivePreview::load(&zip_path, 100);
    assert_eq!(preview.total_entries, 2);
    assert!(
        preview.has_unsafe_entries,
        "ArchivePreview must detect unsafe traversal entries"
    );

    // Extraction should safely skip or block the path traversal without writing outside target
    let result = preview.extract_all(&zip_path, &target_extract_dir);
    assert!(result.is_ok(), "Extraction should complete without error");

    // Verify safe file exists
    assert!(target_extract_dir.join("safe.txt").exists());

    // Verify malicious file was NOT written outside target directory
    assert!(!dir.path().join("slip.txt").exists());
}

#[test]
fn test_archive_safety_limits() {
    assert_eq!(MAX_EXTRACT_BYTES, 500 * 1024 * 1024);
    assert_eq!(MAX_EXTRACT_FILES, 10_000);
}
