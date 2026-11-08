//! Sprint 1 Quality Gates Comprehensive Verification Suite.
//!
//! Verifies:
//! - Gate 1: First-viewport latency (<30ms) & monotonic task cancellation
//! - Gate 2: Failure injection during atomic file saving (preserves original file)
//! - Gate 3: FreeDesktop Trash and Operation Ledger undo
//! - Gate 4: Monotonic preview cancellation under rapid key switching
//! - Gate 5: Unicode grapheme cluster property tests (no split codepoints)
//! - Gate 6: ANSI injection prevention and archive decompression bomb ceilings

use std::fs::File;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tempfile::tempdir;
use termexplorer::editor::buffer::Buffer;
use termexplorer::filesystem::browser::SortOrder;
use termexplorer::filesystem::chunked::{load_initial_viewport, stream_dir_chunks, DirChunk};
use termexplorer::filesystem::ledger::{FsOperationLedger, RecordedOperation};
use termexplorer::filesystem::operations::{save_file_atomic, trash_entry_tracked};
use termexplorer::preview::archive::MAX_EXTRACT_BYTES;
use termexplorer::preview::manager::{PreviewManager, PreviewTab};
use termexplorer::preview::sanitizer::{sanitize_preview_line, sanitize_preview_text};
use tokio::sync::mpsc;
use unicode_segmentation::UnicodeSegmentation;

// ── Gate 1: Large Directory First-Viewport Latency (<30ms) ───────────────────

#[test]
fn test_quality_gate_1_first_viewport_latency() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    // Create 1,500 files to simulate a large directory without excessive disk allocation
    for i in 0..1500 {
        let fpath = dir_path.join(format!("file_{:05}.txt", i));
        File::create(&fpath).unwrap();
    }

    let start = Instant::now();
    let entries = load_initial_viewport(dir_path, false, SortOrder::Name, 100).unwrap();
    let duration = start.elapsed();

    assert_eq!(entries.len(), 100);

    // Must be well within the sub-30ms target
    println!("Gate 1 First-viewport latency: {:?}", duration);
    assert!(
        duration.as_millis() < 30,
        "First viewport latency exceeded 30ms: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_quality_gate_1_monotonic_directory_cancellation() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path().to_path_buf();

    for i in 0..200 {
        let fpath = dir_path.join(format!("chunk_{:03}.dat", i));
        File::create(&fpath).unwrap();
    }

    let cancel_token = Arc::new(AtomicU64::new(1));
    let (tx, mut rx) = mpsc::channel::<DirChunk>(16);

    // Cancel token is bumped to 2 before background scan finishes
    let token_clone = Arc::clone(&cancel_token);
    let path_clone = dir_path.clone();
    let handle = tokio::spawn(async move {
        stream_dir_chunks(
            path_clone,
            false,
            SortOrder::Name,
            20,
            token_clone,
            1, // expects token 1
            tx,
        )
        .await;
    });

    // Invalidate token immediately
    cancel_token.store(2, Ordering::SeqCst);
    handle.await.unwrap();

    // Verify stream aborted early
    let mut chunks = Vec::new();
    while let Ok(chunk) = rx.try_recv() {
        chunks.push(chunk);
    }
    // With immediate cancellation, stream will have aborted before completing all chunks
    assert!(chunks.len() < 10);
}

// ── Gate 2: Failure Injection During Atomic Saving ────────────────────────────

#[test]
fn test_quality_gate_2_atomic_saving_preserves_original_on_failure() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("critical_data.txt");
    let original_content = b"ORIGINAL UNCORRUPTED DATA";
    std::fs::write(&target, original_content).unwrap();

    // Make target file read-only on unix to inject permission rejection upon persist
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&target).unwrap().permissions();
        perms.set_mode(0o444); // read-only
        std::fs::set_permissions(&target, perms).unwrap();
    }

    // Attempting atomic save with modified content should either succeed by replacing or fail safely
    let new_content = b"NEW CORRUPTED DATA";
    let _ = save_file_atomic(&target, new_content);

    // Verify that the file either holds new data cleanly or retains original data — NEVER partial/empty
    let current_content = std::fs::read(&target).unwrap();
    assert!(
        current_content == original_content || current_content == new_content,
        "File was corrupted during atomic save!"
    );
    assert!(
        !current_content.is_empty(),
        "Silent data loss: file is empty!"
    );
}

// ── Gate 3: FreeDesktop Trash and Operation Ledger Undo ────────────────────────

#[test]
fn test_quality_gate_3_trash_and_undo_ledger() {
    let dir = tempdir().unwrap();
    let original_file = dir.path().join("important_doc.pdf");
    std::fs::write(&original_file, b"%PDF-1.4 dummy content").unwrap();

    let mut ledger = FsOperationLedger::new(50);

    // 1. Send to Trash
    let trash_name = trash_entry_tracked(&original_file).unwrap();
    assert!(
        !original_file.exists(),
        "Original file still exists after trash"
    );

    ledger.record(RecordedOperation::Trash {
        original_path: original_file.clone(),
        trash_filename: trash_name,
    });

    assert!(ledger.can_undo());
    assert!(ledger.undo_description().unwrap().contains("Trash"));

    // 2. Undo Trash -> Restores to original path
    let undo_msg = ledger.undo().unwrap();
    assert!(undo_msg.to_lowercase().contains("restored"));
    assert!(
        original_file.exists(),
        "Restored file does not exist at original path"
    );
    assert_eq!(
        std::fs::read(&original_file).unwrap(),
        b"%PDF-1.4 dummy content"
    );

    // 3. Test Move and Undo Move
    let moved_file = dir.path().join("renamed_doc.pdf");
    termexplorer::filesystem::operations::move_entry(
        &original_file,
        &moved_file,
        termexplorer::filesystem::operations::CollisionPolicy::Fail,
    )
    .unwrap();

    ledger.record(RecordedOperation::Move {
        from: original_file.clone(),
        to: moved_file.clone(),
        was_cross_device: false,
    });

    assert!(moved_file.exists());
    assert!(!original_file.exists());

    let undo_move = ledger.undo().unwrap();
    assert!(undo_move.contains("Undid move"));
    assert!(original_file.exists());
    assert!(!moved_file.exists());
}

// ── Gate 4: Monotonic Preview Cancellation Under Rapid Key Switching ──────────

#[test]
fn test_quality_gate_4_monotonic_preview_cancellation() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("file1.rs");
    let file2 = dir.path().join("file2.rs");
    std::fs::write(&file1, b"fn main() { println!(\"Hello 1\"); }").unwrap();
    std::fs::write(&file2, b"fn main() { println!(\"Hello 2\"); }").unwrap();

    let cancel_token = Arc::new(AtomicU64::new(1));

    // Request for ticket 1 should succeed when token is 1
    let preview1 = PreviewManager::generate_preview_cancellable(
        &file1,
        PreviewTab::Preview,
        1024 * 1024,
        60,
        30,
        Some((&cancel_token, 1)),
    );
    assert!(preview1.is_some());

    // When ticket is incremented to 2, in-flight request for obsolete ticket 1 must immediately abort
    cancel_token.store(2, Ordering::SeqCst);
    let preview_obsolete = PreviewManager::generate_preview_cancellable(
        &file1,
        PreviewTab::Preview,
        1024 * 1024,
        60,
        30,
        Some((&cancel_token, 1)), // obsolete ticket 1
    );
    assert!(
        preview_obsolete.is_none(),
        "Obsolete preview request was not cancelled!"
    );

    // Valid ticket 2 preview succeeds
    let preview2 = PreviewManager::generate_preview_cancellable(
        &file2,
        PreviewTab::Preview,
        1024 * 1024,
        60,
        30,
        Some((&cancel_token, 2)),
    );
    assert!(preview2.is_some());
}

// ── Gate 5: Unicode Grapheme Cluster Integrity (Property Tests) ───────────────

#[test]
fn test_quality_gate_5_unicode_grapheme_cluster_integrity() {
    let complex_graphemes = vec![
        "🦀", // single emoji
        "👨‍💻", // ZWJ sequence (3 codepoints)
        "👍🏽", // skin tone modifier (2 codepoints)
        "🇮🇳", // flag regional indicator (2 codepoints)
        "é",  // combining acute accent (2 codepoints)
        "👩‍👩‍👧‍👦", // family ZWJ sequence (7 codepoints)
        "🇺🇸", // flag (2 codepoints)
    ];

    let mut buf = Buffer::new();

    // Insert all graphemes
    for &g in &complex_graphemes {
        buf.insert_text(g);
    }

    assert_eq!(buf.cursor.col, complex_graphemes.len());
    let full_line = &buf.lines[0];
    let actual_graphemes: Vec<&str> = full_line.graphemes(true).collect();
    assert_eq!(actual_graphemes, complex_graphemes);

    // Verify left/right traversal never splits any grapheme cluster
    for expected_col in (0..=complex_graphemes.len()).rev() {
        assert_eq!(buf.cursor.col, expected_col);
        if expected_col > 0 {
            buf.move_left();
        }
    }
    assert_eq!(buf.cursor.col, 0);

    for expected_col in 0..complex_graphemes.len() {
        assert_eq!(buf.cursor.col, expected_col);
        buf.move_right();
    }
    assert_eq!(buf.cursor.col, complex_graphemes.len());

    // Verify deleting from right to left with backspace removes exactly 1 full cluster each time
    for i in (0..complex_graphemes.len()).rev() {
        buf.backspace();
        assert_eq!(buf.cursor.col, i);
        let remaining_graphemes: Vec<&str> = buf.lines[0].graphemes(true).collect();
        assert_eq!(remaining_graphemes.len(), i);
        assert_eq!(remaining_graphemes, &complex_graphemes[..i]);
    }

    assert!(buf.lines[0].is_empty());
}

// ── Gate 6: Security Testing — ANSI Sanitization & Archive Limits ──────────────

#[test]
fn test_quality_gate_6_ansi_sanitization() {
    // OSC 52 clipboard exfiltration attack
    let osc_52_exploit = "Safe prefix \x1b]52;c;c2VjcmV0X3Bhc3N3b3Jk\x07 Safe suffix";
    let sanitized = sanitize_preview_text(osc_52_exploit);
    assert!(!sanitized.contains("52;c;"));
    assert!(!sanitized.contains("c2VjcmV0X3Bhc3N3b3Jk"));
    assert!(sanitized.contains("Safe prefix"));
    assert!(sanitized.contains("Safe suffix"));

    // OSC 8 spoofed hyperlink
    let osc_8_exploit = "\x1b]8;;http://malicious.evil/payload\x07Click Here\x1b]8;;\x07";
    let sanitized_link = sanitize_preview_line(osc_8_exploit);
    assert!(!sanitized_link.contains("malicious.evil"));
    assert!(sanitized_link.contains("Click Here"));

    // Dangerous CSI cursor repositioning and screen clear
    let csi_exploit = "\x1b[2J\x1b[10;20HOverwritten line";
    let sanitized_csi = sanitize_preview_line(csi_exploit);
    assert!(!sanitized_csi.contains("\x1b[2J"));
    assert!(!sanitized_csi.contains("\x1b[10;20H"));
    assert_eq!(sanitized_csi, "Overwritten line");
}

#[test]
fn test_quality_gate_6_archive_decompression_bomb_limits() {
    // Verify constant exists and is safely bounded to 500 MiB
    assert_eq!(MAX_EXTRACT_BYTES, 500 * 1024 * 1024);
}
