# Filesystem Subsystem Architecture

The Filesystem subsystem (`src/filesystem/`) manages directory traversal, entry classification, file mutations, chunked streaming, and operation logging.

---

## 1. Subsystem Components

```text
src/filesystem/
├── browser.rs       # Directory entry model (DirEntry, EntryKind, SortOrder)
├── chunked.rs       # Chunked directory viewport loader & background streaming
├── ledger.rs        # FsOperationLedger: mutation audit log & undo support
├── metadata.rs      # POSIX permissions, size formatting, hash calculation
├── operations.rs    # Create, rename, delete, move, atomic save, collision handling
├── search.rs        # Fast inline fuzzy & substring filtering
└── watcher.rs       # notify-rs background filesystem event watcher
```

---

## 2. Chunked Viewport Loading (`chunked.rs`)

Directories in developer environments can contain hundreds of thousands of files (e.g. `node_modules`, `target`, build caches). Traditional synchronous directory reading (`fs::read_dir`) blocks the UI thread for seconds.

TermExplorer solves this via a two-phase chunked pipeline:
1. **Initial Viewport Loading (`load_initial_viewport`)**:
   - Immediately scans the first 100 entries.
   - Populates the UI within **<10 milliseconds**, allowing instant interaction.
2. **Background Chunk Streaming (`stream_dir_chunks`)**:
   - Asynchronous worker iterates remaining directory entries in chunks of 500.
   - Delivers chunks via Tokio `mpsc` channels to the main state.
   - If the user navigates away before streaming completes, a monotonic cancellation token aborts directory scanning immediately.

---

## 3. Operations Ledger & FreeDesktop Trash (`ledger.rs`, `operations.rs`)

Every file mutation passes through safe abstractions:
- **Ledger Recording (`FsOperationLedger`)**: All mutations (moves, renames, deletions, creations) are logged with source paths, destination paths, and timestamps.
- **Trash Protocol**: Soft deletion dispatches to FreeDesktop Trash (`gio trash` or `trash-cli`). If external utilities are unavailable, TermExplorer archives items into `.termexplorer/trash/` alongside a JSON metadata manifest, enabling recovery.
- **Collision Resolution (`CollisionPolicy`)**: When moving or copying into an existing destination:
  - `Overwrite`: Replaces the destination file.
  - `AutoRename`: Appends an incremental counter (e.g. `file_1.rs`).
  - `Cancel`: Aborts operation without modifying destination.

---

## 4. Atomic File Persistence (`save_file_atomic`)

To prevent data corruption on sudden power failure or crashes during file writing:
```rust
pub fn save_file_atomic(path: &Path, content: &[u8]) -> Result<()> {
    let parent = path.parent().unwrap_or(Path::new("."));
    let tmp_path = parent.join(format!(".{}.tmp.{}", filename, std::process::id()));
    
    // 1. Write content to temporary sibling file
    let mut file = File::create(&tmp_path)?;
    file.write_all(content)?;
    
    // 2. Flush file buffers to physical storage
    file.sync_all()?;
    
    // 3. Atomically rename over target file
    std::fs::rename(&tmp_path, path)?;
    
    // 4. Sync parent directory entry
    if let Ok(dir) = File::open(parent) {
        let _ = dir.sync_all();
    }
    Ok(())
}
```
