# Project Structure & Repository Layout

This document provides a tour of the TermExplorer codebase and module boundaries.

---

## 1. Top-Level Directory Tour

```text
TermExplorer/
├── Cargo.toml               # Crate manifest, dependency definitions, features
├── Cargo.lock               # Exact pinned dependency lockfile
├── README.md                # Repository landing experience
├── CONTRIBUTING.md          # Contributor guide and pull request workflow
├── SECURITY.md              # Security policy and vulnerability disclosure
├── LICENSE                  # MIT open-source license
├── CHANGELOG.md             # Release history and sprint milestones
├── rustfmt.toml             # Project-wide code formatting rules
├── deny.toml                # cargo-deny license and advisory configuration
│
├── src/                     # Core Rust application source code
├── tests/                   # 26 comprehensive integration test suites
├── plugins/                 # Reference WebAssembly plugin crates & binaries
├── packaging/               # Debian control files, desktop entry, manpage, icons
├── scripts/                 # Packaging, build automation, and checksum scripts
├── docs/                    # Technical documentation portal
└── screenshots/             # Workstation terminal capture recordings
```

---

## 2. Core Source Modules (`src/`)

- **`main.rs`**: Application entry point, CLI arguments parsing (`--help`, `--version`, `--health`, `--cwd-file`), daily rotating tracing setup, panic restoration hook, and terminal setup.
- **`app.rs`**: Central `App` struct coordinating state, the main event loop, input routing, and view transitions.
- **`state.rs`**: Operational state models, panel focus enumerations, active selection, and status bar notices.
- **`config/`**: Configuration management:
  - `mod.rs`: Strongly typed configuration tables, clamping ranges, atomic save, and corrupted file recovery.
  - `first_run.rs`: First-run detection (`FirstRunStatus`), state recording in `first_run.json`.
  - `migration.rs`: Version gating and automatic schema v1 to v2 migration.
- **`ui/`**: Ratatui rendering components:
  - `file_explorer.rs`, `preview.rs`, `editor.rs`, `terminal.rs`, `git_panel.rs`, `settings.rs`, `onboarding.rs`, `command_palette.rs`, `system_monitor.rs`, `task_viewer.rs`, `plugin_center.rs`, `help.rs`, `diagnostics.rs`, `status_bar.rs`, `header.rs`, `sidebar.rs`.
- **`editor/`**: Integrated editor:
  - `buffer.rs`: `ropey::Rope` text buffer, cursor movement, selection, transactional undo/redo.
  - `auto_pair.rs`: Intelligent bracket and quote pairing.
  - `completion.rs`: 4-layer autocomplete engine.
  - `syntax.rs`: `syntect` syntax tokenization and scope caching.
  - `lsp.rs`: Language Server Protocol client.
- **`filesystem/`**: Filesystem interactions:
  - `browser.rs`: Directory scanning, sorting, entry classification.
  - `chunked.rs`: Chunked viewport loader for large directories.
  - `operations.rs`: File creation, atomic saving, collision resolution.
  - `ledger.rs`: Mutation logging and FreeDesktop trash integration.
- **`preview/`**: Universal multi-format preview engine:
  - `detector.rs`: File format detection via magic bytes and MIME sniffer.
  - `registry.rs`: Preview capability registry.
  - `cache.rs`: Multi-tier LRU memory cache and disk cache.
  - Format handlers: `source.rs`, `markdown.rs`, `image.rs`, `data.rs`, `database.rs`, `pdf.rs`, `audio.rs`, `video.rs`, `office.rs`, `archive.rs`, `hex.rs`.
- **`shell/`**: Terminal drawer and PTY:
  - `detection.rs`: Hardware capabilities, shell discovery, CLI tool integrations.
  - `pty.rs`: `portable-pty` session spawning.
  - `terminal_screen.rs`: In-memory screen cell buffer.
- **`git/`**: Git integration:
  - `status.rs`, `diff.rs`, `branch.rs`, `commit.rs`, `history.rs`, `manager.rs`.
- **`workflow/`**: Developer workflows:
  - `detect.rs`: Project tech stack detection.
  - `dag.rs`: Task DAG dependency resolution.
  - `runner.rs`: Process execution and line streaming.
  - `trust.rs`: Workspace trust store.
- **`plugins/`**: WebAssembly extensibility:
  - `sandbox.rs`: `wasmi` execution, fuel metering, memory limits.
  - `manifest.rs`: `plugin.toml` parser and validator.
  - `permissions.rs`: Capability permissions model.
