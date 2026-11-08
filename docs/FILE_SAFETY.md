# TermExplorer File Safety Architecture

## Overview

Filesystem operations in TermExplorer are built on zero-data-loss principles. Every mutation—including file writes, moves, renames, deletions, and copies—is subject to pre-flight verification, atomic replacement semantics, and transaction logging.

---

## 1. Atomic File Saving (`save_file_atomic`)

TermExplorer never overwrites existing user files directly with standard truncated open calls. Direct overwriting risks silent data corruption or empty files if the write is interrupted by a system crash, power failure, or disk-full condition.

### Implementation Protocol (`src/filesystem/operations.rs`)
1. **Sibling Temporary File**: Creates a temporary file (`tempfile::NamedTempFile::new_in`) in the target file's parent directory, guaranteeing the temp file resides on the identical filesystem volume.
2. **Buffer Flush & Sync**: Writes content completely, flushes internal user-space buffers, and executes `tmp.as_file().sync_all()` to ensure all blocks and metadata are committed to storage media.
3. **Permission Preservation**: Preserves original POSIX file mode bits (`0o644`, `0o755`, etc.) from the pre-existing target file.
4. **Atomic Rename (`persist`)**: Atomically renames the temporary file over the target path. In Linux and POSIX kernels, `rename(2)` is guaranteed to be atomic: either the new file replaces the old file completely, or the original file remains untouched.

---

## 2. FreeDesktop Trash Integration (`trash_entry_tracked`)

Destructive file operations default to the FreeDesktop Trash specification rather than unrecoverable permanent deletion:

1. **Standard Directory Structure**:
   - Files stored in `~/.local/share/Trash/files/`
   - Metadata recorded in `~/.local/share/Trash/info/<name>.trashinfo`
2. **Standard `.trashinfo` Schema**:
   ```ini
   [Trash Info]
   Path=/absolute/path/to/original/file
   DeletionDate=2026-11-08T07:22:00
   ```
3. **Collision Avoidance**: If a file of the same name was previously trashed, `auto_rename_target` appends a unique numeric counter (`file.1.txt`) to avoid metadata collision.
4. **Restoration Protocol (`restore_from_trash`)**: Reads `.trashinfo`, identifies the original destination, moves the file back, and cleans up the metadata record.

---

## 3. Reversible Operation Ledger (`FsOperationLedger`)

TermExplorer maintains an in-memory transactional ledger (`src/filesystem/ledger.rs`) recording up to 100 filesystem operations for instant reversal via the `u` key in Explorer mode.

### Recorded Operations
- **Move**: Records original source path, target path, and cross-device flags. Reverse operation moves the target back to the source (validating that the target exists and the source is not occupied).
- **Rename**: Records previous name and new name. Reverse operation safely renames the entry back.
- **Copy**: Records copied destination path. Reverse operation safely deletes the created copy.
- **Trash**: Records original path and `.trashinfo` identifier. Reverse operation restores the file to its exact original path via `restore_from_trash`.

---

## 4. Safeguards for Permanent Deletion

Permanent deletion bypasses the FreeDesktop Trash and cannot be undone. To prevent accidental data loss:
- **Dedicated Keybinding**: Permanent deletion requires explicit `Shift+D` or `Shift+Delete`.
- **Confirmation Modal (`ModalKind::DeletePermanent`)**:
  - Displays explicit warning header in high-visibility warning styling.
  - Displays the exact target absolute path.
  - Displays item count (recursively inspected for directories).
  - Explicitly states: *"Permanent, bypasses trash — CANNOT BE UNDONE"*.
  - Requires typing `"DELETE"` or `"y"` to confirm.
