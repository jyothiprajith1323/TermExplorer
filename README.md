# 🚀 TermExplorer

**Your terminal. Your workspace. Your rules.**

A high-performance, keyboard-driven Linux developer environment written in pure Rust. Combines a dual-pane file explorer, code editor, interactive PTY terminal, live system monitor, Git dashboard, and universal multi-format preview engine into one unified TUI.

[![Continuous Integration](https://github.com/jyothiprajith1323/TermExplorer/actions/workflows/ci.yml/badge.svg)](https://github.com/jyothiprajith1323/TermExplorer/actions/workflows/ci.yml)
[![GitHub Pages](https://github.com/jyothiprajith1323/TermExplorer/actions/workflows/pages.yml/badge.svg)](https://jyothiprajith1323.github.io/TermExplorer/)
[![Rust](https://img.shields.io/badge/rust-1.80%2B%20stable-brightgreen.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Ubuntu-orange.svg)]()
[![Clippy: Clean](https://img.shields.io/badge/clippy-0%20warnings-brightgreen.svg)]()

🌐 **Official Website & Docs:** [https://jyothiprajith1323.github.io/TermExplorer/](https://jyothiprajith1323.github.io/TermExplorer/)

---

## 📸 Interface Overview

<p align="center">
  <img src="docs/images/termexplorer-ui.svg" alt="TermExplorer Modern Terminal Developer Environment" width="100%" />
</p>

```text
┌── SIDEBAR [1] ────┬── FILE EXPLORER [2] ──────────────┬── PREVIEW [Preview|Info|Hex|Actions] ────────────┐
│ * BOOKMARKS       │ SIZE   MODIFIED   NAME            │ Type: Rust Source File (14.2 KB)                 │
│   ~ Home          │ DIR    10:14:02   src/            │ 1  use anyhow::Result;                           │
│   ~/Projects      │ DIR    09:45:11   tests/          │ 2  use ratatui::widgets::Paragraph;              │
│ [+] WORKSPACES    │ 2.4 KB 11:02:30   Cargo.toml      │ 3                                                │
│   > TermExplorer  │ 12.1KB 11:02:30   README.md       │ 4  #[derive(Debug, Clone)]                       │
│     IronKV        │ 14.2KB 11:05:12 > main.rs         │ 5  pub struct AppEngine {                        │
│ [*] SETTINGS      │  1.1KB 08:12:00   build.rs        │ 6      pub terminal: TerminalSession,            │
│   F10 Studio      │                                   │ 7  }                                             │
├───────────────────┴───────────────────────────────────┴──────────────────────────────────────────────────┤
│ termexplorer:~/Projects/TermExplorer$ cargo test --all-targets --all-features                            │
│    Finished `test` profile in 0.45s — 42 passed; 0 failed; 0 warnings                                    │
└──────────────────────────────────────────────────────────────────────────────────────────────────────────┘
 NORMAL  ~/Projects/TermExplorer/src/main.rs | Git: master* | F1:Help F7:Dual-Pane F10:Settings q:Quit
```

---

## ✨ Key Subsystems & Features

### 1. 👁️ Universal Multi-Format Preview Engine (`src/preview/`)
- **4 Dedicated Inspector Tabs:** `[Preview]`, `[Information]`, `[Hex]`, and `[Actions]`.
- **Magic-Byte Sniffer:** Sniffs file signatures directly to prevent incorrect MIME mapping.
- **Source Code:** Full syntax highlighting via `syntect` with line numbers and bounded memory reads.
- **Markdown:** Renders headers, lists, code fences, blockquotes, checklists, and auto-extracted Table of Contents.
- **Images:** 24-bit Truecolor half-block (`▀`) pixel rendering for PNG, JPEG, GIF, WebP, BMP, and dimension parsing for SVG.
- **Video Containers:** Extracts duration, resolution, codecs, bitrate via `ffprobe`, and first-frame thumbnails via `ffmpeg`.
- **Audio Media:** Inspects sample rate, duration, channels, ID3 tags (artist, title, album), and detects CLI audio players.
- **PDF Documents:** Pure-Rust catalog header parser (page count, title, author) & extracted text stream via `pdftotext`.
- **Pure-Rust Office Documents:** Direct XML parsing from ZIP packages for `.docx` paragraphs, `.xlsx` shared strings/cells, and `.pptx` slide counts (no LibreOffice required).
- **Tabular Data:** Delimiter sniffer for CSV/TSV with column alignments, and JSON pretty-printer with syntax error indicators.
- **Archives:** Nested index table for `.zip`, `.tar`, and `.tar.gz` with compression ratios and path traversal safety checks.
- **SQLite 3 Databases:** Header decoding (page size, version, change count), `sqlite_master` schema browser, and row sampler.
- **Canonical Hex Viewer:** 16-byte hex dump with memory offset, hex byte pairs, and printable ASCII pane.
- **LRU Cache:** Bounded in-memory preview cache (max 64 MB / 100 entries) preventing repeated disk reads.

### 2. ⚡ Dual-Pane Explorer & FreeDesktop Trash
- **Dual-Pane View (`F7`):** Side-by-side file managers with independent navigation, selections, and sorting.
- **Cross-Pane Transfers:** Copy files to the opposing pane with `c`, move with `m`.
- **FreeDesktop Trash (`d`):** Safely moves deleted items to FreeDesktop Trash (`gio trash` / `trash-put`) before permanent deletion (`D`).
- **Sidebar & Bookmarks (`b` / `p`):** Pin folders, access Linux shortcuts (`Home`, `Root`, `Downloads`, `/etc`), and manage workspaces.

### 3. 🎛️ Interactive Settings Studio (`F10`)
- **15 Configuration Sections:** General, Appearance, Keybindings, Explorer, Preview, Editor, Terminal, System Monitor, Git, Bookmarks, Profiles, Performance, Plugins, Advanced, About.
- **Keybinding Conflict Detection:** Scoped key action registry flags clashing shortcuts automatically.
- **6 Built-in Profiles:** `Default`, `Developer`, `Minimal`, `TerminalPowerUser`, `EditorFocus`, and `Monitoring`.
- **Resilient Persistence:** Configuration boundary clamping and atomic `.toml.bak` backup recovery.

### 4. 🎨 12 Built-in Themes & Transparency
- **Included Themes:** Catppuccin Mocha, Catppuccin Macchiato, Dracula, Nord, Gruvbox, Tokyo Night, One Dark, Solarized Dark, Solarized Light, Monokai, Default Dark, Default Light.
- **Terminal Transparency:** Seamless inheritance of your terminal emulator's background and blur settings (`bg: None`).

### 5. 💻 Embedded Interactive PTY Terminal
- Real pseudo-terminal powered by `portable-pty` spawning `$SHELL` (or Bash).
- Preserves native shell completions, aliases, prompts, and job control.
- Non-blocking channel I/O eliminates rendering locks.

### 6. ✏️ Lightweight Code Editor
- Auto-pairing for `( )`, `[ ]`, `{ }`, `" "`, `' '`, and `` ` ``.
- Multi-level Undo/Redo, word autocompletion popup, line numbers, and atomic writes via `tempfile`.

### 7. 📊 System Monitor & Git Dashboard
- **System Monitor (`m`):** Per-core CPU load, memory utilization, swap, disk capacity, and live top processes.
- **Git Dashboard (`g`):** Branch tracking, staged/unstaged views, recent commit history, and inline staging (`s`).
- **Command Palette (`Ctrl+Shift+P` / `P`):** Fuzzy-search internal actions, panel toggles, and bookmarks.
- **Onboarding Wizard (`F1`):** Interactive 5-step walkthrough on first launch.

---

## ⌨️ Essential Keyboard Shortcuts

| Shortcut | Context | Action |
|---|---|---|
| <kbd>Tab</kbd> / <kbd>Shift+Tab</kbd> | Global | Cycle focus between active panels |
| <kbd>Ctrl+Shift+P</kbd> / <kbd>P</kbd> | Global | Open fuzzy-search Command Palette |
| <kbd>F1</kbd> | Global | Open Onboarding Wizard / Help |
| <kbd>F7</kbd> | Global | Toggle Dual-Pane side-by-side explorer |
| <kbd>F10</kbd> | Global | Open Interactive Settings Studio |
| <kbd>`</kbd> (Backtick) | Global | Toggle embedded PTY terminal drawer |
| <kbd>m</kbd> | Global | Toggle system resource monitor |
| <kbd>g</kbd> | Global | Toggle Git dashboard overlay |
| <kbd>↓</kbd> / <kbd>↑</kbd> (or <kbd>j</kbd>/<kbd>k</kbd>) | Explorer | Move selection down / up in directory |
| <kbd>→</kbd> / <kbd>Enter</kbd> / <kbd>l</kbd> | Explorer | Navigate into directory (or preview file) |
| <kbd>←</kbd> / <kbd>Backspace</kbd> / <kbd>h</kbd> | Explorer | Navigate out to parent directory |
| <kbd>/</kbd> | Explorer | Real-time fuzzy filter files |
| <kbd>.</kbd> | Explorer | Toggle hidden dotfiles |
| <kbd>s</kbd> | Explorer | Cycle sort order (Name, Size, Modified, Type) |
| <kbd>a</kbd> / <kbd>A</kbd> | Explorer | Create new file / new directory |
| <kbd>r</kbd> | Explorer | Rename selected file or directory |
| <kbd>d</kbd> | Explorer | Move selected item to FreeDesktop Trash |
| <kbd>D</kbd> | Explorer | Permanently delete selected item |
| <kbd>c</kbd> / <kbd>m</kbd> | Explorer | Copy / Move item (or across panes in Dual-Pane) |
| <kbd>[</kbd> / <kbd>]</kbd> | Preview | Cycle preview tabs (`[Preview]`, `[Info]`, `[Hex]`, `[Actions]`) |
| <kbd>Ctrl+S</kbd> | Editor | Save buffer to disk atomically |
| <kbd>Esc</kbd> | Editor / Modal | Close modal / return focus to File Explorer |
| <kbd>q</kbd> | Global | Exit TermExplorer safely |

*For the complete keybinding matrix, see [KEYBINDINGS.md](docs/KEYBINDINGS.md).*

---

## 📦 Installation & Getting Started

### Prerequisites (Ubuntu / Linux)
```bash
sudo apt update
sudo apt install -y build-essential pkg-config libasound2-dev libxcb1-dev
```

### Optional Multimedia Preview Utilities
```bash
sudo apt install -y ffmpeg poppler-utils gio trash-cli
```

### Option 1: Install via Cargo
```bash
cargo install --git https://github.com/jyothiprajith1323/TermExplorer.git
```

### Option 2: Build from Source
```bash
git clone https://github.com/jyothiprajith1323/TermExplorer.git
cd TermExplorer
cargo build --release
```
The compiled binary will be placed at `target/release/termexplorer`. Move it into your `$PATH`:
```bash
cp target/release/termexplorer ~/.local/bin/
```

### Run
```bash
termexplorer
```

---

## 🧪 Verification & Testing

TermExplorer is tested thoroughly under strict systems safety criteria:
```bash
# Run all 42 automated unit and integration tests across 8 test suites
cargo test --all-targets --all-features

# Verify strict Clippy compliance (0 warnings allowed)
cargo clippy --all-targets --all-features -- -D warnings

# Check code formatting
cargo fmt --all -- --check
```

---

## 📚 Documentation & Specifications

- [Architecture & Design Guide](docs/ARCHITECTURE.md)
- [Complete Keybindings Reference](docs/KEYBINDINGS.md)
- [Developer & Contributing Guide](docs/DEVELOPMENT.md)
- [Release Changelog](CHANGELOG.md)
- [Contributing Guidelines](CONTRIBUTING.md)
- [Security Policy](SECURITY.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)

---

## 🛡️ License

This project is licensed under the [MIT License](LICENSE).
TermExplorer &copy; 2026 Jyothi Prajith and contributors.
