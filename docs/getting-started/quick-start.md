# Quick Start Tutorial

This 5-minute hands-on tutorial walks through the primary development workflows in **TermExplorer**.

---

## 1. Launching TermExplorer

Launch TermExplorer in any repository or directory:

```bash
# Open current directory
termexplorer

# Open specific project folder
termexplorer ~/Projects/my-rust-app
```

The screen displays the default workstation layout:
- **Left Panel (25% width)**: File Explorer
- **Right Panel (75% width)**: Contextual Preview & Inspector
- **Bottom**: Status bar indicating current path, Git branch, and key hints

---

## 2. Navigating Files & Folders

TermExplorer provides intuitive cursor movement and Vim-style keys:

- Move up/down: `↑` / `↓` or `k` / `j`.
- Open a folder: `→`, `Enter`, or `l`.
- Return to parent folder: `←`, `Backspace`, or `h`.
- Quick fuzzy filter in directory: press `/` and type part of the file name. Press `Esc` to clear.
- Toggle hidden dotfiles: press `.` or `zh`.

As you highlight different files, the right panel updates instantly with an appropriate contextual preview.

---

## 3. Inspecting Files & Data

The **Contextual Preview & Inspector** features 4 tabs:
- Press `1` for **Preview**: Syntax-highlighted code, formatted Markdown, pixel half-block images, parsed CSV/JSON tables, or database schema.
- Press `2` for **Information**: File size, permissions, owner, MIME type, SHA-256 hash, and timestamps.
- Press `3` for **Hex View**: Byte-level hexadecimal dump with ASCII translation.
- Press `4` for **Actions**: Contextual commands (Open in system editor, Extract archive, Execute script).

To scroll the preview pane:
- Press `Space` to switch focus into the Preview panel.
- Scroll with `j` / `k` or `PageDown` / `PageUp`.
- Press `Esc` or `h` to return focus to the File Explorer.

---

## 4. Editing Code with the Integrated Editor

To edit any text or code file:
1. Highlight the file in Explorer and press `F2` (or `Enter`).
2. The right panel switches to the integrated code editor with syntax highlighting and line numbers.
3. Edit text normally.
4. Auto-completion triggers automatically as you type, or press `Ctrl+Space`. Press `Tab` or `Enter` to accept completions.
5. Save changes atomically with `Ctrl+S`.
6. Undo edits with `Ctrl+Z`; redo with `Ctrl+Y`.
7. Press `Esc` to exit the editor. If there are unsaved changes, TermExplorer prompts before discarding.

---

## 5. Using the Embedded Terminal Drawer

TermExplorer includes a full pseudo-terminal (PTY) shell:
- Press `` ` `` (backtick) or `Ctrl+` ` `` to toggle the terminal drawer.
- The drawer opens at the bottom (15% height default).
- Your interactive login shell (`zsh`, `bash`, `fish`) starts automatically.
- Run commands (e.g. `cargo build`, `git status`, `npm test`).
- Press `` ` `` again to hide the drawer—background jobs continue executing without interruption.
- Switch focus between panels with `Tab` or `Shift+Tab`.

---

## 6. Using the Command Palette & Git

- **Command Palette**: Press `Ctrl+Shift+P` (or `Ctrl+P`) to open the linear fuzzy command launcher. Search for commands, workspace actions, or file bookmarks.
- **Git Workbench**: Press `g` (or `Ctrl+G`) to open the Git status dashboard. Inspect unstaged and staged files, view unified diffs, stage files with `s`, and commit with `c`.

---

## 7. Customizing Settings & Exiting

- Open **Settings Studio** with `Ctrl+,` or `F10`. Explore themes (Catppuccin Mocha, Nord, Dracula), layout proportions, and editor behaviors.
- Exit TermExplorer cleanly with `Ctrl+Q` or `q`.
