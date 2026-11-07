# 🚀 TermExplorer

**A Modern, Keyboard-Driven Terminal File Explorer, Integrated Code Editor, Embedded Shell, and System Telemetry Dashboard in Rust.**

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Ubuntu-orange.svg)]()

TermExplorer is a fast, keyboard-driven terminal developer environment that combines the convenience of a graphical file manager, the speed of an embedded code editor with syntax highlighting and auto-pairing, a real interactive Linux pseudo-terminal (PTY) with full native shell completion, and a live system monitoring dashboard — completely inside your Ubuntu terminal.

---

## 📸 Interface Layout

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  TERMEXPLORER                             Current Path      System Status     │
├───────────────┬──────────────────────────────┬───────────────────────────────┤
│               │                              │                               │
│  SIDEBAR      │       FILE EXPLORER          │        FILE PREVIEW           │
│               │                              │                               │
│  Pinned       │  Name              Size      │                               │
│  ├ Projects   │  📁 IronKV         --        │  Selected file preview        │
│  ├ Rust       │  📁 Documents      --        │                               │
│  └ Workspace  │  📄 Cargo.toml    1.2 KB     │  Syntax highlighting           │
│               │  📄 README.md     4.5 KB     │  File information              │
│  Shortcuts    │  📄 main.rs       2.1 KB     │                               │
│  ├ Home       │                              │                               │
│  ├ Downloads  │                              │                               │
│  ├ Documents  │                              │                               │
│  ├ Root       │                              │                               │
│  └ /etc       │                              │                               │
│               │                              │                               │
├───────────────┴──────────────────────────────┴───────────────────────────────┤
│  TERMINAL / SYSTEM MONITOR                                      [F6 Toggle]  │
├──────────────────────────────────────────────────────────────────────────────┤
│  user@ubuntu:~/workspace$ cargo test --all                                   │
├──────────────────────────────────────────────────────────────────────────────┤
│  F1 Help | F2 Edit | F3 Preview | F4 New | F5 Refresh | F6 Monitor | Ctrl+Q │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## ✨ Features

- **⚡ Fast, Keyboard-Driven Navigation:**
  - Intuitive arrow-key and shortcut navigation.
  - Interactive left sidebar with Pinned folders and Linux standard shortcuts (`Home`, `Documents`, `Downloads`, `Root`, `/etc`, `/var`, `/tmp`, etc.).
  - Pinned folders persist automatically across application restarts in `~/.config/termexplorer/config.toml`.
  - In-place directory search with fuzzy matching (`fuzzy-matcher` Skim algorithm).
  - Dynamic sort cycling by Name (▲/▼), Size (▲/▼), Modification Date (▲/▼), and Extension (`s`).
  - Native file operations: Create (`F4`), Delete (`F8`), Rename (`r`), Copy (`c`/`y`), Cut (`x`), Paste (`p`/`v`), Refresh (`F5`).
  - Real-time inotify filesystem watching via `notify` — updates immediately when files are changed outside.

- **🎨 Syntax-Highlighted File Preview:**
  - Instant live preview of selected files as you navigate.
  - Multi-language syntax highlighting powered by `syntect` (Rust, Python, C, C++, Java, JavaScript, TypeScript, Markdown, HTML, CSS, JSON, TOML, YAML, Shell).
  - Line numbers, scrollable viewport, formatted file sizes, and MIME type deductions.
  - Automatic binary file heuristic detection preventing garbled binary dumps.
  - Safe memory bounds for large files.

- **🖥 Embedded Interactive Linux Terminal (PTY):**
  - Real pseudo-terminal powered by `portable-pty` executing your configured `$SHELL` (or Bash fallback).
  - Preserves user aliases, environment variables, command history, and custom prompt formatting.
  - Full native shell autocompletion (Tab completion for commands, files, flags, and arguments).
  - Interactive process support (`htop`, `git`, `ssh`, `nano`, `python`, etc.).
  - Clean process lifecycle management with guaranteed child termination on exit.
  - Dynamic panel resizing and quick toggling (`Ctrl+` `).

- **✏ Integrated In-Terminal Code Editor (`F2`):**
  - Edit and save files without switching out to an external editor.
  - Multi-level Undo (`Ctrl+Z`) and Redo (`Ctrl+Y`).
  - Atomic, safe file writes eliminating corruption (`tempfile` + persist).
  - Intelligent auto-pairing for `( )`, `[ ]`, `{ }`, `" "`, `' '`, and `` ` ``.
  - Closing character skip-over and pair backspace deletion.
  - Contextual word autocompletion with a floating popup suggestion list.
  - Built-in Language Server Protocol (LSP) prober for `rust-analyzer`, `clangd`, `pyright`, and `typescript-language-server`.

- **📊 Live System Telemetry Dashboard (`F6`):**
  - One-touch system monitoring panel without blocking file browsing or terminal tasks.
  - CPU: Model, global usage %, load averages (1/5/15 min), and per-core utilization bars.
  - Memory: Total, used, and free RAM, utilization %, and Swap telemetry.
  - Disks: Mounted devices, available space, total capacity, and usage ratios.
  - Processes: Live top processes sorted by CPU utilization or memory footprint.

- **💎 Clean, Transparent Terminal UI:**
  - Transparent-friendly rendering inheriting terminal emulator transparency and blur settings.
  - Sleek dark theme with One Dark / Catppuccin accents.
  - Responsive layouts adapting to small and large terminal window dimensions.

---

## 🛠 Technology Stack

| Component | Library / Crate | Purpose |
|---|---|---|
| **Language** | Rust 2021 (Stable 1.80+) | Core systems implementation |
| **TUI Engine** | `ratatui` (v0.29) | Modern terminal UI widgets & layout engine |
| **Terminal I/O** | `crossterm` (v0.28) | Raw mode, ANSI rendering, and key events |
| **Async Runtime** | `tokio` (v1) | Non-blocking concurrent event loop |
| **Pseudo-Terminal** | `portable-pty` (v0.8) | Real Linux PTY master/slave shell session |
| **System Telemetry** | `sysinfo` (v0.33) | Real-time CPU, RAM, disk, and process telemetry |
| **Syntax Highlighter** | `syntect` (v5) | High-speed multi-language syntax highlighting |
| **Filesystem Watcher** | `notify` (v7) | Linux inotify directory monitoring |
| **Fuzzy Matcher** | `fuzzy-matcher` (v0.3) | Real-time fuzzy file searching |
| **Serialization** | `serde` + `toml` | Configuration persistence |
| **Error Handling** | `anyhow` + `thiserror` | Robust context-aware error handling |

---

## 📦 Prerequisites & Ubuntu Installation

### Prerequisites
- Ubuntu Linux 20.04 LTS, 22.04 LTS, 24.04 LTS (or Debian / WSL2).
- Standard C library and POSIX PTY support (default on all Ubuntu systems).

### 1. Install Rust
If Rust is not already installed:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env
```

Verify your installation:
```bash
rustc --version
cargo --version
```

### 2. Clone and Build TermExplorer
```bash
git clone https://github.com/jyothiprajith1323/TermExplorer.git
cd termexplorer
cargo build --release
```

### 3. Install Globally
```bash
cargo install --path .
```
The executable `termexplorer` is now available directly in your `$PATH`.

---

## 🚀 Usage

Launch TermExplorer in your current directory:
```bash
termexplorer
```

Or run from the project source:
```bash
cargo run --release
```

---

## ⌨ Complete Keyboard Shortcuts

### Global Navigation
- `Ctrl+Q`: Exit TermExplorer and restore terminal.
- `Ctrl+` `: Toggle embedded terminal panel.
- `Ctrl+B`: Toggle left sidebar visibility.
- `F1`: Open / close the interactive help overlay.
- `F6`: Toggle system resource monitor.
- `Tab` / `Shift+Tab`: Cycle active focus across panels.
- `Esc`: Cancel search / close popup / return to browse mode.

### File Explorer
- `↑` / `↓`: Move selection in directory list.
- `Enter`: Open selected folder / preview file.
- `Backspace`: Navigate to parent directory.
- `F2`: Edit selected file in integrated text editor.
- `F4`: Create a new file.
- `F5`: Refresh current directory.
- `F7`: Create a new directory.
- `F8`: Delete selected item (prompts for confirmation).
- `r`: Rename selected item.
- `s`: Cycle sort order (`Name`, `Size`, `Date`, `Type`).
- `c` or `y`: Copy selected item.
- `x`: Cut selected item.
- `p` or `v`: Paste item into current directory.
- `h`: Toggle hidden files (`.dotfiles`).
- `Ctrl+F` or `/`: In-directory fuzzy search.
- `Alt+P`: Pin current directory to sidebar.

### Sidebar (when focused)
- `↑` / `↓`: Navigate pinned folders and shortcuts.
- `Enter`: Navigate explorer into highlighted location.
- `d` or `Delete`: Unpin selected pinned folder.

### Integrated Text Editor
- `Ctrl+S`: Save file atomically.
- `Ctrl+Z`: Undo.
- `Ctrl+Y`: Redo.
- `Ctrl+C` / `Ctrl+X` / `Ctrl+V`: Copy / Cut / Paste line.
- `Tab`: Indent (or accept autocompletion suggestion).
- `Enter`: Newline with auto-indent.
- `Esc`: Close editor (prompts if unsaved changes exist).
- Auto-pairing: Type `(`, `[`, `{`, `"`, `'`, ```` to auto-close; Backspace between deletes both.

---

## ⚙ Configuration (`config.toml`)

TermExplorer automatically generates its configuration at:
```text
~/.config/termexplorer/config.toml
```

Example configuration:
```toml
show_sidebar = true
show_preview = true
show_terminal = false
show_monitor = false
sidebar_width = 20
preview_width = 35
terminal_height = 12
monitor_height = 14
show_hidden = false
max_preview_bytes = 524288

[theme]
name = "dark"
accent = "#61AFEF"
transparent_background = true

[editor]
tab_width = 4
auto_pair = true
line_numbers = true
syntax_highlight = true
show_completion = true

[monitor]
refresh_ms = 1000

[[pinned_folders]]
name = "Projects"
path = "/home/user/projects"
```

---

## 🧪 Testing and Verification

TermExplorer includes comprehensive test suites verifying filesystem operations, buffer mechanics, auto-pairing, autocompletion, configuration roundtrips, PTY life-cycle, and system monitoring:

```bash
# Run all unit and integration tests
cargo test

# Run formatting checks
cargo fmt --all -- --check

# Run compilation checks across all targets
cargo check --all-targets

# Run Clippy lints with strict warnings
cargo clippy --all-targets --all-features -- -D warnings
```

---

## 🔧 Architecture

For in-depth architectural details, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).
For a full keybinding reference, see [docs/KEYBINDINGS.md](docs/KEYBINDINGS.md).
For contributor setup instructions, see [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

---

## 🛡 License

This project is licensed under the [MIT License](LICENSE).
