# TermExplorer Keyboard Shortcuts & Keybinding Architecture Reference

**TermExplorer** is completely keyboard-driven with ergonomics designed for high-velocity software engineering, systems administration, and daily terminal file management.

In Sprint 6B, keybinding management was centralized in `src/settings/keybindings.rs`, providing strict scope separation, automated conflict detection, and fuzzy-searchable keybinding discovery.

---

## 1. Keybinding Architecture & Scope Separation

To prevent shortcut collisions across complex modalities (e.g. typing text inside the rope editor versus navigating directories in the explorer), all shortcuts are assigned a specific `BindingScope`:

| Scope | Active Context | Conflict Rules |
|---|---|---|
| `Global` | Active throughout the workstation regardless of focused panel | Cannot conflict with any other Global binding |
| `Explorer` | Active when the file browser panel has keyboard focus | Can reuse single letters (e.g., `j`, `k`, `r`, `d`) safely |
| `Editor` | Active when the integrated code editor is focused | Prioritizes text input; reserved prefixes like `Ctrl+` only |
| `Terminal` | Active when the bottom embedded PTY drawer is focused | Passes through most keys to the shell process; intercept keys like `Ctrl+T` or `` ` `` |
| `Preview` | Active when the Contextual Preview/Inspector pane is focused | Allows scrolling (`j`, `k`) and tab cycling (`[`, `]`, `1`-`4`) |
| `Git` | Active when the Git Workbench modal is open | Dedicated staging (`s`), unstaging (`u`), and commit (`c`) keys |
| `Search` | Active when the Unified Search / ripgrep pane is open | Input filtering and result traversal |
| `Workflow` | Active in Task Runner and Plugin Center | Execution and management bindings |

The built-in conflict detector (`KeybindingManager::detect_conflicts()`) runs at startup and in test verification, ensuring **0 conflicting assignments** across all registered default shortcuts.

---

## 2. Global Shortcuts (Available Workstation-Wide)

| Key | Action | Description | Scope |
|---|---|---|---|
| `Tab` | **Cycle Panel Focus** | Moves focus between Explorer, Preview, and Terminal drawer | `Global` |
| `Shift+Tab` | **Cycle Focus Backward** | Moves focus in reverse order | `Global` |
| `Ctrl+P` / `Ctrl+Shift+P` | **Command Palette** | Quick action launcher, file switcher & workflow runner | `Global` |
| `Ctrl+,` / `F10` | **Settings Studio** | Interactive live preference configuration across 14 categories | `Global` |
| `Ctrl+Shift+X` | **Plugin Center** | Browse marketplace, manage active plugins, configure Wasm permissions | `Global` |
| `F1` / `?` | **Shortcuts Matrix** | Workstation keyboard cheat-sheet overlay | `Global` |
| `Ctrl+G` / `g` | **Git Workbench** | Staged/unstaged file changes, branch indicator, and commit history | `Global` |
| `F6` / `Ctrl+M` | **System Telemetry** | Real-time CPU, Memory, Swap, Disk, and Process monitor | `Global` |
| `F7` | **Dual-Pane View** | Toggle side-by-side secondary file manager (50/50 split) | `Global` |
| `F3` / `F11` | **Preview Zoom** | Maximize preview pane or restore 25%/75% default layout | `Global` |
| `` ` `` / `Ctrl-` ` `` | **Terminal Drawer** | Toggle embedded PTY terminal drawer (15% height) | `Global` |
| `Ctrl+Right` / `Alt+]` | **Expand Preview Width** | Increases preview width by +5% (up to 85% width limit) | `Global` |
| `Ctrl+Left` / `Alt+[` | **Shrink Preview Width** | Decreases preview width by -5% (down to 50% width limit) | `Global` |
| `Ctrl+Up` / `Alt+}` | **Expand Terminal Height** | Increases bottom PTY drawer height (up to 40% height limit) | `Global` |
| `Ctrl+Down` / `Alt+{` | **Shrink Terminal Height** | Decreases bottom PTY drawer height (down to 10% height limit) | `Global` |
| `F5` | **Refresh View** | Force re-read of directory tree and update Git status | `Global` |
| `Ctrl+Q` | **Quit TermExplorer** | Cleanly terminate child processes and restore shell terminal | `Global` |
| `Esc` | **Universal Escape** | Closes any modal, dialog, or search and returns focus to Explorer | `Global` |

---

## 3. File Explorer Navigation & Operations

| Key | Action | Description | Scope |
|---|---|---|---|
| `↓` / `j` | **Move Down** | Select next file or folder in directory listing | `Explorer` |
| `↑` / `k` | **Move Up** | Select previous file or folder in directory listing | `Explorer` |
| `→` / `Enter` / `l` | **Open / Descend** | Descend into highlighted folder or open file in viewer | `Explorer` |
| `←` / `Backspace` / `h` | **Ascend Directory** | Navigate up to parent folder | `Explorer` |
| `Space` | **Focus Preview** | Move keyboard focus immediately into Contextual Preview | `Explorer` |
| `Home` / `gg` | **Jump to Top** | Instantly select first entry in directory | `Explorer` |
| `End` / `G` | **Jump to Bottom** | Instantly select last entry in directory | `Explorer` |
| `PageUp` / `PageDown` | **Scroll Page** | Fast-scroll directory listing by one screen | `Explorer` |
| `/` or `Ctrl+F` | **Inline Fuzzy Filter** | Real-time search filter matching filenames instantly | `Explorer` |
| `.` or `zh` | **Toggle Hidden Files** | Show or hide dotfiles (`.gitignore`, `.env`, etc.) | `Explorer` |
| `s` | **Cycle Sort Order** | Sort by Name, Size, Date Modified, or Extension | `Explorer` |
| `a` / `F4` | **New File** | Prompt to create a new file in current directory | `Explorer` |
| `A` | **New Directory** | Prompt to create a new subdirectory in current directory | `Explorer` |
| `r` | **Rename Entry** | In-place rename modal for highlighted file or folder | `Explorer` |
| `d` / `Delete` | **Delete Entry** | Move selected item to FreeDesktop Trash with confirmation | `Explorer` |
| `c` / `y` | **Copy Entry** | Copy highlighted file or folder to application clipboard | `Explorer` |
| `x` | **Cut Entry** | Cut highlighted file or folder for moving | `Explorer` |
| `v` / `p` | **Paste Entry** | Paste clipboard item into current directory | `Explorer` |
| `F2` | **Open in Editor** | Open highlighted file in the integrated code editor | `Explorer` |

---

## 4. Contextual Preview & Inspector

| Key | Action | Description | Scope |
|---|---|---|---|
| `1` | **Preview Tab** | Jump directly to syntax-highlighted code / markdown / media viewer | `Preview` |
| `2` | **Info Tab** | Jump directly to filesystem metadata, permissions, hashes, and size | `Preview` |
| `3` | **Hex / Metadata Tab** | Jump directly to hex binary viewer and low-level byte inspector | `Preview` |
| `4` | **Actions Tab** | Jump directly to contextual actions (Open with, Extract, Run) | `Preview` |
| `[` / `]` | **Cycle Active Tab** | Navigate previous / next inspector tab | `Preview` |
| `↓` / `j` | **Scroll Down** | Scroll preview content line by line | `Preview` |
| `↑` / `k` | **Scroll Up** | Scroll preview content upwards | `Preview` |
| `PageDown` / `PageUp` | **Page Scroll** | Scroll preview by 20 lines | `Preview` |
| `Home` / `End` | **Start / End of File** | Instantly jump to line 1 or end of document | `Preview` |
| `1` – `9` (in Actions) | **Execute Action** | Trigger numbered action directly | `Preview` |
| `Esc` / `Backspace` / `h` | **Return to Explorer** | Switch keyboard focus back to File Explorer | `Preview` |

---

## 5. Integrated Code Editor (`F2` / `Enter`)

| Key | Action | Description | Scope |
|---|---|---|---|
| `Ctrl+S` | **Save Buffer** | Write changes atomically to disk with directory sync | `Editor` |
| `Ctrl+Z` | **Undo** | Revert latest buffer edit | `Editor` |
| `Ctrl+Y` | **Redo** | Restore previously reverted edit | `Editor` |
| `Tab` | **Accept Completion / Indent** | Accept autocomplete suggestion or indent by `tab_width` spaces | `Editor` |
| `Enter` | **Newline** | Insert newline with automatic bracket indentation | `Editor` |
| `Backspace` | **Smart Backspace** | Delete character (or matched pair if between delimiters) | `Editor` |
| `Esc` | **Close Editor** | Exit editor (prompts with safety warning if buffer is dirty) | `Editor` |

---

## 6. Integrated Linux Terminal Drawer (`` ` `` / `Ctrl-` ` ``)

| Key | Action | Description | Scope |
|---|---|---|---|
| `Ctrl+C` | **SIGINT** | Standard terminal interrupt signal sent directly to PTY shell | `Terminal` |
| `Ctrl+D` | **EOF** | End-of-file / exit shell session | `Terminal` |
| `Tab` / `Shift+Tab` | **Cycle Focus** | Switch keyboard focus to Explorer or Preview | `Terminal` |
| `` ` `` / `Ctrl-` ` `` | **Hide Drawer** | Hide terminal drawer without killing background shell process | `Terminal` |

---

## 7. Interactive Settings Studio (`Ctrl+,` / `F10`)

The Settings Studio provides full live preference inspection and modification across 14 sections:

| Key | Action | Description |
|---|---|---|
| `Tab` | **Switch Column** | Toggle focus between Section Category List (left) and Item Controls (right) |
| `↑` / `↓` / `k` / `j` | **Select Item** | Navigate sections or configuration settings |
| `Space` / `Enter` | **Toggle Value** | Toggle booleans or activate selected theme / layout profile |
| `←` / `→` | **Adjust Numeric** | Change percentages, widths, or cycle dropdown options |
| `r` | **Reset Highlighted Setting** | Revert the currently selected setting to its default factory value |
| `R` | **Reset All Settings** | Revert all 14 categories back to standard TermExplorer factory defaults |
| `e` | **Export Configuration** | Export active configuration to `~/.config/termexplorer/config_export.toml` |
| `i` | **Import Configuration** | Import configuration from `~/.config/termexplorer/config_export.toml` |
| `q` / `Esc` / `F10` | **Save & Exit** | Atomically commit validated settings to `config.toml` and return |

---

## 8. Git Workbench (`g` / `Ctrl+G`)

| Key | Action | Description | Scope |
|---|---|---|---|
| `↑` / `↓` / `k` / `j` | **Select Change** | Navigate list of modified, staged, or untracked files | `Git` |
| `s` | **Stage File** | Stage selected file (`git add <file>`) | `Git` |
| `u` | **Unstage File** | Unstage selected file (`git restore --staged <file>`) | `Git` |
| `c` | **Commit Changes** | Prompt for commit message and commit staged changes | `Git` |
| `q` / `Esc` | **Close Workbench** | Return to file explorer view | `Git` |

---

## 9. Plugin Marketplace & Extension Center (`Ctrl+Shift+X`)

| Key | Action | Description | Scope |
|---|---|---|---|
| `1` / `2` / `3` / `Tab` | **Switch Tabs** | 1: Marketplace Catalog, 2: Installed Plugins, 3: Permissions | `Workflow` |
| `↑` / `↓` / `k` / `j` | **Select Extension** | Navigate plugin entries | `Workflow` |
| `i` / `Enter` | **Install / Toggle** | Install from marketplace (tab 1) or toggle enable/disable (tab 2) | `Workflow` |
| `d` / `u` | **Uninstall** | Remove installed extension and purge on-disk assets | `Workflow` |
| `r` | **Reload Extensions** | Rescan plugin directory from disk | `Workflow` |
| `c` | **Cycle Category** | Filter marketplace plugins by category | `Workflow` |
| `Esc` | **Exit Center** | Return to file explorer view | `Workflow` |
