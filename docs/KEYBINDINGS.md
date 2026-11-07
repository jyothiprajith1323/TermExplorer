# TermExplorer Keybindings Reference

**TermExplorer** is completely keyboard-driven. Below is the complete reference of shortcuts organized by panel, subsystem, and mode.

---

## 1. Global Shortcuts (Available Across Panels)

| Key | Action |
|---|---|
| `Tab` | Cycle focus forward across open panels |
| `Shift+Tab` | Cycle focus backward across open panels |
| `Ctrl+Shift+P` or `P` | Open fuzzy-search Command Palette |
| `F1` | Open Onboarding Wizard / Help guide |
| `F7` | Toggle Dual-Pane side-by-side file explorer view |
| `F10` | Open Interactive Settings Studio |
| `` ` `` (Backtick) | Toggle embedded Linux terminal PTY panel |
| `m` | Toggle live system resource monitor panel |
| `g` | Toggle Git repository status and commit history overlay |
| `Ctrl+B` | Toggle left navigation sidebar |
| `Ctrl+Q` | Immediately exit TermExplorer and restore terminal |
| `Esc` | Close active modal/popup, exit editor, or return to browse mode |

---

## 2. File Explorer Navigation & Operations

| Key | Action |
|---|---|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `Enter` / `l` | Open selected directory (or open file in editor) |
| `h` / `Backspace` | Navigate to parent directory |
| `/` or `Ctrl+F` | Open in-directory live fuzzy search bar |
| `.` or `h` | Toggle display of hidden files (`.dotfiles`) |
| `s` | Cycle sort order (Name, Size, Modified Date, Extension) |
| `a` | Create a new file in current directory |
| `A` | Create a new directory in current directory |
| `r` | Rename selected item |
| `d` | Move selected item to FreeDesktop Trash (`gio trash` / `trash-put`) |
| `D` | Permanently delete selected item (prompts for confirmation) |
| `c` or `y` | Copy selected item (or copy to opposing pane in Dual-Pane mode) |
| `m` or `x` | Move selected item (or move to opposing pane in Dual-Pane mode) |
| `p` or `v` | Paste item from clipboard into current directory |
| `b` | Quick jump to bookmarked directory |
| `p` | Pin current directory to Bookmarks |
| `F5` | Force reload / refresh directory listing |

---

## 3. Dual-Pane Mode (`F7`)

| Key | Action |
|---|---|
| `Tab` | Switch active focus between Left and Right panes |
| `c` | Copy selected file/folder directly to the opposing pane |
| `m` | Move selected file/folder directly to the opposing pane |
| `F7` | Exit Dual-Pane mode (returns to single explorer) |

---

## 4. Universal Preview Panel

| Key | Action |
|---|---|
| `[` | Switch to previous preview tab (`[Preview]` &larr; `[Info]` &larr; `[Hex]` &larr; `[Actions]`) |
| `]` | Switch to next preview tab (`[Preview]` &rarr; `[Info]` &rarr; `[Hex]` &rarr; `[Actions]`) |
| `j` / `↓` | Scroll preview content line by line |
| `k` / `↑` | Scroll preview content upwards |
| `PageUp` / `PageDown` | Scroll preview content by 20 lines |

---

## 5. Interactive Settings Studio (`F10`)

| Key | Action |
|---|---|
| `Tab` | Switch focus between Category List and Setting Controls |
| `j` / `k` / `↑` / `↓` | Select category or navigate settings within a section |
| `Space` / `Enter` | Toggle boolean values or activate profile / action |
| `←` / `→` / `h` / `l` | Adjust numeric slider or cycle choice options |
| `Esc` | Save validated configuration and return to file explorer |

---

## 6. Integrated Code Editor (`F2` / `Enter`)

| Key | Action |
|---|---|
| `Ctrl+S` | Save file atomically to disk |
| `Ctrl+Z` | Multi-level Undo |
| `Ctrl+Y` | Multi-level Redo |
| `Ctrl+C` | Copy current line or selection to editor clipboard |
| `Ctrl+X` | Cut current line to editor clipboard |
| `Ctrl+V` | Paste text at cursor position |
| `Tab` | Indent 4 spaces (or accept highlighted autocomplete candidate) |
| `Enter` | Insert newline with automatic indentation |
| `Esc` | Exit editor (prompts to save if buffer is dirty) |
| `Backspace` | Deletes character (or deletes matching pair if cursor is between) |

### Auto-Pairing Characters:
- `(` inserts `()`
- `[` inserts `[]`
- `{` inserts `{}`
- `"` inserts `""`
- `'` inserts `''`
- `` ` `` inserts ```` ````
- Typing matching closing character skips over it.

---

## 7. Embedded Terminal (`Ctrl+`` or ``` ` ```)

When the terminal panel is focused, input is forwarded directly to the slave pseudo-terminal:
- `Tab`: Real shell autocompletion (commands, paths, flags, options).
- `Ctrl+C`: Send interrupt signal (SIGINT).
- `Ctrl+D`: Send EOF to shell.
- `Ctrl+Z`: Send suspend signal (SIGTSTP).
- `Ctrl+L`: Clear terminal display.
- `Tab` / `Ctrl+``: Defocus terminal back to explorer.
