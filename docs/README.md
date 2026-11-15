# TermExplorer Documentation Portal

Welcome to the technical knowledge base and documentation system for **TermExplorer v1.1.0**, an inspection-first terminal developer workstation written in pure Rust.

This documentation is organized into focused sections designed for users, contributors, plugin developers, and systems architects.

---

## 🧭 Documentation Navigator

### 1. [Getting Started](getting-started/quick-start.md)
Everything required to install, launch, configure, and troubleshoot your first TermExplorer session:
- **[Installation Guide](getting-started/installation.md)**: Native Linux packages (`.deb`, `.tar.gz`), package managers, source builds, and hardware requirements.
- **[First Launch & Onboarding](getting-started/first-launch.md)**: The 5-pathway Welcome surface, Guided Setup Wizard, and first-run detection.
- **[Quick Start Tutorial](getting-started/quick-start.md)**: A practical 5-minute tour of navigation, previewing, editing, and terminal tasks.
- **[Troubleshooting Guide](getting-started/troubleshooting.md)**: Diagnosing startup issues, terminal color degradations, missing tools, and corrupted configs.

### 2. [User Guide](user-guide/application-overview.md)
Comprehensive, feature-by-feature operational manuals for daily development:
- **[Workstation Overview](user-guide/application-overview.md)**: Inspection-first philosophy, 25%/75% default layout, panel focus, and status bar.
- **[Workspace Navigation](user-guide/navigation.md)**: Directory browsing, vim keybindings (`hjkl`), fuzzy filtering (`/`), and parent jumping.
- **[File Explorer](user-guide/file-explorer.md)**: File creation, renaming, sorting, hidden dotfile management, and FreeDesktop trash safety.
- **[Preview & Inspector Engine](user-guide/preview-inspector.md)**: Multi-format previewers: source code, Markdown, images, CSV/TSV, JSON/YAML, SQLite, PDF, media, and hex dumping.
- **[Integrated Code Editor](user-guide/editor.md)**: Rope-backed text editing, atomic persistence, undo/redo delta history, 4-layer autocomplete, and LSP diagnostics.
- **[Embedded Terminal Drawer](user-guide/terminal.md)**: Real interactive PTY drawer (15% height default), login shells, signal handling, and capabilities probing.
- **[Unified Search Engine](user-guide/search.md)**: Fuzzy filename indexing and fast ripgrep-style regex content search with live cancellation.
- **[Native Git Workbench](user-guide/git-workflow.md)**: Staged/unstaged inspection, unified diff viewer, inline staging (`s`), and commit creation (`c`).
- **[Task Runner & Workflow Engine](user-guide/task-runner.md)**: Project tech stack detection (Cargo, npm, pytest), DAG task execution, and workspace trust boundaries.
- **[Settings Studio](user-guide/settings.md)**: Interactive preferences interface across 14 categories with live theme and layout preview.
- **[Keyboard Shortcuts Guide](user-guide/keyboard-shortcuts.md)**: Centralized shortcut matrix, scope separation, and cheat sheet.
- **[WebAssembly Plugins](user-guide/plugins.md)**: Plugin Center (`Ctrl+Shift+X`), installed extensions, and permission management.

### 3. [Architecture Documentation](architecture/overview.md)
In-depth systems architecture documents covering internal components, ownership models, and lifecycle mechanics:
- **[Architecture Overview](architecture/overview.md)**: Architectural pillars, subsystem layout, and technology stack.
- **[System Context Diagram](architecture/system-context.md)**: Mermaid diagrams illustrating process boundaries and external integrations.
- **[Application Lifecycle](architecture/application-lifecycle.md)**: Startup initialization sequence, event loop, panic recovery hook, and clean terminal shutdown.
- **[Event Dispatch & Routing](architecture/event-system.md)**: Crossterm event streams, focus trees, modal dispatch hierarchy, and mouse hit-testing.
- **[UI Rendering Pipeline](architecture/ui-architecture.md)**: Ratatui layout algebra, terminal frame buffering, and performance diagnostics.
- **[Filesystem Subsystem](architecture/filesystem.md)**: Chunked viewport scanning, operations ledger, FreeDesktop trash protocol, and atomic writes.
- **[Editor Architecture](architecture/editor.md)**: Ropey data structure, UTF-8 coordinate conversion, transactional undo tree, and completion pipeline.
- **[Preview Engine Architecture](architecture/preview-engine.md)**: Magic-byte sniffing, capability registry, multi-tier LRU caching, and monotonic cancellation.
- **[Search Engine Architecture](architecture/search-engine.md)**: Parallel directory walking with `ignore`, ripgrep regex engine, and atomic cancellation.
- **[PTY Terminal Architecture](architecture/terminal-pty.md)**: `portable-pty` process supervision, non-blocking channel I/O, and VT100 screen buffer emulation.
- **[Git Integration Architecture](architecture/git-integration.md)**: Repository detection, asynchronous status polling, unified diff parser, and credential safety.
- **[Task Runner Architecture](architecture/task-runner.md)**: Task DAG dependency resolver, variable expansion, streaming process supervision, and trust boundary.
- **[WebAssembly Plugin Runtime](architecture/plugin-runtime.md)**: Wasmi sandboxed interpreter, fuel metering (5M), memory limits (16 MiB), and host import linkers.
- **[Configuration Architecture](architecture/configuration.md)**: Schema Version 2 design, migration engine, atomic file replacement, and corruption recovery.
- **[Caching Architecture](architecture/caching.md)**: Multi-tier LRU memory cache, disk cache, syntax cache, and invalidation rules.
- **[Concurrency & Async Model](architecture/concurrency.md)**: Tokio async runtime, thread pool boundaries, channels, and monotonic cancellation tokens.
- **[Security Boundaries](architecture/security-boundaries.md)**: Threat model, untrusted workspace policies, task runner trust, and terminal escape sequence sanitization.

### 4. [Developer & Contributor Guide](development/environment-setup.md)
Guidelines for building, debugging, testing, and contributing to the TermExplorer codebase:
- **[Development Environment Setup](development/environment-setup.md)**: Required Rust toolchain, system libraries, optional preview utilities, and WSL2 setup.
- **[Repository & Module Layout](development/project-structure.md)**: Codebase tour, crate dependencies, and source organization.
- **[Build & Test Workflows](development/build-and-test.md)**: Compiling debug/release targets, running test suites, and packaging commands.
- **[Debugging Guide](development/debugging.md)**: File-based tracing subscriber (`termexplorer.log`), panic diagnosis, and mock terminal testing.
- **[Profiling & Performance](development/profiling.md)**: Memory profiling, CPU flamegraphs, frame render latency diagnostics (`F12`), and benchmarks.
- **[Fuzzing Guide](development/fuzzing.md)**: Cargo-fuzz targets, corpus generation, and parser hardening.
- **[Testing Strategy](development/testing-strategy.md)**: Unit tests, integration tests, adversarial filesystem tests, and headless TUI tests.
- **[Code Style & Quality Standards](development/code-style.md)**: Rust formatting, strict Clippy compliance (`-D warnings`), and error handling principles.
- **[Release Engineering](development/release-engineering.md)**: Build scripts, Debian packaging, tarball generation, and SHA-256 manifest verification.

### 5. [WebAssembly Plugin Development Guide](plugin-development/overview.md)
Complete guide for external developers building sandboxed extensions:
- **[Plugin Platform Overview](plugin-development/overview.md)**: Sandboxed extension architecture and capabilities.
- **[Getting Started Walkthrough](plugin-development/getting-started.md)**: Writing, compiling, packaging, and installing a `#![no_std]` Rust plugin in 10 minutes.
- **[Component & ABI Model](plugin-development/component-model.md)**: WebAssembly ABI, linear memory layout, and string passing across the host boundary.
- **[Host & Guest Interface Reference](plugin-development/wit-reference.md)**: Complete specification of guest exports (`init`, `on_command`, `get_status_text`) and host imports (`host_log`, `host_get_workspace`, `host_get_active_file`, `host_set_status`).
- **[Plugin Manifest Reference](plugin-development/manifest-reference.md)**: Complete `plugin.toml` specification and field validation rules.
- **[Capability & Permissions Guide](plugin-development/permissions.md)**: Default-deny capability model, permission declaration, and host enforcement.
- **[Resource Limits & Sandboxing](plugin-development/resource-limits.md)**: Instruction fuel limits, linear memory ceilings (16 MiB), table bounds, and SHA-256 integrity verification.
- **[Plugin Lifecycle](plugin-development/lifecycle.md)**: Discovery, loading, validation, execution, and cleanup.
- **[Plugin Examples](plugin-development/examples.md)**: Annotated walkthroughs of built-in plugins (`file-inspector` and `workspace-tools`).

### 6. [Technical Reference](reference/configuration-reference.md)
Exhaustive reference specifications:
- **[Configuration Reference](reference/configuration-reference.md)**: All Schema v2 configuration keys, data types, defaults, and clamping ranges.
- **[Keyboard Shortcut Reference](reference/keyboard-reference.md)**: Complete shortcut matrix categorized by scope with conflict detection rules.
- **[Command Palette Reference](reference/command-reference.md)**: Complete registry of internal action IDs, categories, and keywords.
- **[Supported Formats Reference](reference/supported-formats.md)**: Comprehensive matrix of file formats, detection signatures, and viewer capabilities.
- **[Environment Variables](reference/environment-variables.md)**: Sourced environment variables (`TERM`, `COLORTERM`, `SHELL`, `LANG`, XDG paths).
- **[System Dependencies](reference/dependencies.md)**: Runtime and build-time library requirements (mandatory vs optional).
- **[Error Code Reference](reference/error-reference.md)**: Diagnostics codes, failure modes, and recovery procedures.

### 7. [Architecture Decision Records (ADRs)](decisions/README.md)
Historical record of core architectural decisions:
- **[ADR Index](decisions/README.md)**
- **[ADR-0001: Ratatui & Crossterm Workstation Architecture](decisions/ADR-0001-ratatui-crossterm-architecture.md)**
- **[ADR-0002: Rope Data Structure for Text Editing](decisions/ADR-0002-ropey-editor-buffer-model.md)**
- **[ADR-0003: Embedded PTY via portable-pty](decisions/ADR-0003-portable-pty-embedded-terminal.md)**
- **[ADR-0004: In-Process WebAssembly Sandboxing via wasmi](decisions/ADR-0004-wasmi-sandboxed-plugin-engine.md)**
- **[ADR-0005: Schema Version 2 Configuration & Migration Engine](decisions/ADR-0005-schema-v2-configuration-migration.md)**
- **[ADR-0006: Multi-Tier Preview Engine Architecture](decisions/ADR-0006-multi-tier-preview-architecture.md)**
- **[ADR-0007: Monotonic Cancellation Tokens for Async Operations](decisions/ADR-0007-monotonic-cancellation-and-concurrency.md)**
