# TermExplorer Development Guide

This guide covers building, running, testing, and developing TermExplorer on Ubuntu Linux and compatible Unix environments.

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

---

## 2. Repository Structure

```text
termexplorer/
├── Cargo.toml          # Package configuration & dependencies
├── rustfmt.toml        # Formatting rules
├── src/
│   ├── main.rs         # Application entry point & terminal restoration
│   ├── lib.rs          # Exported library interface
│   ├── app.rs          # Core event loop, state machine & input routing
│   ├── config.rs       # TOML configuration loading & persistence
│   ├── state.rs        # Centralized application state
│   ├── events.rs       # Crossterm & async event streaming
│   ├── theme.rs        # Transparent styling & color palettes
│   ├── filesystem/     # Directory listing, operations, metadata, search & watcher
│   ├── editor/         # Buffer, auto-pairing, autocompletion, syntect syntax & LSP
│   ├── shell/          # PTY spawning, session management & key translation
│   ├── monitor/        # CPU, memory, disk, and process telemetry
│   └── ui/             # Ratatui UI components & layouts
├── tests/              # Comprehensive test suites
│   ├── filesystem_tests.rs
│   ├── editor_tests.rs
│   ├── config_tests.rs
│   ├── monitor_tests.rs
│   └── terminal_tests.rs
└── docs/               # Technical documentation
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

### Run TermExplorer:
```bash
cargo run
```

### Optimized Release Build:
```bash
cargo build --release
```
The compiled binary will be placed at `target/release/termexplorer`.

### Install to User Bin:
```bash
cargo install --path .
```
Now `termexplorer` can be executed directly from anywhere in your terminal.

---

## 4. Quality Verification & Testing

Always verify all code quality checks before submitting changes:

### 1. Formatting Check:
```bash
cargo fmt --all -- --check
```

### 2. Compilation & Target Check:
```bash
cargo check --all-targets
```

### 3. Automated Test Suite:
```bash
cargo test
```

### 4. Clippy Linter Check:
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

---

## 5. Adding New Features

- **New Shortcuts or Keys:** Define key routing in `src/app.rs` under `handle_key` or panel-specific handlers.
- **New Syntax Highlight Languages:** `syntect` includes default Sublime packages. Add mappings in `src/editor/syntax.rs` if needed.
- **New System Metrics:** Extend `src/monitor/` and add fields to `MonitorSnapshot`.
- **UI Customization:** Adjust themes in `src/theme.rs` or components under `src/ui/`.
