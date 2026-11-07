# TermExplorer Development Guide

This guide covers building, running, testing, and developing **TermExplorer** on Ubuntu Linux and compatible Unix environments.

---

## 1. Prerequisites

- **OS:** Ubuntu Linux 22.04 LTS / 24.04 LTS or any modern Linux distribution / WSL2.
- **Rust Toolchain:** Stable Rust 1.80+ (recommended Rust 1.85+ or latest stable).
- **System Libraries:** Standard C library and POSIX pseudo-terminal support (included with Ubuntu).

Install Rust via rustup if not already present:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env
```

Install optional multimedia tools for expanded preview support:
```bash
sudo apt update && sudo apt install -y build-essential pkg-config ffmpeg poppler-utils gio trash-cli
```

---

## 2. Repository Structure

```text
termexplorer/
├── .github/
│   ├── workflows/          # CI, GitHub Pages & Release workflows
│   │   ├── ci.yml
│   │   ├── pages.yml
│   │   └── release.yml
│   ├── ISSUE_TEMPLATE/     # Bug report & feature request templates
│   └── pull_request_template.md
├── website/                # GitHub Pages static documentation website
│   ├── index.html
│   ├── features.html
│   ├── docs.html
│   ├── architecture.html
│   ├── roadmap.html
│   └── assets/ (style.css, app.js)
├── src/
│   ├── main.rs             # Application entry point & terminal restoration
│   ├── lib.rs              # Exported library interface
│   ├── app.rs              # Core event loop, state machine & input routing
│   ├── config.rs           # TOML configuration loading & persistence
│   ├── state.rs            # Centralized application state
│   ├── events.rs           # Crossterm & async event streaming
│   ├── theme.rs            # 12 themes & transparent styling
│   ├── filesystem/         # Directory listing, FreeDesktop trash, operations, metadata
│   ├── preview/            # Universal multi-format preview engine
│   │   ├── detector.rs     # Magic bytes format classifier
│   │   ├── registry.rs     # CLI tool & capability detector
│   │   ├── rendering.rs    # Half-block Truecolor renderer
│   │   ├── cache.rs        # Bounded LRU cache
│   │   └── [format handlers: source, markdown, image, video, audio, pdf, office, data, archive, database, hex]
│   ├── settings/           # Settings Studio engine
│   │   ├── schema.rs       # 15 typed settings sections
│   │   ├── keybindings.rs  # Scoped action registry & conflict detector
│   │   ├── profiles.rs     # 6 layout presets
│   │   └── persistence.rs  # Validation, clamping & .toml.bak backups
│   ├── commands/           # Command Palette engine & registry
│   ├── git/                # Git CLI status & commit tracker
│   ├── workspace/          # Project kind detector & actions
│   ├── editor/             # Buffer, auto-pairing, autocompletion & syntect syntax
│   ├── shell/              # PTY spawning, session management & key translation
│   ├── monitor/            # CPU, memory, disk, and process telemetry
│   └── ui/                 # Ratatui UI components & modal overlays
├── tests/                  # 8 integration test suites (42 tests)
│   ├── config_tests.rs
│   ├── editor_tests.rs
│   ├── filesystem_tests.rs
│   ├── git_and_workspace_tests.rs
│   ├── monitor_tests.rs
│   ├── preview_tests.rs
│   ├── settings_tests.rs
│   └── terminal_tests.rs
└── docs/                   # Architectural & user manuals
    ├── ARCHITECTURE.md
    ├── KEYBINDINGS.md
    └── DEVELOPMENT.md
```

---

## 3. Build & Run Commands

### Development Build:
```bash
cargo build
```

### Run Locally:
```bash
cargo run
```

### Run with Debug Logging:
```bash
RUST_LOG=debug cargo run
```

### Production Release Build:
```bash
cargo build --release
```

---

## 4. Testing & Verification

TermExplorer enforces strict test coverage and zero Clippy warnings:

```bash
# Run all 42 automated tests across 8 suites
cargo test --all-targets --all-features

# Check formatting compliance
cargo fmt --all -- --check

# Deny all Clippy warnings
cargo clippy --all-targets --all-features -- -D warnings
```
