# Changelog

All notable changes to **TermExplorer** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.1.0-RC2] - 2026-11-07

### Approved Product Layout Architecture
- **25% Explorer / 75% Preview Workspace**:
  - Affirmed 25% File Explorer and 75% Contextual Preview as the approved product layout standard.
  - Terminal visibility set to OFF by default on fresh installations (`show_terminal = false`).
  - Terminal drawer compact 15% height when active (`terminal_height_percent = 15`).
  - Main workspace automatically expands to full vertical height (38 of 40 rows, 95% screen real-estate) when terminal is closed.
  - Added native `[layout]` configuration table in `config.toml` with backward compatibility and synchronization with legacy top-level keys.

### Filesystem Durability & Data Safety Audit
- **Storage Synchronization**:
  - Enhanced `save_file_atomic` with `tmp.as_file().sync_all()` to flush physical storage media prior to atomic rename.
  - Added parent directory metadata synchronization (`dir_file.sync_all()`) to ensure atomic directory entry commitment on POSIX filesystems.
  - Clarified distinction between atomic replacement and power-loss durability.

### WebAssembly Security Enforcement
- **Native Runtime Memory Interception**:
  - Implemented `wasmi::ResourceLimiter` on `HostState` to intercept `memory.grow` instructions during guest execution.
  - Enforced strict 16 MiB linear memory ceiling across initial module instantiation and runtime heap expansion.
  - Added `tests/adversarial_plugin_tests.rs` with 5 automated adversarial tests validating fuel limits, loop traps, memory bounds, and capability permissions.

### Performance Validation Suite
- **Comprehensive Benchmarks (`tests/performance_benchmarks.rs`)**:
  - Added reproducible automated benchmark suite measuring 10 key performance indicators across debug and release profiles.
  - Verified 41.25 ms startup time, 5.13 MB base RSS, 0.36 µs typing latency, and ~3,597 FPS TUI redraw capability.

### User-Friendly Keybindings & Workstation Ergonomics
- **Intuitive Panel Routing & Collision Elimination**:
  - Unlocked panel resizing up to 85% width via `Ctrl+Right` / `Alt+]` and `Ctrl+Left` / `Alt+[` with synchronized `config.layout` ratios.
  - Resolved panel resize collision: `[` and `]` cycle preview tabs when Preview is active, without triggering unintended resizing.
  - Direct preview tab jumping with `1` (Preview), `2` (Info), `3` (Hex/Metadata), and `4` (Actions).
  - Quick focus transfer: `Space` moves focus from Explorer directly into Contextual Preview; `Esc` or `h` returns to Explorer.
  - Full Git Workbench integration: `Ctrl+G` and `g` open the Git overlay with Vim `j`/`k` navigation and staging actions (`s`, `u`, `c`).
  - System Telemetry access: `F6` and `Ctrl+M` toggle the live hardware monitor.
  - Added Explorer shortcuts: `a` (New File), `A` (New Directory), `d` / `Del` (Delete to Trash), `r` (Rename), `c`/`y` (Copy), `x` (Cut), `p`/`v` (Paste).
  - Added dedicated test suite `tests/keybinding_and_panel_verification_tests.rs` (9 automated tests).
### Advanced Editor Engine & Multi-Layer Autocomplete
- **Rope-Backed Text Buffer**:
  - Integrated `ropey` rope data structure for fast insertions, deletions, visual block selections, and bounded undo/redo.
  - Multi-language line comment toggling (`Ctrl+/` or `Ctrl+_`) for 20+ file formats.
  - Viewport-clamped syntax highlighting with tokenized styling for Rust, Python, C/C++, Java, SQL, TOML, Markdown, YAML, JSON, Go, Bash, HTML, CSS.
  - Search and replace (`replace_current`, `replace_all`) with character-safe indexing.
- **4-Layer Autocomplete Engine**:
  - Layer 1: Fast in-buffer word extractor (<1 ms).
  - Layer 2: Asynchronous workspace symbol indexer (<26 ms index, <0.2 ms queries).
  - Layer 3: Multi-language code snippets and templates (<0.02 ms).
  - Layer 4: Async LSP provider integration with cancellation tokens and fallback keywords.
  - Multi-signal ranking and deduplication with floating editor completion popup.
- **Unicode & Preview Ergonomics**:
  - Added `src/fonts/unicode.rs` for display-column slicing, character/column bidirectional conversions, and UTF-8 panic-free rendering.
  - Fixed preview mouse scrolling offset runaway by clamping to document total lines.
  - Added horizontal preview navigation (`Shift+Left`/`H`, `Shift+Right`/`L`).
  - Replaced hardcoded block glyphs in markdown preview with clean standard Markdown headings and checkboxes.
- **Comprehensive Quality Assurance**:
  - Total automated test inventory expanded to **129 tests across 26 test suites** with 100% pass rate.
  - Zero compiler or Clippy warnings (`-D warnings`).

---

## [1.1.0-RC1] - 2026-11-07

### Architectural & UI Redesign
- **Two-Column Workstation Layout**:
  - Completely eliminated the permanent left workspace panel (bookmarks, pinned directories, disk locations) from default screen real-estate to maximize productivity space.
  - New default horizontal proportions: **25% File Explorer** and **75% Preview/Inspector**.
  - Pinned folders and locations remain accessible via Command Palette (`Ctrl+Shift+P`), Quick Jump (`Ctrl+P`), and Breadcrumbs.
  - Multi-tier adaptive responsive layout: gracefully collapses or expands across 6 terminal width tiers (from 60 columns to 160+ columns).
- **Integrated Terminal Drawer**:
  - Reduced default vertical height to a compact 15% drawer (or hidden by default).
  - Terminal does not auto-open on startup; opens cleanly on hotkey (`Ctrl+T` or backtick).
  - Compact terminal header controls with exact hit-testing: session title, shell name, CWD, new session, close, maximize, minimize, and resize handle.
- **Redesigned Help Menu Modal**:
  - Structured 6-category navigation (`[1] Global`, `[2] Navigation`, `[3] File Ops`, `[4] Editor`, `[5] Terminal`, `[6] Plugins`).
  - GitHub-style `<kbd>` key badges, workflow pro-tips, category tabs, and scroll indicators.
  - Full mouse hit-testing support for tab switching and dismissal.

### Reliability & Data Safety Guarantees
- **Zero Silent Data Loss Filesystem Architecture**:
  - Atomic file saving (`save_file_atomic`) via temporary files with POSIX permission preservation.
  - Concurrent modification protection: editor buffer tracks `loaded_mtime` and rejects silent overwrites if a file changes externally.
  - Unsaved buffer crash recovery snapshots automatically persisted to `~/.local/share/termexplorer/recovery/`.
  - Name collision policies (`CollisionPolicy::{Fail, Overwrite, AutoRename}`) with deterministic `file (1).ext` auto-renaming.
  - Cross-filesystem move support handling `EXDEV` (errno 18) via verified copy-verify-unlink with automatic rollback on error.
  - FreeDesktop `.trashinfo` metadata compliance (`[Trash Info]` headers) with bidirectional restore support (`restore_from_trash`).
- **Configuration Resilience & Self-Healing**:
  - Atomic configuration saving with automated `.bak` backup generation.
  - Corrupted configuration recovery: falls back to `.bak` if primary is malformed, moves corrupt file to timestamped quarantine archive, and regenerates safe defaults.
  - Added schema-validated export (`export_to_file`) and import (`import_from_file`).

### Systems Stability & Sandboxing
- **Terminal Raw Mode & Panic Recovery**:
  - Installed emergency panic hook restoring terminal raw mode, leaving alternate screens, and disabling mouse capture if an unhandled panic occurs.
  - Guaranteed child process tree termination on terminal session drop.
- **WebAssembly Security Enforcement**:
  - Enforced a hard 16 MiB linear memory ceiling per WASM plugin instance in `wasmi` (v0.36).
  - Metered execution fuel (`5,000,000` steps) preventing infinite loops.
  - Strict capability verification rejecting undeclared syscalls.

### Distribution & Packaging
- **Automated POSIX Shell Installer (`install.sh`)**: Rust toolchain verification, optimized release compilation, and user-space binary placement.
- **Arch Linux / AUR Packaging**: Created `packaging/aur/PKGBUILD` for automated Makepkg builds.
- **Website Modernization**: Overhauled website design with Catppuccin Mocha tokens, ambient mesh lighting, responsive CSS grid, interactive multi-tab installation widget, and updated SVG UI diagram reflecting the 25/75 workspace split.

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
