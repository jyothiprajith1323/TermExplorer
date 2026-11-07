# Changelog

All notable changes to **TermExplorer** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.1.0] - 2026-11-07

### Added
- **Catppuccin Theme Ecosystem**:
  - Full canonical 26-color system across all 4 official flavors: **Mocha** (default dark), **Macchiato**, **Frappé**, and **Latte** (light).
  - Strongly-typed `ThemeTokens` mapping background, text, accent, border, status, filesystem, and syntax tokens directly from the palette.
  - Custom TOML theme loader from `~/.config/termexplorer/themes/*.toml` with hex validation and graceful fallback.
  - Transparent terminal background toggle (`bg: None`) across all themes.

- **File and Folder Icon Ecosystem**:
  - 5 switchable icon modes: `Catppuccin` (Nerd Font v3 + palette colors), `NerdFont` (monochrome), `Unicode` (emojis & symbols), `Ascii` (bracketed text tags), and `None`.
  - Comprehensive associations covering 40+ programming languages, 15+ special project files (`Cargo.toml`, `package.json`, `Dockerfile`, etc.), and folder categories (`src/`, `tests/`, `docs/`, `target/`, etc.).
  - Custom overrides and glyph associations via `~/.config/termexplorer/icons.toml`.

- **Typography & Font Support (JetBrains Mono)**:
  - Automated inspection using `fontconfig` (`fc-list` and `fc-match`) to detect `JetBrains Mono` and `JetBrainsMono Nerd Font Mono`.
  - Terminal emulator configuration snippets and guides for Kitty, Alacritty, WezTerm, and GNOME Terminal.
  - User-space non-root font installation script targeting `~/.local/share/fonts/`.

- **WebAssembly Sandboxed Plugin Engine**:
  - Pure-Rust sandboxed plugin runtime powered by `wasmi` (v0.36) and `sha2` (v0.10).
  - Explicit capability boundaries: `commands`, `workspace_actions`, `file_metadata`, `context_actions`, `preview_provider`, and `status_text`.
  - Bounded instruction fuel limits (`5,000,000` steps) preventing infinite loops.
  - Panic and memory crash isolation ensuring guest traps never crash the host application.
  - Cryptographic SHA-256 integrity verification against manifest declarations.
  - Two working example plugins: `plugins/workspace-tools` and `plugins/file-inspector`.
  - Live status bar telemetry display for active plugins.

- **Interactive Personalization & Command Palette**:
  - Real-time theme, icon pack, and font settings in Settings Studio (`F10`).
  - Command Palette shortcuts for instant theme switching, icon packs, font guides, and plugin discovery (`Ctrl+Shift+P` / `Space`).

- **Testing & Quality Metrics**:
  - Expanded test suite to **70 automated integration tests** across 14 test suites with 100% pass rate.
  - Zero clippy warnings under strict `-D warnings` on all targets and features.
  - Full documentation suite in `docs/`: `THEMES.md`, `ICONS.md`, `FONTS.md`, `PLUGINS.md`, `PLUGIN_API.md`, `CUSTOMIZATION.md`, and `V1.1_RELEASE_NOTES.md`.

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
