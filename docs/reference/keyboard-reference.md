# Keyboard Shortcuts Master Reference

This reference is derived from the centralized shortcut registry in `src/settings/keybindings.rs`.

---

## 1. Scope Architecture & Conflict Rules

All shortcuts are strictly partitioned across 8 operational scopes:
- **`Global`**: Accessible workstation-wide.
- **`Explorer`**: Active when the File Explorer has keyboard focus.
- **`Editor`**: Active when editing in the rope buffer.
- **`Terminal`**: Active when inside the bottom PTY drawer.
- **`Preview`**: Active when scrolling or navigating in the Inspector pane.
- **`Git`**: Active inside the Git Workbench modal.
- **`Search`**: Active in the Workspace Search modals.
- **`Workflow`**: Active in the Task Viewer and Plugin Center.

The automated conflict detection suite verifies that **0 conflicting shortcuts** exist across active scopes.

---

## 2. Complete Keybinding Registry

| Scope | ID | Name | Primary Key | Description |
|---|---|---|---|---|
| `Global` | `help` | Show Help Overlay | `F1` / `?` | Display keyboard shortcuts cheat-sheet |
| `Global` | `edit` | Open in Editor | `F2` | Open selected file in integrated code editor |
| `Global` | `preview_zoom` | Toggle Preview Zoom | `F3` | Maximize or restore right preview panel |
| `Global` | `refresh` | Refresh View | `F5` | Reload active directory and system stats |
| `Global` | `monitor` | Toggle System Monitor | `F6` | Show/hide bottom system telemetry monitor |
| `Global` | `dual_pane` | Toggle Dual Pane Explorer | `F7` | Toggle side-by-side dual explorer panes |
| `Global` | `settings` | Settings Studio | `F10` / `Ctrl+,` | Open interactive settings application |
| `Global` | `cmd_palette` | Command Palette | `Ctrl+Shift+P` | Fuzzy-search commands, tasks, git, and files |
| `Global` | `term_drawer` | Toggle Terminal Drawer | `` ` `` / `Ctrl-` ` `` | Toggle embedded PTY terminal drawer (15% height) |
| `Global` | `cycle_focus` | Cycle Focus | `Tab` / `Shift-Tab` | Cycle focus between Explorer, Preview, and Terminal |
| `Global` | `quit` | Quit Application | `Ctrl+Q` / `q` | Cleanly restore terminal and exit |
| `Explorer` | `nav_down` | Move Down | `Down` / `j` | Highlight next file or folder |
| `Explorer` | `nav_up` | Move Up | `Up` / `k` | Highlight previous file or folder |
| `Explorer` | `open` | Enter Directory | `Right` / `Enter` / `l` | Open selected directory or inspect file |
| `Explorer` | `parent` | Parent Directory | `Left` / `Backspace` / `h` | Navigate to parent folder |
| `Explorer` | `top_bottom` | Jump Top / Bottom | `gg` / `G` | Jump to first or last item in file list |
| `Explorer` | `new_file` | New File | `a` | Create new empty file in current directory |
| `Explorer` | `new_dir` | New Directory | `A` | Create new subdirectory in current directory |
| `Explorer` | `rename` | Rename Item | `r` | Rename selected file or folder |
| `Explorer` | `delete` | Delete Item | `d` / `Delete` | Send to FreeDesktop Trash or confirm delete |
| `Explorer` | `search_filter` | Fuzzy Filter | `/` | Filter files in current directory |
| `Explorer` | `hidden` | Toggle Hidden Files | `zh` / `.` | Show or hide dotfiles and hidden folders |
| `Editor` | `save` | Atomic Save | `Ctrl+S` | Atomically persist rope buffer to disk |
| `Editor` | `undo` | Undo Edit | `Ctrl+Z` | Revert last editing operation |
| `Editor` | `redo` | Redo Edit | `Ctrl+Y` | Reapply reverted edit |
| `Editor` | `complete` | Accept Completion | `Tab` / `Enter` | Insert selected word completion popup item |
| `Editor` | `close_editor` | Close Editor | `Esc` | Return to file explorer view |
| `Terminal` | `term_interrupt` | Send SIGINT | `Ctrl+C` | Interrupt foreground process in PTY |
| `Terminal` | `term_eof` | Send EOF | `Ctrl+D` | Send end-of-file signal to active shell |
| `Git` | `git_panel` | Git Workbench | `g` | Open Git status, diff viewer, and branch manager |
| `Search` | `project_search` | Unified Search | `Ctrl+Shift+F` | Global ripgrep-style content search across project |
| `Workflow` | `plugin_manager` | Plugin Center | `Ctrl+Shift+X` | Browse, inspect, and manage WebAssembly plugins |
