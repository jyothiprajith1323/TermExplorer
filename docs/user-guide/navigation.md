# Workspace Navigation

TermExplorer offers fluid, low-latency navigation through large file hierarchies using familiar terminal conventions.

---

## 1. Directory Traversal

| Key | Vim Alternative | Action | Description |
|---|---|---|---|
| `↓` | `j` | **Next Item** | Highlight next entry in directory listing |
| `↑` | `k` | **Previous Item** | Highlight previous entry in directory listing |
| `→` or `Enter` | `l` | **Descend / Open** | Enter highlighted folder or preview file |
| `←` or `Backspace` | `h` | **Ascend** | Navigate up to parent directory |
| `Home` | `gg` | **Top of List** | Select first item in active directory |
| `End` | `G` | **Bottom of List** | Select last item in active directory |
| `PageDown` | `Ctrl+D` | **Page Down** | Scroll viewport down by 20 entries |
| `PageUp` | `Ctrl+U` | **Page Up** | Scroll viewport up by 20 entries |

---

## 2. In-Directory Fuzzy Filtering (`/`)

Press `/` while focused in the File Explorer to activate the inline fuzzy filter:
- As you type characters, the directory listing immediately narrows to matching entries.
- Use `↑` / `↓` or `j` / `k` to choose between matched results.
- Press `Enter` to open or enter the matched entry.
- Press `Esc` to cancel the filter and restore the full directory listing.

---

## 3. Hidden Files & Dotfile Toggling

To show or hide hidden files (e.g. `.git`, `.env`, `.config`):
- Press `.` or `zh`.
- The directory view instantly updates without reloading the entire filesystem cache.
- The default visibility can be configured in `config.toml` via `[layout].show_hidden = false`.

---

## 4. Sorting Modes (`s`)

Press `s` to cycle directory sorting:
1. **Name (A → Z)**: Alphabetical directory listing (directories pinned to top).
2. **Size (Descending)**: Largest files first (useful for finding disk usage hogs).
3. **Modified Time**: Most recently updated files first.
4. **Extension**: Grouped by file format (`.rs`, `.toml`, `.md`, `.json`).

---

## 5. Dual-Pane Explorer Mode (`F7`)

Press `F7` to toggle a side-by-side secondary file manager (50% / 50% split):
- Navigate both directories independently.
- Press `Tab` to switch focus between the left and right pane.
- Copy files from active pane to inactive pane with `c`.
- Move files to opposing pane with `m`.
- Press `F7` again to return to the standard inspection layout (25% Explorer / 75% Preview).
