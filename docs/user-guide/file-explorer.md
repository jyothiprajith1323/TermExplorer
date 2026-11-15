# File Explorer Operations & Safety

The File Explorer in TermExplorer handles everyday filesystem operations with strict safety guards, confirmation dialogs, and FreeDesktop.org Trash compliance.

---

## 1. File & Directory Creation

- **Create New File**: Press `a` (or `F4`). A text prompt appears at the bottom. Type the filename (e.g. `config.rs`, `README.md`) and press `Enter`.
- **Create New Directory**: Press `A` (or `F7`). Enter the new folder name and press `Enter`.

Safety checks ensure names do not contain forbidden path separators or invalid null bytes.

---

## 2. In-Place Renaming

To rename any highlighted file or folder:
1. Press `r`.
2. An inline modal opens pre-filled with the current name.
3. Edit the filename and press `Enter`.
4. If a target with that name already exists, TermExplorer prompts with collision choices (Overwrite, Auto-Rename, or Cancel).

---

## 3. Deletion & FreeDesktop Trash Safety

TermExplorer avoids permanent data loss by integrating with standard desktop trash specifications:

- **Soft Delete (Trash)**: Press `d` or `Delete`.
  - A confirmation dialog appears: `Move <filename> to Trash? [y/N]`.
  - Upon confirmation (`y`), the file is moved to the system trash via `gio trash` or `trash-cli`.
  - If external trash utilities are unavailable, TermExplorer archives items into `.termexplorer/trash/` with a manifest tracking original paths and deletion timestamps.
- **Permanent Hard Delete**: Press `D` (Shift+D).
  - Explicit warning dialog: `Permanently delete <filename>? This cannot be undone! [y/N]`.
  - Bounded recursive deletion ensures large directory hierarchies do not lock the user interface.

---

## 4. Clipboard Operations (Copy, Cut, Paste)

- **Copy Entry**: Press `c` or `y`. The full path is copied to the application clipboard buffer.
- **Cut Entry**: Press `x`. The file is marked for moving.
- **Paste Entry**: Press `v` or `p`.
  - If a copy was buffered, the file is duplicated with POSIX file permissions preserved.
  - If a cut was buffered, the file is moved atomically across the filesystem.

---

## 5. Directory Refreshing & Large Directory Performance

- **Refresh Directory (`F5`)**: Force re-reads directory entries from disk and triggers Git status updates.
- **Chunked Viewport Loading**: For directories containing 50,000+ files (e.g. `node_modules` or cache dirs), TermExplorer loads the initial visible viewport instantly (<10 ms) and streams subsequent chunks asynchronously in the background.
