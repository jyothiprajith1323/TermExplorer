# Unified Search Engine

TermExplorer includes a high-performance Unified Search Engine combining fuzzy workspace filename matching and ripgrep-style regular expression content search.

---

## 1. Fuzzy Filename Search (`/` or `Ctrl+F`)

Press `/` or `Ctrl+F` to open the Workspace File Search modal:
- **Instantaneous Fuzzy Matching**: Uses Smith-Waterman / Sublime-style fuzzy ranking to match characters across filenames and directory paths.
- **Ignore Rules**: Automatically respects `.gitignore`, `.ignore`, and global gitignore rules using the `ignore` crate.
- **Results Navigation**:
  - `↑` / `↓` or `k` / `j`: Move up and down in matched results.
  - `Enter`: Open selected file and focus in the editor (or preview).
  - `Esc`: Close search modal and return to previous workspace view.

---

## 2. Project-Wide Content Grep (`Ctrl+Shift+F`)

Press `Ctrl+Shift+F` to open the Full-Text Content Search modal:
- **Fast Content Search**: Scans files using parallel worker threads and the `regex` engine.
- **Search Options**:
  - **Smart Case (Default)**: Searches case-insensitively if the query is all lowercase, and case-sensitively if it contains uppercase characters.
  - **Case Sensitive Toggle**: Force case sensitivity on or off.
  - **Symlink Following**: By default, symlinks are not followed to prevent infinite recursive loops. Toggleable in settings.
  - **Maximum Results**: Caps results (default 1,000; configurable up to 50,000) to keep memory usage bounded.

---

## 3. Real-Time Cancellation & Responsiveness

When typing rapidly or searching large projects (>500,000 lines of code):
- Each keystroke cancels any previous in-flight search task using atomic cancellation tokens (`SearchCancellation`).
- Threads yield immediately upon detecting cancellation, preventing CPU contention and UI stutter.
- Results stream asynchronously into the modal list as they are discovered.

---

## 4. Result Navigation & Preview

Selecting a match in the content search list displays:
- The matched file path and line number.
- Syntax-highlighted context lines around the match.
- Press `Enter` to open the file directly at the matched line number in the integrated code editor.
