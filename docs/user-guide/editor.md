# Integrated Code Editor

TermExplorer includes a full-featured, rope-backed terminal text editor designed for fast code edits, configuration tuning, and scratch work.

---

## 1. Opening and Closing the Editor

- **Open File**: Highlight any file in the Explorer and press `F2` or `Enter`.
- **Close Editor**: Press `Esc`.
  - If the buffer has no unsaved changes, the editor closes immediately, returning focus to the File Explorer.
  - If the buffer contains unsaved changes, TermExplorer displays an explicit confirmation modal: `Discard unsaved changes? [y/N]`. Press `y` to discard, or `n` to return to the editor.

---

## 2. Buffer Engine & Rope Architecture

The editor uses `ropey::Rope`, a B-tree of string fragments:
- **Instantaneous Large File Edits**: Insertions, deletions, and line splits execute in logarithmic time $O(\log N)$, making editing large source files (>100,000 lines) fast and memory-efficient.
- **Multibyte Unicode Precision**: Cursor movement and text slicing operate on Unicode scalar values and extended grapheme clusters rather than raw bytes, avoiding character splitting or panics on emojis, accents, and CJK characters.
- **Line Endings & Encoding**: Automatically detects Unix (`LF`) and Windows (`CRLF`) line endings, and supports UTF-8, UTF-8 BOM, and Latin-1 encoding preservation.

---

## 3. Editing Capabilities

| Shortcut | Action | Description |
|---|---|---|
| `Ctrl+S` | **Atomic Save** | Writes buffer atomically via `.tmp` file and syncs directory metadata |
| `Ctrl+Z` | **Undo** | Reverts latest text insertion or deletion |
| `Ctrl+Y` | **Redo** | Restores previously reverted edit |
| `Ctrl+C` | **Copy Line** | Copies active line or visual selection to clipboard |
| `Ctrl+X` | **Cut Line** | Cuts active line or visual selection to clipboard |
| `Ctrl+V` | **Paste** | Pastes text from clipboard at cursor position |
| `Ctrl+/` | **Toggle Comment** | Toggles line comment based on detected programming language |
| `F3` / `Shift+F3` | **Find Next / Previous** | Traverses search matches in active buffer |
| `Tab` | **Indent / Accept** | Inserts `tab_width` spaces (default 4) or accepts autocomplete item |
| `Enter` | **Smart Newline** | Inserts newline preserving indentation level of previous line |

---

## 4. Four-Layer Autocomplete Engine

The autocomplete popup provides intelligent suggestions composed of four tiers:
1. **Layer 1 (Buffer Words)**: Words and identifiers existing in the active text buffer.
2. **Layer 2 (Workspace Symbols)**: Project-wide symbol indexer (<26 ms latency).
3. **Layer 3 (Language Snippets)**: Common control flow templates (`fn`, `for`, `match`, `class`, `def`).
4. **Layer 4 (LSP Protocol)**: Language Server Protocol suggestions if an active LSP server (such as `rust-analyzer`) is detected in `$PATH`.

Press `Ctrl+Space` to trigger completions manually. Use `↑` / `↓` to select, and `Tab` or `Enter` to insert.

---

## 5. Atomic Saving & Recovery Safety

When `Ctrl+S` is triggered, TermExplorer safeguards against data corruption or power loss:
1. The rope buffer is serialized to a temporary sibling file (`.filename.tmp.<pid>`).
2. `fsync()` / `sync_all()` flushes contents to disk.
3. The temporary file is renamed atomically over the target file using POSIX `rename(2)`.
4. Parent directory metadata is synced to ensure directory entry consistency.
