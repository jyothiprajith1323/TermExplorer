# TERMEXPLORER — SPRINT 4 PREREQUISITE ARCHITECTURAL AUDIT

**Milestones Under Scope:** Sprint 4A (Embedded PTY Terminal Engine) & Sprint 4B (Native Git Integration)  
**Date:** November 14, 2026  
**Auditor:** Principal Systems Engineer & Terminal Architect  
**Commit Inspected:** `06f7e32` (`master`)  
**Git Environment:** `git version 2.53.0` on Linux WSL2 (`Standard-WSL2`, Ubuntu 24.04 LTS, x86_64)  
**CI Status:** 100% Green across all GitHub Actions workflows (`Continuous Integration`, `Security Scanning`, `Deploy GitHub Pages`)  

---

## 1. Executive Assessment & Working-Tree Status

### 1.1 Git Topology
- **Current Branch:** `master`
- **Working Tree:** Clean (zero untracked files, zero unstaged changes, zero merge conflicts).
- **Remote Synchronization:** Up to date with `origin/master`.
- **Sprint 3 Verification Status:** All 5 phases (3A Universal Preview Foundation, 3B Structured Data, 3C Media Engine, 3D Virtual Archive & Hex, 3E Unified Search) passed 100% of quality gates and tests. Zero P0/P1 defects remain in the codebase.

---

## 2. Sprint 4A: PTY & Terminal Subsystem Audit

### 2.1 Current Implementation State
- **Backend:** `portable-pty = "0.8"` allocated in `src/shell/pty.rs`. Spawns `$SHELL` (or `/bin/bash`) with a dedicated background thread reading from `master.try_clone_reader()`.
- **Session Layer:** `src/shell/session.rs` wraps `PtySession`, maintains `scrollback: VecDeque<u8>`, and parses OSC 7 directory notifications (`\x1b]7;file://...`).
- **Input Routing:** `src/shell/input.rs` translates `crossterm::KeyEvent` into raw terminal escape sequences.
- **Rendering:** `src/ui/terminal.rs` implements `render_terminal`.

### 2.2 Critical Limitations of Current Terminal Engine
1. **Lack of Virtual Terminal Screen Emulation:**
   - Currently, `process_terminal_output` in `src/ui/terminal.rs` uses a simple byte-by-byte loop that strips ANSI escape sequences and produces plain text strings.
   - **Consequence:** Full-screen terminal programs (`htop`, `vim`, `nano`, `less`), cursor-movement sequences (`\x1b[H`, `\x1b[2J`), and alternate screen buffers (`DECSET 1049`) produce visual corruption or fail to display.
2. **Missing Text Attributes & Color Depth:**
   - 24-bit Truecolor, 256-color palettes, ANSI colors, bold, italic, underline, and inverted colors are currently discarded.
3. **Frame-by-Frame O(N) Re-parsing:**
   - The entire raw scrollback (up to 1 MB) is re-parsed on every frame in the UI thread, causing unnecessary CPU cycles during continuous rendering.
4. **Single-Session Constraint:**
   - Only a single terminal session is supported. No multi-session manager, session tabs, or session switching exists.
5. **No Scrollback Search:**
   - Users cannot search within historical terminal output.
6. **Process Lifecycle & Exit Handling:**
   - Child processes are only reaped on `Drop`. If a shell exits unexpectedly or with an error, the terminal does not display exit codes or allow a clean session restart.

### 2.3 Sprint 4A Architectural Upgrades
- **Screen Emulation with `vt100`:** Integrate `vt100` (v0.18.1, MIT license). Maintain a virtual screen buffer incrementally updated as PTY bytes arrive. Render directly to Ratatui cells with full RGB truecolor, 256 colors, bold/dim/italic/underline attributes, and cursor tracking.
- **Multi-Session Manager (`TerminalSessionManager`):** Manage multiple active sessions, session tabs (`[1: bash]`, `[2: zsh]`), session creation (`[+]`), closing (`[×]`), and restart upon exit.
- **Dynamic Resizing:** Propagate `master.resize(cols, rows)` and `vt100::Screen::set_size(rows, cols)` synchronously when the drawer height or window size changes.
- **Bounded Scrollback & History Search:** Configurable history limit (10,000 lines) with interactive in-buffer searching.

---

## 3. Sprint 4B: Git Integration Subsystem Audit

### 3.1 Current Implementation State
- **Backend:** `src/git/mod.rs` provides `GitRepoStatus::query()` which executes synchronous `git status --porcelain=v1` and `git log -n 5`.
- **UI:** `src/ui/git_panel.rs` renders a centered modal dialog displaying staged files, unstaged files, and 5 recent commits.
- **Actions:** Basic file staging (`git add`), unstaging (`git restore --staged`), and commit (`git commit -m`).

### 3.2 Critical Limitations of Current Git Implementation
1. **Synchronous Subprocess Blocking:**
   - `Command::new("git").output()` is invoked directly on the UI thread. In large repositories (100,000+ files) or slow filesystems (WSL 9P mounts), status checks block the 60 FPS event loop.
2. **Fragile Porcelain v1 Parsing:**
   - Uses `status --porcelain=v1` line splitting. Filenames containing spaces, quotes, newlines, or Unicode characters can be misparsed.
   - Does not parse ahead/behind counts, upstream tracking branches, or detailed conflict states.
3. **Missing Branch Management:**
   - No branch listing, branch creation, branch switching (`git switch`), or local branch deletion.
4. **No Native Diff Viewer:**
   - Users cannot inspect working-tree or index diffs before staging or committing.
5. **No Blame or File History:**
   - No file-level commit history or `git blame` inspection.
6. **Unsaved Editor Buffer Conflict:**
   - Staging does not verify whether the active file has unsaved in-memory changes in the editor buffer, risking out-of-sync commits.

### 3.3 Sprint 4B Architectural Upgrades
- **Asynchronous Git Operation Manager (`GitOperationManager`):** Execute all Git subprocesses asynchronously off the UI thread with timeouts and cancellation.
- **Porcelain v2 Parsing:** Use `git status --porcelain=v2 --branch -z` with NUL delimiters to guarantee 100% path safety for any filename character.
- **Native Diff Viewer:** Integrated diff inspector in the 75% preview pane with Catppuccin Mocha styling for added, deleted, and modified lines, with hunk navigation.
- **Branch Management:** List branches, display upstream status, switch branches safely with uncommitted-change collision checks.
- **Commit History & Blame:** Paginated commit log with commit diffs, plus file-specific blame view.
- **Editor Synchronization:** Detect dirty editor buffers before staging/committing with clear prompts.

---

## 4. Subprocess Execution & WSL2 Compatibility

- **Git Version:** `git version 2.53.0` installed in environment. Fully supports porcelain v2, worktrees, and modern switch/restore semantics.
- **Unix PTY & Shells:** `/bin/bash` verified present and accessible via `portable-pty`.
- **Safety Policy:**
  - All commands invoke explicit argument vectors (`.args([...])`); zero shell interpolation (`sh -c`).
  - Child processes tracked and killed/reaped on drop or application shutdown.
  - Zero destructive Git operations (e.g., hard checkout, stash drop) without explicit user confirmation.

---

## 5. Phased Implementation Roadmap

### Phase 1: Sprint 4A — Virtual Terminal Engine (`vt100`)
1. Create `src/shell/terminal_screen.rs`: Wrap `vt100::Parser` for incremental stream parsing and cell-by-cell Ratatui rendering.
2. Create `src/shell/manager.rs`: Implement `TerminalSessionManager` for multi-session lifecycle, session tabs, and process exit tracking.
3. Update `src/ui/terminal.rs`: Render truecolor/ANSI cells, cursor position, and interactive session tabs.
4. Add unit and integration tests in `tests/terminal_tests.rs`.

### Phase 2: Sprint 4B — Asynchronous Git Engine
1. Refactor `src/git/`: Modularize into `src/git/mod.rs`, `src/git/status.rs`, `src/git/branch.rs`, `src/git/diff.rs`, `src/git/history.rs`, `src/git/manager.rs`.
2. Implement porcelain v2 NUL-delimited status parser.
3. Implement native diff parser and Catppuccin Mocha syntax renderer.
4. Implement branch listing, switching, and safety guards against dirty buffer overwrites.
5. Update `src/ui/git_panel.rs` and preview pane integration.
6. Add unit and integration tests in `tests/git_tests.rs`.

### Phase 3: Quality Gates & Verification
1. Run `cargo fmt`, `cargo check`, `cargo test`, `cargo clippy`, `cargo build --release`, and `cargo doc`.
2. Verify GitHub Actions CI pipelines.
