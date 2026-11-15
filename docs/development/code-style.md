# Code Style & Engineering Standards

TermExplorer adheres to strict systems-level Rust programming conventions.

---

## 1. Rustfmt & Formatting Standards

Code formatting is enforced automatically by `rustfmt.toml`:
- Edition: **2021**
- Max width: **100 characters**
- Tab spaces: **4 spaces** (no tabs)
- Imports: Grouped and sorted lexicographically

Always run `cargo fmt --all` before committing changes.

---

## 2. Clippy & Linter Standards

All targets must compile cleanly with `-D warnings`:
- No unused variables, imports, or dead code in release builds.
- Prefer explicit error types or `anyhow::Result` with contextual messages (`.context("...")`) over naked `.unwrap()` calls in production code.
- Avoid unnecessary cloning on hot paths; pass slices (`&str`, `&[u8]`, `&[T]`) where possible.
- Wrap numerical arithmetic on layout dimensions with `.saturating_add()` or `.saturating_sub()` to prevent debug mode panics on integer overflow.

---

## 3. Error Handling Principles

- **Library Modules (`src/editor`, `src/filesystem`, `src/preview`)**: Return strongly-typed errors or `Result<T, SubsystemError>`. Do not panic.
- **TUI & Dispatch Layer (`src/app.rs`, `src/ui/`)**: Handle non-fatal errors gracefully by setting an ephemeral status bar error notification (`StatusKind::Error`) rather than aborting the session.
- **Atomic File Operations**: Always write to a `.tmp` file, flush buffers with `sync_all()`, and rename atomically.
