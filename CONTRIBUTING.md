# Contributing to TermExplorer

Thank you for your interest in contributing to **TermExplorer**! We welcome contributions of all kinds: bug fixes, new preview format handlers, UI improvements, documentation, and feature requests.

---

## Code of Conduct

This project adheres to the Contributor Covenant Code of Conduct. By participating, you are expected to uphold this code. Please read [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for details.

---

## Development Setup

### Prerequisites

- **Rust toolchain:** 1.80+ (stable). Install via [rustup.rs](https://rustup.rs):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **Linux Packages (Ubuntu/Debian):**
  ```bash
  sudo apt update
  sudo apt install -y build-essential pkg-config libasound2-dev libxcb1-dev
  ```
- **Optional CLI utilities for preview enhancements:**
  ```bash
  sudo apt install -y ffmpeg poppler-utils gio libtrash-dev
  ```

### Building & Running

1. **Clone the repository:**
   ```bash
   git clone https://github.com/jyothiprajith1323/TermExplorer.git
   cd TermExplorer
   ```

2. **Run in development mode:**
   ```bash
   cargo run
   ```

3. **Run the full test suite:**
   ```bash
   cargo test --all-targets --all-features
   ```

4. **Lint and format checks:**
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   ```

---

## Coding Guidelines

We enforce strict Rust systems programming principles:

1. **No panics in production code:** Avoid `.unwrap()` and `.expect()` in non-test runtime paths. Use `anyhow::Result` or typed `thiserror` variants.
2. **Clippy clean:** All PRs must compile with zero Clippy warnings under `-D warnings`.
3. **Format compliance:** Always run `cargo fmt` before submitting.
4. **Transparent Terminal Background:** Never hardcode black or white backgrounds unless explicitly chosen; respect `bg: None` for terminal transparency inheritance.
5. **Bounded Resource Usage:** Buffers, caches, and file reads must have explicit bounds (e.g., maximum preview file size limits).

---

## Pull Request Process

1. Fork the repository and create your branch from `master`:
   ```bash
   git checkout -b feature/my-amazing-feature
   ```
2. Write unit/integration tests for your changes in `tests/`.
3. Ensure all tests and lints pass:
   ```bash
   cargo test
   cargo clippy --all-targets --all-features -- -D warnings
   ```
4. Update relevant documentation in `docs/` and `README.md` if modifying keybindings, preview formats, or configuration options.
5. Submit a pull request with a clear description of the problem and solution.
