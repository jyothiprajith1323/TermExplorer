# TermExplorer — Integrated Advanced Text Editor Engine

## Overview

TermExplorer features a high-performance terminal code editor engine designed for serious day-to-day software development across Rust, Python, C, C++, Java, SQL, TOML, Markdown, JSON, YAML, Go, Bash, HTML, CSS, JavaScript/TypeScript, and more.

Built with an efficient rope-based text buffer (`ropey`), the editor provides sub-millisecond typing latency, Unicode grapheme safety, multi-level undo/redo, visual selection, multi-language syntax highlighting, and an intelligent 4-layer autocomplete engine.

---

## Key Capabilities

### 1. Rope-Backed Text Buffer
- **Sub-millisecond insertion and deletion**: Backed by a rope structure (`ropey`), ensuring $O(\log N)$ performance even in source files with tens of thousands of lines.
- **Unicode Grapheme Safety**: All cursor positioning, slicing, and deletions operate on character indices and grapheme clusters rather than raw byte offsets, completely eliminating UTF-8 boundary panics.
- **Multiple Line Endings**: Full detection and preservation of both Unix (`LF`) and Windows (`CRLF`) line endings.
- **Atomic Persistence & External Modification Protection**: File saves use tempfile atomic swapping and detect when external processes modify files on disk, preventing accidental overwrites.
- **Crash Recovery Snapshots**: Automatic recovery snapshots are saved to `~/.local/share/termexplorer/recovery/` when unsaved buffers exist, allowing seamless recovery after unexpected shutdowns.

### 2. Syntax Highlighting & Theming
- **Broad Language Support**: 20+ file formats including Rust, Python, C/C++, Java, SQL, TOML, Markdown, JSON, YAML, Go, Bash, HTML, CSS, XML, INI, and Dockerfile.
- **Catppuccin Mocha Integration**: Rich syntax highlighting mapped directly to Catppuccin Mocha palette tokens:
  - Keywords & Storage: Catppuccin Mauve (`#cba6f7`)
  - Functions & Methods: Catppuccin Blue (`#89b4fa`)
  - Types & Structures: Catppuccin Yellow (`#f9e2af`)
  - Strings & Characters: Catppuccin Green (`#a6e3a1`)
  - Comments: Catppuccin Overlay0 (`#6c7086`)
  - Numbers & Constants: Catppuccin Peach (`#fab387`)
  - Operators: Catppuccin Sky (`#89dceb`)
- **Viewport Highlighting**: On-demand highlighting only for visible terminal lines to guarantee zero frame stutter during fast scrolling.

### 3. Visual Selection & Editing Operations
- **Visual Selection**: Start selection (`anchor` to `head`), inspect selected text, cut, copy, or delete selected ranges.
- **Smart Line Comment Toggling (`Ctrl+/` or `Ctrl+_`)**: Automatically toggles language-appropriate comments (`//` for Rust/C/Go, `#` for Python/Bash/YAML/TOML, `--` for SQL).
- **Search & Replace**:
  - `Ctrl+F`: Activate search
  - `F3` / `Shift+F3`: Navigate to next / previous match
  - `replace_current` / `replace_all`: Targeted and global replacement
- **Auto-Pairing**: Automatic insertion and skip-over of brackets `()`, `[]`, `{}`, quotes `""`, `''`, and backticks ````.

---

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `F2` | Open selected file in Editor |
| `Ctrl + S` | Save file atomically to disk |
| `Ctrl + Space` | Explicitly trigger Autocomplete suggestions |
| `Tab` | Accept selected completion / Insert indent tab |
| `Down` / `Up` | Navigate autocomplete popup candidates |
| `Ctrl + Z` | Undo last operation |
| `Ctrl + Y` | Redo last undone operation |
| `Ctrl + C` | Copy line or selection to internal clipboard |
| `Ctrl + X` | Cut line or selection |
| `Ctrl + V` | Paste clipboard content at cursor |
| `Ctrl + /` (`Ctrl+_`) | Toggle line comment |
| `F3` | Search next match |
| `Shift + F3` | Search previous match |
| `Home` / `End` | Smart line home (first non-ws / col 0) and line end |
| `PageUp` / `PageDown` | Scroll viewport up / down by 20 lines |
| `Esc` | Dismiss autocomplete popup / Exit editor to explorer |
