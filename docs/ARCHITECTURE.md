# TermExplorer Architecture

TermExplorer is an asynchronous, event-driven Terminal User Interface (TUI) application designed for high-performance file management, live coding, and system inspection directly within Linux terminal emulators.

---

## 1. High-Level Architectural Diagram

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                             TermExplorer Binary                             │
│                                                                             │
│  ┌───────────────────────┐  Tokio Event Loop   ┌─────────────────────────┐  │
│  │   Crossterm Backend   │ ──────────────────> │      App Core State     │  │
│  │ (Raw Mode / KeyEvents)│                     │       (App / AppState)  │  │
│  └───────────────────────┘                     └────────────┬────────────┘  │
│              ▲                                              │               │
│              │ Render Pass (100ms / on-event)               ▼               │
│  ┌───────────┴───────────┐                     ┌─────────────────────────┐  │
│  │     Ratatui Render    │ <────────────────── │  Modular UI Components  │  │
│  │  (Transparent Engine) │                     │ (Explorer, Editor, etc) │  │
│  └───────────────────────┘                     └─────────────────────────┘  │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │
     ┌───────────────────┬─────────────┴─────┬───────────────────┐
     ▼                   ▼                   ▼                   ▼
┌──────────────┐  ┌──────────────┐   ┌───────────────┐   ┌──────────────┐
│  Filesystem  │  │   Embedded   │   │  Code Editor  │   │    System    │
│    Engine    │  │ Terminal PTY │   │ & Auto-Pairer │   │   Monitor    │
│  (std::fs /  │  │(portable-pty │   │  (Syntect /   │   │  (sysinfo)   │
│   notify)    │  │ + background)│   │  Completions) │   │              │
└──────────────┘  └──────────────┘   └───────────────┘   └──────────────┘
```

---

## 2. Core Subsystems

### A. Main Application Loop (`src/app.rs`)
- Implemented with `tokio::select!` over multiple concurrent event sources:
  1. User keyboard & mouse inputs from `events::start_event_loop`.
  2. Asynchronous PTY terminal byte stream from `pty_rx`.
  3. Real-time filesystem notifications from `notify` file watcher (`fs_rx`).
  4. Periodic tick timers for background resource monitor sampling and screen redraws.
- Avoids blocking the main UI thread by offloading heavy work or polling into asynchronous channels.

### B. Filesystem Layer (`src/filesystem/`)
- `browser.rs`: High-performance directory scanner with configurable sorting (Name, Size, Modified Date, File Extension). Directories always float to the top for intuitive navigation.
- `operations.rs`: Safe filesystem operations including `create_file`, `create_dir`, `rename_entry`, `delete_entry` (recursive), and `copy_entry` (recursive directory copy). Atomic file saves use `tempfile::NamedTempFile` with automatic destination replacement to eliminate file corruption risks.
- `metadata.rs`: File permissions, formatted size, modification timestamp, MIME type heuristic deduction, and binary file sniffing (sampling byte distributions to prevent terminal corruption from binary dumps).
- `search.rs`: Real-time fuzzy filtering using `fuzzy-matcher` (Skim algorithm) and case-insensitive substring search.
- `watcher.rs`: Native Linux inotify file watcher using `notify::RecommendedWatcher` with Tokio unbounded message channels.

### C. Embedded Pseudo-Terminal (`src/shell/`)
- Real Linux pseudo-terminal powered by `portable-pty`.
- Resolves the user's default `$SHELL` (with fallback to `/bin/bash`), inherits environment variables (`TERM=xterm-256color`), and executes in the current explorer directory.
- Background reader thread moves raw bytes from the master PTY into Tokio async channels, allowing interactive applications (Bash, Zsh, Top, Git, Vim) to execute naturally with tab completions, ANSI color sequences, and signal processing (`Ctrl+C`, `Ctrl+D`, etc.).
- `session.rs`: Handles live resizing, input forwarding, process lifecycle, and cleanup on exit via `Child::kill` and `Child::wait`.

### D. Integrated Text Editor (`src/editor/`)
- `buffer.rs`: Text buffer with full cursor tracking, multi-level undo/redo history stacks, home/end, page navigation, line jumping, and clipboard copy/cut/paste.
- `auto_pair.rs`: Intelligent parenthesis, bracket, and quote auto-pairing (`()`, `[]`, `{}`, `""`, `''`, ````). Automatically skips over matching closing symbols when typed and deletes pairs on backspace.
- `completion.rs`: Word extraction and prefix matching across buffer lines with floating candidate popup rendering, Up/Down selection, and Tab/Enter completion insertion.
- `syntax.rs`: High-performance syntax highlighting using `syntect` with default Sublime syntax definitions for Rust, Python, C, C++, Java, JavaScript, TypeScript, Markdown, HTML, CSS, JSON, TOML, YAML, and Shell scripts.
- `lsp.rs`: Language Server Protocol environment prober detecting `rust-analyzer`, `clangd`, `pyright`, and `typescript-language-server`.

### E. System Resource Monitor (`src/monitor/`)
- Powered by `sysinfo` to deliver real-time metrics without external dependencies like `htop`.
- `cpu.rs`: Global CPU percentage, per-core CPU usage, CPU model brand string, and 1/5/15 minute load averages.
- `memory.rs`: Total, used, and available RAM, memory usage percentage, and swap usage.
- `disk.rs`: Mounted block devices and filesystems, total/used/free storage, and utilization percentages.
- `processes.rs`: Top 20 processes sorted dynamically by CPU utilization or memory footprint.

### F. Theme & Presentation (`src/theme.rs` & `src/ui/`)
- Transparent-friendly terminal rendering: relies on terminal default background colors (`Color::Reset` / transparent) so native terminal transparency and blur effects remain visible.
- Professional dark developer theme inspired by One Dark / Catppuccin palettes.
- Modular layout engine with dynamic panel toggling (F6 for monitor, Ctrl+` for terminal, Ctrl+B for sidebar).

---

## 3. Reliability & Terminal Safety

- Terminal initialization puts standard output into raw mode and switches to alternate screen buffer.
- Terminal restoration is guaranteed upon application exit (even on errors) via explicit cleanup in `main.rs` before exiting.
- Child PTY processes are explicitly dropped and reaped on shutdown to eliminate orphaned zombie processes.
