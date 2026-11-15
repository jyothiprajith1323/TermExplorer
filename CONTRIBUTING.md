# Contributing to TermExplorer

Thank you for your interest in contributing to **TermExplorer**! We welcome contributions from developers of all experience levels: bug fixes, preview format enhancements, performance optimizations, documentation improvements, and test coverage.

---

## 1. Code of Conduct

This project adheres to the Contributor Covenant Code of Conduct. By participating, you are expected to uphold this code. Please review [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for details.

---

## 2. Contribution Workflow

Follow this step-by-step workflow for all contributions:

1. **Find or Describe the Issue**: Check existing GitHub Issues or open a new issue describing the bug or feature request.
2. **Understand the Relevant Subsystem**: Consult the [Architecture Documentation](docs/architecture/overview.md) and module headers before making changes.
3. **Reproduce & Design**: Reproduce the behavior locally with a minimal test case.
4. **Implement Minimally**: Keep changes focused on the specific problem. Do not combine unrelated refactorings with feature work.
5. **Add or Update Tests**: Add regression tests in `tests/` validating your fix or new functionality.
6. **Run Quality Checks**:
   ```bash
   cargo fmt --all -- --check
   cargo check --locked --all-targets
   cargo test --all-targets
   cargo clippy --all-targets --all-features -- -D warnings
   cargo doc --no-deps
   ```
7. **Update Documentation**: Update corresponding guides in `docs/` and `README.md` if shortcuts, configuration keys, or behaviors change.
8. **Submit a Pull Request**: Submit a clear, reviewable pull request referencing the relevant issue.

---

## 3. Engineering & Quality Standards

TermExplorer enforces strict systems-level Rust standards:

### A. Reliability & Error Handling
- **No Panics**: Never call `.unwrap()` or `.expect()` in non-test runtime code. Handle errors gracefully using `Result` and `anyhow::Context`.
- **UI Non-Fatal Notifications**: Errors occurring during user operations (e.g. failing to open a locked file) must be communicated via the status bar (`StatusKind::Error`) rather than crashing the workstation.

### B. Async & Cancellation
- **Non-Blocking Main Loop**: The terminal UI loop must never block on synchronous I/O or child processes.
- **Monotonic Cancellation Tokens**: Long-running background jobs (directory walks, search, media conversions) must monitor generation tokens (`Arc<AtomicU64>`) and yield immediately when superseded.

### C. Filesystem Safety
- **Atomic Persistence**: Files must be written to temporary sibling files (`.filename.tmp.<pid>`), flushed with `sync_all()`, and renamed atomically.
- **Trash Over Deletion**: Deletion operations default to FreeDesktop Trash (`gio trash` / `trash-cli`) with explicit user confirmation dialogs.

### D. Unicode & Terminal Correctness
- **Display Width Accuracy**: Use `unicode_width` to determine screen column occupancy, handling multi-byte UTF-8, zero-width characters, combining accents, and fullwidth CJK glyphs correctly.
- **Terminal Sanitization**: All file content preview strings must pass through `sanitize_preview_line()` to strip dangerous escape sequences before terminal emission.

### E. WebAssembly Plugin Security
- **Default-Deny Capabilities**: Plugins run in an isolated `wasmi` sandbox with 16 MiB memory ceilings and 5M fuel limits. Never expose host filesystem or command execution APIs directly to guest modules.

### F. Resource Limits & Bounded Buffers
- All memory caches, search match lists, scrollback buffers, and directory chunk loaders must have hard, configurable ceilings.

---

## 4. How to Report Bugs

When reporting bugs, please provide:
1. TermExplorer version (`termexplorer --version`).
2. Diagnostic health check output (`termexplorer --health`).
3. Host operating system, Linux distribution, and terminal emulator (e.g. Ubuntu 22.04, Ghostty).
4. Steps to reproduce the issue.
5. Relevant log excerpts from `~/.local/share/termexplorer/termexplorer.log`.
