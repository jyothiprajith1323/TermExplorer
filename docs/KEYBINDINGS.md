# TermExplorer Keybindings Reference

TermExplorer is completely keyboard-driven. Below is the complete reference of shortcuts organized by panel and mode.

---

## 1. Global Shortcuts (Available Everywhere)

| Key | Description |
|---|---|
| `Ctrl+Q` | Immediately exits TermExplorer and restores terminal |
| `Ctrl+` ` | Toggles embedded Linux terminal panel |
| `Ctrl+B` | Toggles left sidebar visibility |
| `F1` | Opens / closes the interactive keyboard help overlay |
| `F6` | Toggles system resource monitor panel |
| `Tab` | Cycles focus forward across open panels |
| `Shift+Tab` | Cycles focus backward across open panels |
| `Esc` | Cancels active search, closes popup, or returns to browse mode |

---

## 2. File Explorer Navigation & Operations

| Key | Description |
|---|---|
| `↑` / `↓` | Move selection up / down in directory list |
| `Enter` | Open directory (or select file for preview) |
| `Backspace` | Navigate to parent directory |
| `F2` | Open selected file in integrated text editor |
| `F4` | Prompt to create a new file |
| `F5` | Force reload / refresh current directory |
| `F7` | Prompt to create a new directory |
| `F8` | Prompt to delete selected file or directory (confirm with `y`) |
| `r` | Prompt to rename selected item |
| `s` | Cycle sort order (Name ▲, Name ▼, Size ▲, Size ▼, Date ▲, Date ▼, Type) |
| `c` or `y` | Copy selected item to clipboard |
| `x` | Cut selected item to clipboard (move on paste) |
| `p` or `v` | Paste item from clipboard into current directory |
| `h` | Toggle display of hidden files (`.dotfiles`) |
| `Ctrl+F` or `/` | Open in-directory fuzzy search bar |
| `Alt+P` | Pin current working directory to sidebar |

---

## 3. Sidebar Panel (when focused)

| Key | Description |
|---|---|
| `↑` / `↓` | Navigate pinned items and system shortcuts |
| `Enter` | Navigate File Explorer into selected pinned folder or shortcut |
| `d` or `Delete` | Unpin highlighted pinned folder |

---

## 4. File Preview Panel (when focused)

| Key | Description |
|---|---|
| `↑` / `↓` | Scroll preview content line by line |
| `PageUp` / `PageDown` | Scroll preview content by 20 lines |

---

## 5. Integrated Text Editor (F2)

| Key | Description |
|---|---|
| `Ctrl+S` | Save file atomically to disk |
| `Ctrl+Z` | Undo last edit |
| `Ctrl+Y` | Redo last undone edit |
| `Ctrl+C` | Copy current line to buffer clipboard |
| `Ctrl+X` | Cut current line to buffer clipboard |
| `Ctrl+V` | Paste text at cursor |
| `Ctrl+F` | Find within file |
| `Tab` | Insert 4 spaces (or accept completion if popup is active) |
| `Enter` | Insert newline with automatic indentation |
| `Esc` | Close editor (warns if unsaved changes exist) |
| `↑` / `↓` | Move cursor vertically (or select completion suggestion) |
| `←` / `→` | Move cursor horizontally |
| `Home` / `End` | Jump to start / end of current line |
| `PageUp` / `PageDown` | Scroll by 20 lines |
| `Backspace` | Backspace (auto-deletes matching pair if cursor is between) |
| `Delete` | Delete character under cursor |

### Auto-Pairing Characters:
- `(` inserts `()`
- `[` inserts `[]`
- `{` inserts `{}`
- `"` inserts `""`
- `'` inserts `''`
- `` ` `` inserts ```` ````
- Typing matching closing character (` ) `, ` ] `, ` } `, etc.) skips over it when directly under cursor.

---

## 6. Embedded Terminal (Ctrl+`)

When focused on the terminal panel, all standard keystrokes and shell shortcuts are forwarded directly to the underlying PTY:
- `Tab`: Real shell autocompletion (commands, paths, options, aliases)
- `Ctrl+C`: Send interrupt signal (SIGINT) to running process
- `Ctrl+D`: Send EOF to shell
- `Ctrl+L`: Clear shell screen
- `Up` / `Down`: Shell command history
- `Ctrl+` `: Toggle terminal panel back to file explorer
- `Ctrl+Q`: Force exit TermExplorer
