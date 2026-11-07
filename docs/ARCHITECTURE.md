# TermExplorer Systems Architecture

**TermExplorer** is an asynchronous, event-driven Terminal User Interface (TUI) developer environment designed for high-performance file management, live coding, and system telemetry directly within Linux terminal emulators.

---

## 1. High-Level Architectural Diagram

<p align="center">
  <img src="images/architecture-diagram.svg" alt="TermExplorer Systems Architecture and Dataflow" width="100%" />
</p>

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ TermExplorer Binary Event & State Architecture                               │
│                                                                              │
│  ┌──────────────────────┐  Tokio Event Loop   ┌──────────────────────┐       │
│  │  Crossterm Backend   │ ──────────────────> │    App Core State    │       │
│  │(Raw Mode / KeyEvents)│                     │   (App / AppState)   │       │
│  └──────────────────────┘                     └──────────┬───────────┘       │
│              ▲                                           │                   │
│              │ Render Pass (60 FPS / on-event)           ▼                   │
│  ┌───────────┴──────────┐                     ┌──────────────────────┐       │
│  │    Ratatui Render    │ <────────────────── │Modular UI Components │       │
│  │ (Transparent Engine) │                     │(Explorer,Preview,etc)│       │
│  └──────────────────────┘                     └──────────────────────┘       │
└────────┬───────────────────┬───────────────────┬───────────────────┬─────────┘
         ▼                   ▼                   ▼                   ▼          
 ┌────────────────┐  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐ 
 │Filesystem/Pane │  │ Univ. Preview  │  │  Terminal PTY  │  │  Code Editor   │ 
 │ Dual-Pane (F7) │  │ 12+ Categories │  │  portable-pty  │  │ Syntect Syntax │ 
 │ Desktop Trash  │  │24-bit Halfblock│  │Non-blocking I/O│  │  Auto-Pairing  │ 
 │  Fuzzy / Sort  │  │ 64MB LRU Cache │  │  Native Shell  │  │  Autocomplete  │ 
 └────────────────┘  └────────────────┘  └────────────────┘  └────────────────┘ 
         │                   │                   │                   │          
         └───────────────────┴─────────┬─────────┴───────────────────┘          
                                       ▼                                        
┌──────────────────────────────────────────────────────────────────────────────┐
│ Ratatui Terminal Renderer & Layout Engine                                    │
│  - 24-bit Truecolor Half-Block Graphics Engine (▀)                           │
│  - Transparent Background Inheritance (bg: None)                             │
│  - Approved Workstation Layout: 25% File Explorer / 75% Contextual Preview   │
│  - Compact Integrated Terminal Drawer: 15% height when active, OFF by default│
│  - Full vertical content expansion (95% height) when terminal is hidden      │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Core Subsystems

### A. Main Application Loop & State Machine (`src/app.rs`)
- Implemented with `tokio::select!` over multiple concurrent event streams:
  1. User keyboard & mouse inputs from `events::start_event_loop`.
  2. Asynchronous PTY terminal byte stream from `pty_rx`.
  3. Real-time filesystem notifications from `notify` file watcher (`fs_rx`).
  4. Periodic tick timers for background resource monitor sampling and screen redraws.
- Features modal state machine: `AppMode::Browse`, `AppMode::Edit`, `AppMode::Terminal`, `AppMode::Search`, `AppMode::CommandPalette`, `AppMode::SettingsStudio`, `AppMode::GitStatus`, `AppMode::Onboarding`.

### B. Universal Multi-Format Preview Subsystem (`src/preview/`)
The preview engine unifies inspection across 12+ format categories into 4 UI tabs:
1. **Magic Bytes Detection (`detector.rs`):** Inspects binary signatures to classify formats before falling back to extensions.
2. **Capability Registry (`registry.rs`):** Tracks built-in pure-Rust parsers and optional CLI helpers (`ffmpeg`, `ffprobe`, `pdftoppm`, `pdftotext`, `gio`, etc.).
3. **Truecolor Half-Block Graphics (`rendering.rs`):** Converts images and video thumbnails into 24-bit ANSI half-block characters (`▀`), packing 2 pixels vertically per cell.
4. **LRU Cache (`cache.rs`):** Bounded cache (default 64 MB / 100 entries) storing rendered lines and metadata to eliminate repeated disk reads during rapid directory navigation.
5. **Format Handlers:**
   - `source/`: Syntax-highlighted code streams with line numbers.
   - `markdown/`: Rendered headers, code fences, blockquotes, and auto-generated Table of Contents.
   - `image/`: PNG, JPEG, GIF, WebP, BMP decoding & SVG dimension extraction.
   - `video/`: Media container inspection with `ffprobe` and frame thumbnail generation with `ffmpeg`.
   - `audio/`: Sample rates, durations, channels, ID3 tags (artist, album, title).
   - `pdf/`: Header / catalog inspection (page count, author) & `pdftotext` stream extraction.
   - `office/`: Pure-Rust ZIP XML extraction for Word `.docx`, Excel `.xlsx`, and PowerPoint `.pptx`.
   - `data/`: Delimiter sniffer for CSV/TSV tables and JSON pretty-printer with syntax validation.
   - `archive/`: ZIP, TAR, TGZ catalog with path traversal vulnerability scanning (`is_safe_archive_path`).
   - `database/`: SQLite 3 header parser, schema viewer, and row sampler.
   - `hex/`: 16-byte canonical hex dump with memory offsets, hex bytes, and printable ASCII.

### C. Filesystem & Dual-Pane Engine (`src/filesystem/`)
- `browser.rs`: High-performance directory scanner with configurable sorting (Name, Size, Modified Date, File Extension).
- `operations.rs`: Safe file operations, directory creation, rename, and copy. Implements **FreeDesktop Trash integration** using `gio trash` and `trash-put` before permanent deletions. Atomic file saves use `tempfile::NamedTempFile`.
- **Dual-Pane Mode:** Primary and secondary panes run independent browser instances side-by-side, allowing instant file transfers (`c` for copy, `m` for move).

### D. Interactive Settings Studio (`src/settings/`)
- `schema.rs`: 15 typed configuration sections (`General`, `Appearance`, `Keybindings`, `Explorer`, `Preview`, `Editor`, `Terminal`, `System Monitor`, `Git`, `Bookmarks`, `Profiles`, `Performance`, `Plugins`, `Advanced`, `About`).
- `keybindings.rs`: Scoped action registry with automated conflict detection algorithm across overlapping scopes.
- `profiles.rs`: 6 built-in layout presets (`Default`, `Developer`, `Minimal`, `TerminalPowerUser`, `EditorFocus`, `Monitoring`).
- `persistence.rs`: Boundary clamping, validation, and atomic `.toml.bak` backup creation prior to configuration writes.

### E. Embedded Pseudo-Terminal (`src/shell/`)
- Real Linux pseudo-terminal powered by `portable-pty`. Spawns user's `$SHELL` inside a master/slave pair.
- Background reader thread pumps bytes into a non-blocking channel to prevent TUI rendering locks.
- Resilient child process termination prevents orphaned processes on exit.

### F. Integrated Code Editor (`src/editor/`)
- Pure Rust buffer with cursor navigation, multi-level Undo/Redo, and line-based clipboard.
- Intelligent auto-pairing for `()`, `[]`, `{}`, `""`, `''`, ````.
- Word prefix completion engine with floating candidate popup.
- Syntax highlighting via `syntect`.

### G. Themes & Layout System (`src/theme.rs` & `src/ui/`)
- 12 built-in themes: `Default Dark`, `Default Light`, `Catppuccin Mocha`, `Catppuccin Macchiato`, `Dracula`, `Nord`, `Gruvbox`, `Tokyo Night`, `One Dark`, `Solarized Dark`, `Solarized Light`, `Monokai`.
- Native transparent terminal background inheritance (`bg: None`).

---

## 3. Reliability & Zero-Panic Safety Guarantees

- **Terminal Raw Mode Safety:** RAII cleanup and panic hook ensure terminal alternate screen and raw mode are restored even during unexpected aborts.
- **Explicit Error Propagation:** Replaces all unsafe runtime unwrap calls with `anyhow` contexts and typed enums.
- **Bounded Resource Allocations:** Preview file sizes, LRU cache memory, and editor undo stacks have strict upper boundaries to prevent OOM scenarios.
