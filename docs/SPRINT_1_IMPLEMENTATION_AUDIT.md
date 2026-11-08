# TermExplorer — Sprint 1 Implementation Audit

**Date:** 2026-11-08  
**Auditor:** Principal Rust Systems Engineer & Software Reliability Specialist  
**Repository:** `jyothiprajith1323/TermExplorer`  
**Git Baseline:** `master` (`4162ac7`)  
**Product Strategy Reference:** *TermExplorer: Product Strategy, Systems Architecture, and Execution Roadmap for a Rust Terminal Developer Workstation*

---

## 1. Executive Summary

This audit independently inspects and benchmarks the TermExplorer codebase following the successful merging of all upstream dependencies (`dirs 6.0`, `sha2 0.11`, `zip 9.3`, `toml 1.2`, `ratatui 0.30`) and GitHub Actions CI/CD workflows. 

The application architecture has reached a mature V1.1.0-RC2 state with 100% passing tests (114+ automated tests across 24 suites), strict Zero-Warning Clippy compliance, and verified workstation layout (25% Explorer / 75% Preview, 15% compact Terminal drawer hidden by default).

However, rigorous inspection of the runtime event loop, filesystem pipeline, and editor reveals critical architectural gaps that prevent the application from scaling safely to 100,000+ file trees, concurrent background previews, and non-blocking responsive workflows.

---

## 2. Research Requirement Gap Matrix

| ID | Requirement | Status | Code Evidence | Test Evidence | Gap Description |
|:---|:---|:---:|:---|:---|:---|
| **GAP-01** | Monotonic task cancellation and chunked directory loading | **PARTIAL** | `src/filesystem/browser.rs:178` (`list_dir`), `src/preview/manager.rs:96` (`load_file`), `src/app.rs:452` | `tests/performance_benchmarks.rs`, `tests/preview_tests.rs` | Directory listing and file preview generation are executed synchronously on the main thread inside the event loop. Directory entries are not chunked/streamed. Preview generation lacks a monotonic ticket/generation counter to abort stale in-flight decodes. |
| **GAP-02** | Atomic saving and crash recovery | **COMPLETE** | `src/filesystem/operations.rs:340` (`save_file_atomic`), `src/editor/buffer.rs:753` (`recovery_path_for`), `src/editor/buffer.rs:767` (`create_recovery_snapshot`) | `tests/reliability_and_safety_tests.rs:test_atomic_file_saving_preserves_content`, `test_editor_unsaved_buffer_recovery_snapshot` | Tempfile creation in same target directory with `sync_all()`, permission preservation, atomic replace, parent directory `fsync`, and SHA-256 recovery snapshot generation are implemented and tested. (Needs UI prompt integration on startup when recovery file is found). |
| **GAP-03** | FreeDesktop trash and undoable file operations | **PARTIAL** | `src/filesystem/operations.rs:173` (`trash_entry`), `src/filesystem/operations.rs:279` (`restore_from_trash`), `src/filesystem/operations.rs:68` (`move_entry`) | `tests/reliability_and_safety_tests.rs:test_trash_and_restore_freedesktop_metadata` | FreeDesktop trash with `.trashinfo` metadata and cross-filesystem copy-verify-delete rollback are verified. However, an operation ledger with an undo stack for reversible Move/Rename/Copy/Trash operations is currently missing. |
| **GAP-04** | ANSI sanitization and archive symlink containment | **PARTIAL** | `src/preview/archive/mod.rs:238` (`is_safe_archive_path`), `src/preview/source/mod.rs:34` | `tests/adversarial_filesystem_tests.rs:test_path_traversal_archive_sanitization`, `tests/preview_tests.rs:test_archive_path_traversal_safety` | Zip Slip and `..` traversal are guarded. However, raw untrusted text in previews is rendered without sanitizing terminal control escape codes (OSC, CSI, title, clipboard). Extraction lacks decompression bomb bounds (max bytes / file counts) and symlink target escape containment. |
| **GAP-05** | Unicode grapheme handling | **PARTIAL** | `src/fonts/unicode.rs:23` (`slice_by_display_columns`), `src/fonts/unicode.rs:11` (`str_display_width`), `src/editor/buffer.rs:426` | `tests/unicode_and_emoji_tests.rs` (8 tests) | Grapheme display width calculation, safe column slicing, and multi-tier icon fallbacks are implemented in `src/fonts/unicode.rs`. However, the code editor buffer (`src/editor/buffer.rs`) still uses `chars()` indexing instead of grapheme cluster boundaries for cursor movements, deletions, and backspacing. |
| **GAP-06** | Filesystem watcher | **PARTIAL** | `src/filesystem/watcher.rs:10` (`watch_directory`), `src/app.rs:150` | `tests/filesystem_tests.rs` | Non-recursive `notify` watcher is functional, but lacks debouncing and event coalescing. Bursts of changes (e.g., git checkouts or compiler builds) trigger multiple redundant full `refresh_dir()` calls and complete re-reads. External modification detection on active editor buffers is missing. |
| **GAP-07** | Encoding detection and CRLF preservation | **PARTIAL** | `src/editor/buffer.rs:140` (`from_file`), `src/editor/buffer.rs:774` (`save_force`) | `tests/editor_integration_tests.rs:test_line_endings_detection_and_preservation` | Line ending detection (LF vs. CRLF) and preservation on save are verified. However, UTF-8 BOM detection/stripping and fallback decoding for non-UTF8 text files (Latin-1/CP1252) are missing, resulting in load failures on valid legacy text files. |
| **GAP-08** | Shell integration | **PARTIAL** | `src/shell/pty.rs:37` (`PtySession`), `src/shell/session.rs:15`, `src/main.rs:20` | `tests/terminal_tests.rs`, `tests/terminal_edge_tests.rs` | Interactive PTY lifecycle, terminal drawer resizing, and scrollback buffering are verified. CLI flags `--health`, `--cwd-file`, and `--chooser-file` are not yet wired up. OSC 7 working directory synchronisation between the shell and the file explorer is missing. |

---

## 3. Subsystem Architectural Analysis

### 3.1. Main Loop & Event Architecture (`src/app.rs`, `src/events.rs`)
- **Current State:** The main loop uses a single `App` struct owning state, receiving events from crossterm via an unbounded channel.
- **Problem:** Filesystem directory reading (`list_dir`) and file preview loading (`preview_manager.load_file`) occur synchronously on the main thread during navigation. When navigating into a folder with 10,000+ files or selecting large files, the main UI thread freezes until disk I/O completes.
- **Remedy:** Implement asynchronous chunked directory loading and background preview generation with monotonic cancellation tokens and generation counters.

### 3.2. Directory Loader & Filesystem Operations (`src/filesystem/`)
- **Current State:** `list_dir` in `browser.rs` reads all directory entries and calls `std::fs::metadata` synchronously.
- **Problem:** Does not support chunked streaming. Navigation into massive directories blocks UI responsiveness.
- **Remedy:** Implement chunked batch loading (initial fast batch of 100 items for immediate viewport rendering, followed by background streaming of remaining items). Prioritize metadata for visible items.

### 3.3. Preview Engine & Monotonic Cancellation (`src/preview/`)
- **Current State:** `PreviewManager` has an LRU cache (64 entries) and format detection, but loads previews synchronously in the caller thread.
- **Problem:** When navigating rapidly with arrow keys, previous preview tasks cannot be cancelled. Obsolete preview jobs compute unneeded syntax highlighting, PDF parsing, or image halfblocking.
- **Remedy:** Introduce a `PreviewTaskScheduler` with atomic monotonic ticket generation. When a file is selected, increment ticket ID, cancel existing async preview worker, and discard any completed payload whose ticket does not match the current ticket.

### 3.4. Filesystem Operation Ledger & Undo (`src/filesystem/operations.rs`)
- **Current State:** Functions for `create_file`, `create_dir`, `move_entry`, `copy_entry`, `trash_entry`, and `restore_from_trash` exist.
- **Problem:** No transaction log or history ledger exists. Users cannot undo accidental file moves, copies, renames, or trashings.
- **Remedy:** Introduce `FsOperationLedger` with `RecordedOperation` records (`Move { from, to }`, `Rename { from, to }`, `Copy { created }`, `Trash { original, trash_name }`) allowing structured, safe undo.

### 3.5. Security: Untrusted Preview ANSI Sanitization & Archive Bounds
- **Current State:** Text files are syntax highlighted via `syntect` or read raw. Archives check path traversal via `is_safe_archive_path`.
- **Problem:** Malicious files containing terminal escape sequences (OSC 52 clipboard hijacking, window title spoofing, alternate screen switching) can inject terminal codes into the workstation TUI. Archives have no decompression bomb bounds (max bytes / file counts).
- **Remedy:** Implement strict ANSI sanitization filter for untrusted text in previews. Enforce decompression ceilings (e.g. 100 MB max uncompressed extraction size, max 10,000 files) in `extract_archive`.

### 3.6. Unicode & Grapheme Cluster Editor Fidelity (`src/editor/buffer.rs`)
- **Current State:** `src/fonts/unicode.rs` has display width calculations and slicing by display columns. `src/editor/buffer.rs` still operates on `Vec<char>`.
- **Problem:** Complex multi-codepoint grapheme clusters (flags, skin-tone modifiers, ZWJ sequences) have their codepoints split when moving cursor or deleting single characters.
- **Remedy:** Migrate `TextBuffer` cursor indexing and deletion primitives to grapheme clusters using `unicode-segmentation`.

### 3.7. Filesystem Watching Debounce (`src/filesystem/watcher.rs`)
- **Current State:** Spawns a non-recursive `notify` watcher. Every event is passed through directly.
- **Problem:** Rapid file writes cause storms of `AppEvent::FsChange`, each triggering a full `refresh_dir()`.
- **Remedy:** Introduce an async debouncer (e.g. 100–250ms quiet window) that coalesces multiple filesystem notifications into a single update.

### 3.8. File Encoding & BOM Support (`src/editor/buffer.rs`)
- **Current State:** Loads files strictly with `std::fs::read_to_string`.
- **Problem:** UTF-8 BOM (`\u{feff}`) causes column offset errors. Non-UTF-8 text files fail to load.
- **Remedy:** Read file as raw bytes, detect UTF-8 BOM (strip on load, remember flag to re-add on save), detect Latin-1/Windows-1252 fallback for valid single-byte text, and surface encoding name in editor status bar.

### 3.9. Shell Integration & CLI Flags (`src/main.rs`, `src/shell/`)
- **Current State:** CLI only handles `--help` and `--version`.
- **Problem:** Missing `--health`, `--cwd-file <path>`, and `--chooser-file <path>`. No OSC 7 directory tracking.
- **Remedy:** Implement `--health` system diagnostic check, `--cwd-file` for shell wrapper directory synchronization, and `--chooser-file` for file picker mode. Implement OSC 7 sequence parsing in PTY reader.

---

## 4. Execution Plan for Sprint 1

1. **Step 1:** Implement Monotonic Preview Generation & Cancellation (`GAP-01`)
2. **Step 2:** Implement Chunked & Asynchronous Directory Loading (`GAP-01`)
3. **Step 3:** Implement Filesystem Operation Ledger & Safe Undo (`GAP-03`)
4. **Step 4:** Implement ANSI Escape Code Sanitizer & Archive Extraction Ceilings (`GAP-04`)
5. **Step 5:** Implement Grapheme-Aware Editor Navigation and Deletion (`GAP-05`)
6. **Step 6:** Implement Debounced Filesystem Watching & External Modification Handling (`GAP-06`)
7. **Step 7:** Implement BOM & Fallback Encoding Handling in Editor (`GAP-07`)
8. **Step 8:** Implement Shell CLI flags (`--health`, `--cwd-file`, `--chooser-file`) and OSC 7 integration (`GAP-08`)
9. **Step 9:** Execute Performance Benchmarks (Synthetic 100,000 file directory, latency targets)
10. **Step 10:** Produce Comprehensive Verification & Architecture Documentation
