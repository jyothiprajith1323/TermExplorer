# AGENTS.md — AI Agent Operating Guidelines & Repository Manual

Welcome to **TermExplorer**. This document is the primary reference manual and operational specification for autonomous, semi-autonomous, and assistant AI agents (e.g., Devin, Claude, Copilot, Antigravity, SWE-bench agents) working in this repository.

---

## 1. Repository Identity & Architectural Overview

**TermExplorer** is a modern, high-performance Linux terminal developer workstation written in idiomatic Rust (Rust 2021 edition). It combines a fast file explorer, an intelligent code editor, an interactive Linux pseudo-terminal (PTY) drawer, a multi-format multimedia preview engine, and a real-time system performance telemetry monitor.

### Key Technology Stack
- **Language / Edition:** Rust 1.80+ (2021 Edition).
- **TUI & Rendering:** [Ratatui](https://crates.io/crates/ratatui) + [Crossterm](https://crates.io/crates/crossterm).
- **Embedded Terminal:** [portable-pty](https://crates.io/crates/portable-pty) (spawns `/dev/ptmx` Linux pseudo-terminals with full ANSI/xterm escape handling).
- **Syntax Engine:** [syntect](https://crates.io/crates/syntect) (Sublime text syntax definitions and themes with bounded caching).
- **Sandboxed Plugins:** [wasmi](https://crates.io/crates/wasmi) (WebAssembly interpreter with strict memory and fuel boundaries).
- **Hardware Telemetry:** [sysinfo](https://crates.io/crates/sysinfo) (per-core CPU, RAM, Swap, disk I/O, process table).
- **Error Handling:** [anyhow](https://crates.io/crates/anyhow) and [thiserror](https://crates.io/crates/thiserror).

### Directory & Subsystem Map
```text
termexplorer/
├── src/
│   ├── main.rs                 # CLI entry point, signal handlers & terminal cleanup
│   ├── lib.rs                  # Exported library interfaces & integration endpoints
│   ├── app.rs                  # Core workstation state machine & input dispatcher
│   ├── state.rs                # Application state, pane focus & view modes
│   ├── config.rs               # TOML configuration loading, schema & persistence
│   ├── events.rs               # Async crossterm and background event streamer
│   ├── theme.rs                # 12 themes (Catppuccin Mocha/Latte, Tokyo Night, etc.)
│   ├── filesystem/             # Directory walker, FreeDesktop trash, operations
│   ├── preview/                # Universal preview engine with 11 format handlers
│   │   ├── detector.rs         # Magic byte signature sniffing
│   │   ├── rendering.rs        # Half-block Truecolor ANSI graphic renderer
│   │   ├── cache.rs            # Thread-safe LRU preview cache
│   │   └── handlers/           # Source, Markdown, Media, PDF, DB, Hex, Archive
│   ├── settings/               # Settings Studio engine (schema, keybindings, profiles)
│   ├── commands/               # Command Palette (fuzzy finder, actions)
│   ├── git/                    # Git repository status & commit tracker
│   ├── workspace/              # Project detector & workspace actions
│   ├── editor/                 # Modal editor, auto-pairing, autocompletion
│   ├── shell/                  # PTY drawer, session manager, terminal buffer
│   ├── monitor/                # System telemetry (CPU, RAM, swap, disks, processes)
│   └── ui/                     # Ratatui widgets, workstation views & modal overlays
├── tests/                      # 38+ integration, unit, and property test suites
│   ├── packaging_tests.sh      # Package acceptance and verification suite
│   └── fixtures/               # Test files, images, archives, and mock scripts
├── packaging/                  # Package specs: Debian (DEBIAN/control), RPM (.spec), install scripts
├── scripts/                    # Automation: package_linux.sh, test_package_linux.sh, repos
├── website/                    # GitHub Pages documentation site (HTML/CSS/JS)
└── docs/                       # Comprehensive architectural & user manuals
```

---

## 2. Absolute Execution Policy for AI Agents

When operating within this repository, all AI agents **MUST** strictly adhere to the following guardrails:

1. **Work Locally Only**:
   - Never run `git push`, `git push --force`, or push tags to remote repositories.
   - Never create remote branches or open pull requests unless explicitly commanded by the user.
   - Never publish packages, tags, or deployment artifacts to production registries or GitHub Releases.
2. **Preserve Repository Integrity**:
   - Never run destructive Git commands such as `git reset --hard`, `git clean -fd`, or `git stash drop`.
   - Always preserve uncommitted user changes, git stashes, and scratch files.
   - Make surgical, minimal edits focused directly on the prompt requirements.
3. **Evidence-Based Reporting**:
   - Never declare or report a test passed, a build succeeded, or a package installed unless you actually executed the command and inspected its output in the environment.
   - Never fabricate ABI compatibility claims or CI passes.
   - Never use mocked packaging tools as evidence of a real package build.
4. **Link Files Correctly**:
   - Whenever referencing local files in answers or reports, use clickable Markdown links with the `file://` scheme (e.g., `[main.rs](file:///home/prajith/TermExplorer/src/main.rs)`).

---

## 3. Standard Build, Test & Quality Commands

Always run commands from the repository root:

### Core Rust Quality Gates
```bash
# 1. Format Check
cargo fmt --all -- --check

# 2. Type & Dependency Check (Locked)
cargo check --locked --all-targets --all-features

# 3. Complete Test Suite
cargo test --locked --all-targets

# 4. Strict Clippy Linter
cargo clippy --all-targets --all-features -- -D warnings

# 5. Build Optimized Release Binary
cargo build --release
```

### Application Runtime & Health Checks
```bash
# Run non-interactive diagnostic health check
./target/release/termexplorer --health

# Run interactive workstation locally
./target/release/termexplorer

# Run against a specific directory
./target/release/termexplorer /path/to/folder
```

### Packaging & Distribution Verification
```bash
# Build all Linux packages (.deb, .rpm, .tar.gz, SHA-256 manifests)
./scripts/package_linux.sh --all

# Run complete local packaging acceptance suite (56 tests)
bash tests/packaging_tests.sh
bash scripts/test_package_linux.sh 1.1.0 dist
```

### Website Quality Gates
```bash
cd website
npm ci
npm test
npm run test:lighthouse
cd ..
python3 scripts/validate_website.py
```

---

## 4. Engineering & Rust Coding Standards

Any code written or modified by an agent must satisfy these systems-level engineering standards:

### A. Zero Unhandled Panics
- **Forbidden:** Never use `.unwrap()` or `.expect()` in non-test runtime code (`src/`).
- **Required:** Use `Result<T, anyhow::Error>` with contextual messages (`.with_context(|| "...")`), or explicit domain error enums via `thiserror`.
- **UI Non-Fatal Notifications:** Handle runtime failures (e.g., file permission denied, locked socket, unreadable directory) gracefully by logging and routing user-facing notifications to the status bar (`StatusKind::Error`) rather than aborting the process.

### B. Non-Blocking Async Main Loop
- The Ratatui rendering loop and event dispatcher (`app.rs`) must remain strictly non-blocking (aiming for 60 FPS refresh capability).
- File operations, directory scans, search indexing, and syntax tokenization must occur on background worker threads (`tokio` or `rayon`).
- **Monotonic Cancellation Tokens:** Long-running background operations must take an `Arc<AtomicU64>` generation token. If the user moves to another directory or changes selection, the token increments; background workers must check the token and immediately abort stale work.

### C. Safe Filesystem Operations
- **Atomic Persistence:** When saving files or configuration, never truncate or overwrite directly. Write to a temporary sibling file (`.filename.tmp.<pid>`), flush with `sync_all()`, and atomically rename (`std::fs::rename`) into place.
- **FreeDesktop Trash Protocol:** File deletion operations must default to FreeDesktop Trash (`gio trash` / `trash-cli`) with an explicit confirmation dialog before invoking any destructive unlinks.

### D. Terminal Correctness & Unicode Safety
- **Display Width Accuracy:** Always compute terminal column width using `unicode_width::UnicodeWidthStr` rather than byte or character length (`.len()` / `.chars().count()`). Properly handle combining marks, emojis, fullwidth CJK, and zero-width characters.
- **Escape Sequence Sanitization:** Every string rendered in preview panes or terminal drawers must pass through `sanitize_preview_line()` to strip hazardous ANSI escape sequences, OSC commands, or cursor repositioning codes.

### E. WebAssembly Plugin Sandboxing
- Wasm plugins executed via `wasmi` must run in a default-deny sandbox.
- Hard limits: 16 MiB maximum memory ceiling, 5,000,000 fuel consumption limit.
- Plugins must never be given direct access to the host filesystem, environment variables, or process spawning facilities.

### F. ABI Baseline & Packaging Compatibility
- The release binary links dynamically against GLIBC, requiring `hypotf@GLIBC_2.35`.
- Supported baseline: **Ubuntu 22.04 LTS (GLIBC 2.35)** and **Debian 12 Bookworm (GLIBC 2.36)**.
- Debian packaging metadata (`packaging/debian/DEBIAN/control`) must declare:
  ```deb
  Depends: libc6 (>= 2.35), libgcc-s1
  ```
- RPM packaging metadata (`packaging/rpm/termexplorer.spec`) must declare:
  ```spec
  Requires: glibc >= 2.35, libgcc
  ```

---

## 5. Agent Workflow & Troubleshooting Checklist

When given a task in TermExplorer:

1. **Inspect Git & Context First**:
   - Check status: `git status --short --branch`
   - Check recent commits: `git log -3 --oneline`
2. **Reproduce & Isolate**:
   - Write or identify a reproducing test case in `tests/` before modifying code.
3. **Execute Minimal Edits**:
   - Make minimal, clean edits in `src/` or `packaging/`.
   - Never reformat unrelated files or apply mass whitespace changes.
4. **Verify Locally with Real Tools**:
   - Run `cargo fmt --all -- --check`
   - Run `cargo clippy --all-targets --all-features -- -D warnings`
   - Run `cargo test --locked --test <test_name>`
5. **Clean Workspace Guarantee**:
   - Ensure no leftover temporary artifacts, debug prints, or untracked test files pollute the working tree.
