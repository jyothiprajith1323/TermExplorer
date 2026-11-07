# TermExplorer Keyboard Shortcuts Reference

**TermExplorer** is completely keyboard-driven with ergonomics designed for high-velocity software engineering, systems administration, and daily terminal file management.

---

## 1. Global Shortcuts (Available Anywhere)

| Key | Action | Description |
|---|---|---|
| `Tab` | **Cycle Panel Focus** | Moves focus between File Explorer, Contextual Preview, and Terminal drawer |
| `Shift+Tab` | **Cycle Focus Backward** | Moves focus in reverse order |
| `Ctrl+P` / `Ctrl+Shift+P` | **Command Palette** | Linear/Raycast-inspired quick action launcher & workspace search |
| `Ctrl+,` / `F10` | **Settings Studio** | Interactive live preference configuration across 14 categories |
| `Ctrl+Shift+X` | **Plugin Center** | Browse marketplace, manage active plugins, configure Wasm permissions |
| `F1` / `?` | **Shortcuts Matrix** | Workstation keyboard cheat-sheet overlay |
| `Ctrl+G` / `g` | **Git Workbench** | Staged/unstaged file changes, branch indicator, and commit history |
| `F6` / `Ctrl+M` | **System Telemetry** | Real-time CPU, Memory, Swap, Disk, and Process monitor |
| `F7` | **Dual-Pane View** | Toggle side-by-side secondary file manager (50/50 split) |
| `F11` / `z` | **Fullscreen Preview** | Expand Contextual Preview to 100% workstation width |
| `Ctrl+Right` / `Alt+]` | **Expand Preview Width** | Increases preview width by +5% (up to 85% width) |
| `Ctrl+Left` / `Alt+[` | **Shrink Preview Width** | Decreases preview width by -5% (down to 15% width) |
| `Ctrl+Up` / `Alt+}` | **Expand Terminal Height** | Increases bottom PTY terminal drawer by +2 rows |
| `Ctrl+Down` / `Alt+{` | **Shrink Terminal Height** | Decreases bottom PTY terminal drawer by -2 rows |
| `Ctrl+T` / `` Ctrl+` `` | **Toggle Terminal** | Shows or hides the bottom integrated terminal drawer |
| `Ctrl+B` | **Toggle Sidebar** | Toggle pinned folders and OS shortcuts navigation sidebar |
| `F12` | **Diagnostics Overlay** | Toggle frame render latency, FPS, and hit-testing geometry |
| `Ctrl+Q` | **Quit TermExplorer** | Cleanly terminate all child sessions and restore shell terminal |
| `Esc` | **Universal Escape** | Closes any modal, dialog, or search and returns focus to File Explorer |

---

## 2. File Explorer Navigation & Operations

| Key | Action | Description |
|---|---|---|
| `↓` / `j` | **Move Down** | Select next file or folder in directory listing |
| `↑` / `k` | **Move Up** | Select previous file or folder in directory listing |
| `→` / `Enter` / `l` | **Open / Descend** | Descend into highlighted folder or open file in viewer |
| `←` / `Backspace` / `h` | **Ascend Directory** | Navigate up to parent folder |
| `Space` | **Focus Preview** | Move keyboard focus immediately into Contextual Preview |
| `Home` / `End` | **Jump Top / Bottom** | Instantly select first or last entry in directory |
| `PageUp` / `PageDown` | **Scroll Page** | Fast-scroll directory listing by one screen |
| `/` or `Ctrl+F` | **Inline Fuzzy Filter** | Real-time search filter matching filenames instantly |
| `.` or `Ctrl+H` | **Toggle Hidden Files** | Show or hide dotfiles (`.gitignore`, `.env`, etc.) |
| `s` | **Cycle Sort Order** | Sort by Name, Size, Date Modified, or Extension |
| `a` / `F4` | **New File** | Prompt to create a new file in current directory |
| `A` / `F7` | **New Directory** | Prompt to create a new subdirectory in current directory |
| `r` | **Rename Entry** | In-place rename modal for highlighted file or folder |
| `d` / `F8` / `Del` | **Delete Entry** | Move selected item to FreeDesktop Trash with confirmation |
| `c` / `y` | **Copy Entry** | Copy highlighted file or folder to application clipboard |
| `x` | **Cut Entry** | Cut highlighted file or folder for moving |
| `v` / `p` | **Paste Entry** | Paste clipboard item into current directory (with safety checks) |
| `Alt+P` | **Pin Folder** | Pin current directory to workspace sidebar bookmarks |
| `F2` | **Open in Editor** | Open highlighted file in the integrated code editor |
| `F5` | **Refresh Directory** | Force re-read of directory tree and update Git status |

---

## 3. Contextual Preview & Inspector (`75%` Screen Width)

| Key | Action | Description |
|---|---|---|
| `1` | **Preview Tab** | Jump directly to syntax-highlighted code / markdown / media viewer |
| `2` | **Info Tab** | Jump directly to filesystem metadata, permissions, hashes, and size |
| `3` | **Hex / Metadata Tab** | Jump directly to hex binary viewer and low-level byte inspector |
| `4` | **Actions Tab** | Jump directly to contextual actions (Open with, Extract, Run, Play) |
| `[` / `]` | **Previous / Next Tab** | Cycle active preview tab |
| `↓` / `j` | **Scroll Down** | Scroll preview content line by line |
| `↑` / `k` | **Scroll Up** | Scroll preview content upwards |
| `PageDown` / `PageUp` | **Page Scroll** | Scroll preview by 20 lines |
| `Home` / `End` | **Start / End of File** | Instantly jump to line 1 or end of document |
| `1` – `9` (in Actions) | **Execute Action** | Trigger numbered action directly |
| `Esc` / `Backspace` / `h` | **Return to Explorer** | Switch keyboard focus back to File Explorer |

---

## 4. Integrated Code Editor (`F2` / `Enter`)

| Key | Action | Description |
|---|---|---|
| `Ctrl+S` | **Save Buffer** | Write changes atomically to disk with directory sync |
| `Ctrl+Z` | **Undo** | Revert latest buffer edit |
| `Ctrl+Y` | **Redo** | Restore previously reverted edit |
| `Ctrl+C` | **Copy Line** | Copy active line to clipboard |
| `Ctrl+X` | **Cut Line** | Cut active line to clipboard |
| `Ctrl+V` | **Paste** | Paste text at cursor position |
| `Tab` | **Accept Completion / Indent** | Accept autocomplete suggestion or indent by `tab_width` spaces |
| `Enter` | **Newline** | Insert newline with automatic bracket indentation |
| `Backspace` | **Smart Backspace** | Delete character (or matched pair if between delimiters) |
| `Esc` | **Close Editor** | Exit editor (prompts with safety warning if buffer is dirty) |

---

## 5. Integrated Linux Terminal Drawer (`Ctrl+T` / `` Ctrl+` ``)

| Key | Action | Description |
|---|---|---|
| `Alt+Up` / `Alt+k` | **Focus to Explorer** | Return focus from shell prompt to File Explorer |
| `Alt+F6` / `Ctrl+Shift+T` | **Maximize Drawer** | Toggle terminal drawer to ~80% workstation height |
| `Ctrl+Up` / `Ctrl+Down` | **Adjust Height** | Resize drawer height while inside the terminal |
| `Ctrl+T` / `` Ctrl+` `` | **Hide Drawer** | Hide terminal drawer without killing background shell process |
| `Ctrl+C` | **SIGINT** | Standard terminal interrupt signal sent directly to PTY shell |
| `Ctrl+D` | **EOF** | End-of-file / exit shell session |
| `Ctrl+L` | **Clear Screen** | Standard terminal screen clear |

---

## 6. Interactive Settings Studio (`Ctrl+,` / `F10`)

| Key | Action | Description |
|---|---|---|
| `Tab` | **Switch Column** | Toggle focus between Section List (left) and Item Controls (right) |
| `↑` / `↓` / `k` / `j` | **Select Item** | Navigate sections or configuration settings |
| `Space` / `Enter` | **Toggle Value** | Toggle booleans or activate selected profile |
| `←` / `→` | **Adjust Numeric** | Change percentages, widths, or cycle dropdown options |
| `q` / `Esc` / `F10` | **Save & Exit** | Atomically commit validated settings with automatic backup |

---

## 7. Git Workbench (`Ctrl+G` / `g`)

| Key | Action | Description |
|---|---|---|
| `↑` / `↓` / `k` / `j` | **Select Change** | Navigate list of modified, staged, or untracked files |
| `s` | **Stage File** | Stage selected file (`git add <file>`) |
| `u` | **Unstage File** | Unstage selected file (`git restore --staged <file>`) |
| `c` | **Commit Changes** | Prompt for commit message and commit staged changes |
| `q` / `Esc` | **Close Workbench** | Return to file explorer view |

---

## 8. Plugin Marketplace & Extension Center (`Ctrl+Shift+X`)

| Key | Action | Description |
|---|---|---|
| `1` / `2` / `3` / `Tab` | **Switch Tabs** | 1: Marketplace Catalog, 2: Installed Plugins, 3: Permissions |
| `↑` / `↓` / `k` / `j` | **Select Extension** | Navigate plugin entries |
| `i` / `Enter` | **Install / Toggle** | Install from marketplace (tab 1) or toggle enable/disable (tab 2) |
| `d` / `u` | **Uninstall** | Remove installed extension and purge on-disk assets |
| `r` | **Reload Extensions** | Rescan plugin directory from disk |
| `c` | **Cycle Category** | Filter marketplace plugins by category |
| `Esc` | **Exit Center** | Return to file explorer view |
