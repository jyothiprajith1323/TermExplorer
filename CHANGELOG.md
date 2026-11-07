# Changelog

All notable changes to **TermExplorer** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] - 2026-11-07

### Added
- **Core TUI Architecture**:
  - Implemented responsive four-panel and five-panel terminal layout with `ratatui` and `crossterm`.
  - Configurable panel splits (Sidebar, Explorer, Preview, Editor, Terminal, System Monitor).
  - Truecolor and terminal background transparency support (`bg: None`).

- **Universal Multi-Format Preview Engine (`src/preview/`)**:
  - Four preview inspector tabs: `[Preview]`, `[Information]`, `[Hex]`, and `[Actions]`.
  - Format Sniffer with Magic Bytes detection for binary/archive/media containers.
  - Native Half-Block (`▀`) 24-bit Truecolor terminal image rendering and SVG parsing.
  - Video and Audio metadata inspection with `ffprobe`/`ffmpeg` thumbnail generation.
  - PDF document inspector and text extractor (`pdftotext`/`pdfinfo`).
  - Native pure-Rust Office document inspector (DOCX paragraphs, XLSX shared strings/sheets, PPTX slide count).
  - Tabular Data viewer with automatic delimiter sniffing (CSV/TSV) and JSON pretty-printer with syntax validation.
  - Archive inspector (ZIP, TAR, TGZ) with path traversal vulnerability auditing.
  - SQLite 3 database header parser, schema viewer, and row sampler.
  - 16-byte Hex Dump viewer with byte offset and printable ASCII pane.
  - Bounded LRU Cache (max 64 MB / 100 entries) preventing repeated disk I/O.

- **Dual-Pane File Explorer**:
  - Side-by-side Dual-Pane mode toggled with `F7`.
  - Independent navigation and current directory tracking for both panes.
  - Quick cross-pane file operations (Copy with `c`, Move with `m`).
  - FreeDesktop Trash integration (`gio trash`, `trash-put`, fallback to trash directory).
  - Bookmark and pinning manager (`b`, `p`).

- **Interactive Settings Studio (`F10`)**:
  - Modal dual-pane settings interface covering 15 configuration categories.
  - Real-time parameter modification (`Bool`, `Choice`, `Number`, `Text`).
  - Keybinding Manager with full scope separation and conflict detection.
  - 6 Built-in Configuration Profiles: `Default`, `Developer`, `Minimal`, `TerminalPowerUser`, `EditorFocus`, `Monitoring`.
  - Automatic `.toml.bak` backup creation and boundary clamping validation.

- **12 Built-in Themes**:
  - Default Dark, Default Light, Catppuccin Mocha, Catppuccin Macchiato, Dracula, Nord, Gruvbox, Tokyo Night, One Dark, Solarized Dark, Solarized Light, Monokai.
  - Dedicated theme preview in onboarding and settings studio.

- **Productivity & Developer Tools**:
  - Fuzzy-search Command Palette (`Ctrl+Shift+P` / `P`).
  - Git status overlay panel (`g`) with branch tracking, staged/unstaged views, and commit history.
  - Smart Workspace detector for Cargo, Python, Node.js, Go, C/C++, Java, and Makefiles with contextual build/run/test actions.
  - First-launch interactive Onboarding Wizard (`F1` or automatically on clean state).
  - Embedded PTY terminal (`portable-pty`) with non-blocking I/O and direct child process signals.
  - Lightweight syntax-highlighted code editor with auto-pairing, undo/redo, and word completion.
  - Live system resource monitor tracking per-core CPU, RAM, Swap, and top processes.

- **Testing & Quality Assurance**:
  - 42 comprehensive automated integration tests across 8 test suites.
  - Strict Clippy compliance with zero warnings (`-D warnings`).
  - GitHub Actions CI workflow, Pages deployment, and Release packager.
