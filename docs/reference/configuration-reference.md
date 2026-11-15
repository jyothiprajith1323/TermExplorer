# Configuration Reference (Schema Version 2)

This reference documents every section, key, data type, default value, and validation clamp range in `~/.config/termexplorer/config.toml`.

---

## 1. Top-Level Key

| Key | Type | Default | Description |
|---|---|---|---|
| `schema_version` | `u32` | `2` | Schema specification version. TermExplorer v1.1.0 targets Schema Version 2. Documents without this key are treated as legacy Version 1 and migrated automatically. |

---

## 2. `[general]` Table

| Key | Type | Default | Description |
|---|---|---|---|
| `nerd_fonts` | `bool` | `true` | When true, renders glyph icons via patched Nerd Fonts. When false, falls back to ASCII/Unicode text glyphs. |
| `confirm_file_delete`| `bool` | `true` | When true, displays an explicit confirmation dialog before moving files to trash or deleting. |
| `restore_session` | `bool` | `true` | When true, restores last visited directory upon startup. |
| `default_workdir` | `String` | `"."` | Default directory opened when no path argument is provided on the command line. |
| `check_for_updates` | `bool` | `false` | Always false (no automated background network calls are made). |
| `first_run_completed`| `bool` | `false` | Tracks whether the user has completed or dismissed first-run onboarding. |

---

## 3. `[layout]` Table

| Key | Type | Default | Clamp Range | Description |
|---|---|---|---|---|
| `explorer_ratio` | `f64` | `0.25` | `0.15` – `0.50` | Horizontal width fraction allocated to File Explorer (default 25%). |
| `preview_ratio` | `f64` | `0.75` | `0.50` – `0.85` | Horizontal width fraction allocated to Preview/Inspector (default 75%). |
| `terminal_height_percent` | `u16` | `15` | `10` – `40` | Height percentage allocated to embedded terminal drawer when opened (default 15%). |
| `dual_pane` | `bool` | `false` | `true` / `false` | When true, enables side-by-side dual-pane file explorer view. |
| `show_sidebar` | `bool` | `false` | `true` / `false` | When true, displays pinned folders and OS bookmarks sidebar. |

---

## 4. `[theme]` Table

| Key | Type | Default | Description |
|---|---|---|---|
| `name` | `String` | `"Catppuccin Mocha"` | Active color palette. Supported: `Catppuccin Mocha`, `Nord`, `Dracula`, `Gruvbox`, `One Dark`, `Solarized Dark`, `Light`. |
| `transparent_background` | `bool` | `true` | When true, passes `None` for background cells to inherit host terminal emulator transparency and blur. |
| `active_border_color` | `String` | `"#89b4fa"` | Hex color string for active panel borders. |
| `inactive_border_color` | `String` | `"#45475a"` | Hex color string for inactive panel borders. |

---

## 5. `[editor]` Table

| Key | Type | Default | Clamp Range | Description |
|---|---|---|---|---|
| `tab_width` | `usize` | `4` | `1` – `8` | Number of spaces inserted per indentation level. |
| `auto_pair` | `bool` | `true` | `true` / `false` | Automatically inserts matching brackets (`()`, `[]`, `{}`) and quotes (`""`, `''`). |
| `line_numbers` | `bool` | `true` | `true` / `false` | Displays line numbering gutter in the editor. |
| `syntax_highlight`| `bool` | `true` | `true` / `false` | Enables TextMate syntax highlighting. |
| `word_wrap` | `bool` | `true` | `true` / `false` | Wraps long lines at panel boundary. |
| `show_completion` | `bool` | `true` | `true` / `false` | Displays autocomplete popup automatically while typing. |

---

## 6. `[terminal]` Table

| Key | Type | Default | Clamp Range | Description |
|---|---|---|---|---|
| `default_shell` | `String` | `""` | Valid binary path | Custom shell path. If empty, automatically discovers `$SHELL` or system shells. |
| `login_shell` | `bool` | `true` | `true` / `false` | When true, launches shell with `-l` to load user shell profiles. |
| `scrollback_limit_kib` | `usize` | `1024` | `128` – `16384` | Maximum in-memory scrollback buffer size in KiB (default 1 MiB). |
| `copy_on_select` | `bool` | `false` | `true` / `false` | Copies highlighted text to clipboard immediately on mouse release. |

---

## 7. `[search]` Table

| Key | Type | Default | Clamp Range | Description |
|---|---|---|---|---|
| `max_results` | `usize` | `1000` | `10` – `50000` | Hard ceiling on returned search matches to prevent memory exhaustion. |
| `case_sensitive` | `bool` | `false` | `true` / `false` | When true, searches case-sensitively by default. |
| `smart_case` | `bool` | `true` | `true` / `false` | Auto-enables case sensitivity if query contains uppercase letters. |
| `follow_symlinks`| `bool` | `false` | `true` / `false` | When true, follows filesystem symlinks during directory walks. |

---

## 8. `[developer]` Table

| Key | Type | Default | Clamp Range | Description |
|---|---|---|---|---|
| `auto_detect_project` | `bool` | `true` | `true` / `false` | Automatically inspects project markers (`Cargo.toml`, `package.json`, etc.). |
| `watch_git` | `bool` | `true` | `true` / `false` | Polls Git repository status in background. |
| `telemetry_rate_ms` | `u64` | `1000` | `250` – `5000` | Telemetry refresh polling interval in milliseconds. |

---

## 9. `[monitor]` Table

| Key | Type | Default | Clamp Range | Description |
|---|---|---|---|---|
| `refresh_ms` | `u64` | `1000` | `100` – `10000` | System monitor telemetry update interval in milliseconds. |

---

## 10. `[keybindings]` Table

| Key | Type | Default | Description |
|---|---|---|---|
| `preset` | `String` | `"default"` | Active keybinding configuration preset (`default`, `vim`, `minimalist`). |
