# TermExplorer — Sprint 5B: WebAssembly Plugin Sandbox & Extensibility Architecture
## Comprehensive Pre-Implementation Architecture Audit & Security Review

**Milestone:** Sprint 5B — WebAssembly Plugin Sandbox & Extensibility Architecture  
**Role:** Principal Rust Systems Architect, WebAssembly Runtime Engineer & Application Security Specialist  
**Date:** 2026-11-14  
**Workspace:** `/home/prajith/TermExplorer`  
**Toolchain:** `rustc 1.99.1`, `cargo 1.99.1`, `x86_64-unknown-linux-gnu`  
**Installed Wasm Targets:** `wasm32-wasip2`, `wasm32-wasip1`, `wasm32-unknown-unknown`  
**Target WebAssembly Runtime:** `wasmtime v50.0.1` with WebAssembly Component Model & WIT tooling  

---

## 1. Executive Summary

This architecture audit evaluates the TermExplorer codebase prior to commencing Sprint 5B (WebAssembly Plugin Sandbox & Extensibility Architecture). The objective is to design and implement a secure, high-performance, and isolated extension platform that allows developers to write plugins in WebAssembly (compiled via the Component Model) without granting extensions ambient access to the host operating system, filesystem, processes, terminal, or network.

### Key Audit Findings:
1. **Repository Health:** The repository is clean and stable. All 36 library unit tests pass (0.56s), all 23 Sprint 4 integration tests pass, zero compiler errors/warnings exist under `--locked`, and zero Clippy warnings exist under `-D warnings`.
2. **Runtime Upgrade:** The existing minimal prototype in `src/plugins/sandbox.rs` utilized `wasmi` (an interpreted, core-Wasm-only stack engine). Sprint 5B replaces this with **Wasmtime v50.0.1** with the **WebAssembly Component Model**, enabling native JIT execution via Cranelift, deterministic fuel metering, epoch-based interruption, typed WIT contracts, and fine-grained resource limiting.
3. **Component Model Target Support:** The Rust 1.99.1 compiler on this system natively supports the `wasm32-wasip2` target with built-in `wasm-component-ld`, producing valid Component Model binaries (`\0asm\r\0\x01\0` header) out of the box.
4. **Extension Points Identified:** Clean integration points exist for:
   - **Command Palette:** `CommandRegistry` in `src/workflow/command.rs` (can accept plugin-contributed commands with `AppAction::ExecutePluginCommand`).
   - **Preview/Inspector:** `PreviewManager` in `src/preview/manager.rs` and `PreviewSanitizer` in `src/preview/sanitizer.rs` (declarative UI data model with zero raw terminal escape sequences).
   - **Unified Search:** `SearchEngine` in `src/search/engine.rs` (can poll plugin search providers asynchronously with cancellation tickets).
   - **File Inspector / Telemetry:** Metadata tab integration in `src/ui/preview.rs`.
5. **Security Isolation:** Pure WebAssembly JIT isolation guarantees plugins cannot read or write arbitrary host memory. However, OS-level security requires strict host-side capability enforcement: default-deny permissions, opaque file handles, strictly scoped workspace access, bounded memory/fuel budgets, and sandboxed private storage.

---

## 2. Git Status & Working Tree Verification

- **Current Branch:** `master` (synchronized with `origin/master`).
- **Sprint 4 Work In Tree:** Sprint 4A (PTY Terminal Engine), Sprint 4B (Native Git Integration), Sprint 4C (Developer Workflow Engine), and Sprint 4D (Integration Gate) are present in the working tree:
  - Modified: `Cargo.lock`, `Cargo.toml`, `src/app.rs`, `src/git/mod.rs`, `src/lib.rs`, `src/preview/metadata.rs`, `src/shell/mod.rs`, `src/shell/pty.rs`, `src/shell/session.rs`, `src/state.rs`, `src/ui/command_palette.rs`, `src/ui/git_panel.rs`, `src/ui/hit_test.rs`, `src/ui/mod.rs`, `src/ui/preview.rs`, `src/ui/status_bar.rs`, `src/ui/terminal.rs`, `tests/adversarial_filesystem_tests.rs`, `tests/sprint_1_quality_gates.rs`, `tests/terminal_tests.rs`.
  - Untracked: `docs/SPRINT_4_PREREQUISITE_AUDIT.md`, `src/git/*.rs`, `src/shell/manager.rs`, `src/shell/terminal_screen.rs`, `src/ui/task_viewer.rs`, `src/workflow/`, `tests/git_tests.rs`, `tests/workflow_tests.rs`.
- **Sprint Invariant:** All existing uncommitted changes are strictly preserved. No destructive Git actions (`git reset --hard`, `git checkout .`, `git clean -f`) will be performed.

---

## 3. Sprint 5A Status & Baseline Verification

- **Sprint 5A Audit:** The project summary outlined Sprint 5A (Unicode Correctness & Terminal Hardening). A comprehensive audit reveals that the underlying Unicode infrastructure was already largely implemented during Sprints 1, 2, and 2.1:
  - `src/fonts/unicode.rs` provides `str_display_width`, `char_display_width`, and `slice_by_display_columns` respecting multi-cell wide CJK, emoji modifiers, and ZWJ sequences.
  - `src/editor/buffer.rs` enforces grapheme cluster boundary navigation and multi-cursor offset shifts without UTF-8 boundary panics.
  - `tests/unicode_and_emoji_tests.rs` validates emoji navigation and display widths.
- **Sprint 5B Precedence:** The user has explicitly requested immediate implementation of **Sprint 5B: WebAssembly Plugin Sandbox & Extensibility Architecture**. Zero blocking P0/P1 defects exist in the core application.

---

## 4. Current Subsystem Architecture Audit

### 4.1 Preview & Inspector Architecture (`src/preview/`)
- **Structure:** `PreviewManager` holds an LRU cache (64 entries) and dispatches background preview decoding via `tokio::task::spawn_blocking`.
- **Sanitization:** `src/preview/sanitizer.rs` (`sanitize_for_terminal`) validates UTF-8 and systematically strips dangerous control codes:
  - Dangerous C0 controls (NUL, SOH, STX, ENQ, BEL, BS, FF, VT).
  - CSI cursor movement and erase commands (`\x1b[...H`, `\x1b[...J`).
  - OSC escape sequences (window title, OSC 52 clipboard hijacking, OSC 8 hyperlinks).
- **Extension Strategy:** Plugin previewers must NOT return arbitrary strings with escape sequences or custom Ratatui widgets. Instead, WIT contracts will define declarative preview documents (`PreviewDocument` with title, sections, key-value tables, code blocks, and plain text), which the host renders using existing Catppuccin Mocha styles.

### 4.2 Workflow & Command Registry (`src/workflow/`)
- **Structure:** `CommandRegistry` manages all application actions. Commands are searched using `fuzzy_matcher::skim::SkimMatcherV2` with recency boosts and availability predicates.
- **Task Runner:** Spawns asynchronous processes with Unix process group isolation (`cmd.process_group(0)`), streaming output buffers (5,000 lines), and Workspace Trust enforcement.
- **Extension Strategy:** Plugins cannot execute raw host commands. If a plugin registers a command, it dispatches an invocation into the plugin's exported `run-command` function or proposes a structured task that requires user confirmation and native task runner execution.

### 4.3 Unified Search Subsystem (`src/search/`)
- **Structure:** `SearchEngine` coordinates fuzzy file searching (`search_files_fuzzy`) and regex content grep (`search_content`) on background worker threads, reporting through `mpsc::UnboundedSender<SearchResultEvent>` with monotonic generation tokens for instantaneous cancellation.
- **Extension Strategy:** Plugins with the `extension.register-search-provider` capability can contribute matches for specialized file formats or indexed metadata, respecting host cancellation tokens and maximum result limits (e.g. 50 items).

### 4.4 Filesystem & Security Abstractions (`src/filesystem/`)
- **Structure:** Transactional operations (`OperationLedger`) with atomic file writes via `tempfile` and chunked directory streaming (`chunked.rs`).
- **Security Rule:** No plugin receives direct access to arbitrary filesystem paths (`/etc`, `~/.ssh`, host binaries). The host mediates access via opaque handles (`FileHandle(u64)`), bounded byte ranges, and path canonicalization preventing directory traversal (`..`) and symlink escapes.

### 4.5 Configuration & XDG Storage Conventions (`src/config.rs`)
- Follows the XDG Base Directory specification:
  - Configuration: `~/.config/termexplorer/` (`Config::config_dir()`)
  - Plugins: `~/.local/share/termexplorer/plugins/`
  - Plugin Private Data: `~/.local/share/termexplorer/plugin-data/<plugin-id>/`
  - Plugin Cache: `~/.cache/termexplorer/plugins/`
  - Plugin Permissions: `~/.config/termexplorer/plugin_permissions.json`

---

## 5. Architectural Evaluation: Wasmtime vs. Wasmi

| Criterion | `wasmi v0.36` (Prototype) | `wasmtime v50.0.1` (Sprint 5B Selection) | Advantage / Rationale |
| :--- | :--- | :--- | :--- |
| **Execution Architecture** | Pure interpreter (Wasm bytecode loop) | Cranelift native JIT compilation | **10x–50x faster execution**, critical for previewing large files |
| **Component Model** | Not supported (Core Wasm only) | First-class native Component Model support | Clean typed interfaces, canonical ABI, no raw pointer hacking |
| **WIT Tooling** | None | Built-in `wasmtime::component::bindgen!` | Automated, strongly typed host/guest bindings |
| **Interruption & Metering** | Basic fuel consumption | Deterministic fuel + epoch-based interruption | Safe preemption of infinite loops and runaway computations |
| **Memory Isolation** | Linear memory limits | `ResourceLimiter` + per-instance memory caps | Hard bounding on Wasm memory allocation |
| **Security Track Record** | Embedded interpreter | Bytecode Alliance production-grade sandbox | Rigorously audited, fuzz-tested, formal verification |

**Decision:** Adopt **Wasmtime v50.0.1** with the `component-model` feature as the foundational plugin runtime.

---

## 6. Threat Model & Security Boundaries

```
┌────────────────────────────────────────────────────────────────────────┐
│                        TermExplorer Host Process                       │
│                                                                        │
│  ┌───────────────────────┐         ┌────────────────────────────────┐  │
│  │   UI & Event Loop     │         │  Plugin Permission Manager     │  │
│  │   (Ratatui / 60 FPS)  │         │  (Default-Deny, Scoped Caps)   │  │
│  └───────────┬───────────┘         └───────────────┬────────────────┘  │
│              │                                     │                   │
│              ▼                                     ▼                   │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │              Plugin Manager & Background Supervisor              │  │
│  │   - Bounded execution slots (max 4 concurrent)                   │  │
│  │   - Wall-clock timeouts (5s default, 10s analysis)               │  │
│  │   - Circuit breaker: disable plugin after 3 consecutive crashes  │  │
│  └──────────────────────────────────┬───────────────────────────────┘  │
│                                     │                                  │
│  ┌──────────────────────────────────▼───────────────────────────────┐  │
│  │                     Wasmtime Engine & Store                      │  │
│  │   - Fuel consumption: max 10,000,000 units                       │  │
│  │   - Memory limiter: max 32 MiB linear memory                     │  │
│  │   - Table elements: max 10,000                                   │  │
│  │                                                                  │  │
│  │   ┌──────────────────────────┐    ┌──────────────────────────┐   │  │
│  │   │  Plugin Instance A       │    │  Plugin Instance B       │   │  │
│  │   │  (Isolated Store & State)│    │  (Isolated Store & State)│   │  │
│  │   └──────────────────────────┘    └──────────────────────────┘   │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ NO DIRECT OS ACCESS
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                   Untrusted / Forbidden Operations                     │
│  ❌ Arbitrary filesystem access (`/`, `/etc`, `~/.ssh`)                │
│  ❌ Native process execution (`/bin/sh`, child commands)               │
│  ❌ Terminal PTY access or raw ANSI/OSC/CSI escape injection           │
│  ❌ Network socket connections (TCP/UDP/HTTP)                          │
│  ❌ Host environment variable inspection (`$TOKEN`, `$PASSWORD`)       │
│  ❌ Host clipboard access without explicit user consent                │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 7. Phased Implementation Roadmap

1. **Phase A: Dependency & Runtime Integration**
   - Add `wasmtime = { version = "50", features = ["component-model", "async"] }` to `Cargo.toml`.
   - Configure `Config` with fuel metering (`consume_fuel(true)`), epoch interruption, and `ResourceLimiter`.
   - Replace prototype `wasmi` code with `wasmtime::component::Component` and `wasmtime::Store`.
2. **Phase B: WIT Interface Definition & Host Bindings**
   - Create `wit/termexplorer/` with `types.wit`, `host.wit`, and `world.wit`.
   - Generate host bindings via `wasmtime::component::bindgen!`.
   - Implement host callback handlers with capability checks and opaque handle lookups.
3. **Phase C: Manifest Parsing, Discovery & Storage**
   - Implement `PluginManifest` parser with strict schema validation, version checks, and SHA-256 verification.
   - Implement `PluginPermissionManager` with persistence in `~/.config/termexplorer/plugin_permissions.json`.
   - Implement private plugin storage with quota checks in `~/.local/share/termexplorer/plugin-data/<id>/`.
4. **Phase D: Plugin Lifecycle & Supervisor**
   - Implement lifecycle state machine (`Discovered`, `Validating`, `AwaitingPermission`, `Installed`, `Enabled`, `Active`, `Disabled`, `Error`).
   - Circuit breaker: automatically disable plugins exceeding 3 consecutive traps or timeouts.
   - Thread isolation: all plugin invocations run in `tokio::task::spawn_blocking` with wall-clock cancellation timeouts.
5. **Phase E: Extension Integration**
   - Integrate plugin commands into `CommandRegistry` (`AppMode::CommandPalette`).
   - Integrate custom previews into `PreviewManager` (`PreviewTab::Preview`).
   - Integrate plugin search into `SearchEngine` (`SearchMode`).
6. **Phase F: Example Plugins & SDK**
   - Create `crates/termexplorer-plugin-sdk` for plugin authors.
   - Implement 3 example plugins compiled to `wasm32-wasip2`:
     - **File Statistics Plugin:** Calculates line/word counts on selected files.
     - **Custom Preview Plugin:** Renders structured `.recipe` or `.logfmt` files.
     - **Search Provider Plugin:** Indexes and matches specialized formats.
7. **Phase G: Plugin Manager UI & Verification**
   - Build `render_plugin_manager` modal UI displaying installed plugins, permission review, enable/disable toggles, and resource stats.
   - Adversarial security tests (infinite loop timeout, memory limit traps, unauthorized file access, path traversal, ANSI injection).
   - Quality gates: `cargo fmt`, `cargo check`, `cargo test`, `cargo clippy -D warnings`, `cargo build --release`.
