# TermExplorer Filesystem Watching & Debouncing

## Overview

Filesystem monitoring keeps the explorer file list and open editor buffers in sync with external background activities (such as `git pull`, `cargo build`, or file creation in the integrated terminal drawer). TermExplorer implements an efficient, debounced watching pipeline.

---

## 1. Non-Recursive Inotify Monitoring (GAP-06)

TermExplorer watches only the directory currently displayed in the active file explorer pane:
- Uses `notify::RecursiveMode::NonRecursive`.
- Prevents inotify file descriptor exhaustion when navigating into large directory trees (such as Linux kernel repositories, deep monorepos, or `target/` directories).
- Whenever the user navigates into a new folder (`navigate_into`), the previous inotify watcher is dropped and a new watcher is established immediately.

---

## 2. Event Debouncing Engine (`watch_directory_debounced`)

During bulk filesystem mutations (e.g., executing `cargo build` which creates hundreds of `.rlib` and `.rmeta` artifacts in milliseconds, or `npm install`), raw filesystem watchers emit hundreds of individual notifications per second. Processing every event individually produces severe UI stutter and redundant directory rescanning.

### Debounce Loop Protocol (`src/filesystem/watcher.rs`)
1. An internal background task drains raw `notify::Event` items from the OS.
2. Upon receipt of the initial event, a **150ms quiet window** timer is armed.
3. If subsequent events arrive before the timer elapses, the timer resets and events are coalesced.
4. Once the filesystem has been quiet for 150ms, a single consolidated notification (`()`) is emitted to the application event channel.
5. In tests, a burst of 500 file operations triggers exactly 1 directory reload instead of 500.

---

## 3. External Modification Detection & Conflicts

When an open file is edited externally while loaded in TermExplorer's editor:
1. `Buffer::check_external_modification` compares the current disk `mtime` with the buffer's `loaded_mtime`.
2. Periodic checks on tick and upon filesystem events alert the user with a non-blocking warning in the status bar:
   ```text
   Warning: File modified externally! Press Ctrl+R to reload or Ctrl+S to overwrite.
   ```
3. Attempting to save via standard `Ctrl+S` detects the conflict and refuses to silently overwrite external work.
4. **Resolution Paths**:
   - `Ctrl+R`: Reloads the latest contents directly from disk.
   - `Ctrl+Shift+S` or `Alt+S`: Forces overwrite with `save_force()`.
