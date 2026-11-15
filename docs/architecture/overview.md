# TermExplorer Architecture Overview

TermExplorer is engineered as a high-performance, single-binary terminal developer workstation written in pure Rust.

---

## 1. Core Architectural Pillars

1. **Inspection-First Philosophy**:
   Developers spend significantly more time reading, inspecting, and navigating code than authoring new files. TermExplorer dedicates 75% of default screen space to deep, contextual inspection across documents, multimedia, code, and structured data.

2. **Strict Process & Thread Safety**:
   TermExplorer is built in safe Rust, enforcing zero-cost abstractions, memory safety, and thread safety. All asynchronous background tasks (file walking, syntax parsing, git queries, search) communicate over bounded Tokio channels with monotonic cancellation tokens.

3. **Non-Blocking UI Loop**:
   The terminal rendering thread never performs synchronous I/O or blocks on external processes. Even while compiling large projects or streaming high-throughput terminal output, Ratatui renders smoothly at a consistent frame rate.

4. **Layered Defense & Default-Deny Sandboxing**:
   All untrusted input—such as workspace task definitions, external terminal escape sequences, and WebAssembly plugins—operates within explicit security boundaries and capability matrices.

---

## 2. Technology Stack & Subsystems

| Subsystem | Primary Crate / Technology | Key Architectural Responsibility |
|---|---|---|
| **TUI Rendering** | `ratatui` (v0.30) + `crossterm` (v0.28) | Immediate-mode terminal rendering, double-buffering, layout algebra |
| **Async Runtime** | `tokio` (v1) | Multi-threaded async task executor, channels (`mpsc`), timers |
| **Text Editor Buffer**| `ropey` (v1.6) | B-tree rope data structure, grapheme slicing, delta undo history |
| **Syntax Highlighting**| `syntect` (v5.2) | TextMate grammar tokenization, viewport-bounded token caching |
| **Embedded Terminal** | `portable-pty` (v0.8) | Unix pseudo-terminal (`/dev/ptmx`), shell process supervision |
| **Plugin Sandbox** | `wasmi` (v0.36) | In-process WebAssembly interpreter, fuel metering, memory limiter |
| **Search Engine** | `ignore` + `regex` | Multi-threaded `.gitignore` traversal and parallel regex content grep |
| **Configuration** | `serde` + `toml` | Versioned schema (v2), atomic writes, automatic migration engine |
| **Git Integration** | Subprocess CLI (`std::process::Command`) | Status querying, unified diff parsing, staging, commit authoring |
| **Hashing & Digests** | `sha2` + `blake3` | Fast cryptographic content checksums and plugin integrity verification |

---

## 3. Subsystem Module Layout

```text
src/
├── app.rs               # Top-level application state & event dispatch loop
├── main.rs              # Binary CLI entry point, logging, terminal init/cleanup
├── lib.rs               # Public library crate interface
├── state.rs             # Application operational state & panel modes
├── config/              # Schema v2, migration engine, first-run manager
├── ui/                  # Ratatui rendering components, layouts, hit-testing
├── editor/              # Ropey buffer, autocomplete, LSP, syntax highlighting
├── filesystem/          # Chunked directory scanning, operations, trash ledger
├── preview/             # Multi-tier preview registry, format detectors, caching
├── search/              # Fuzzy filename search & parallel regex content grep
├── shell/               # PTY session management, terminal screen buffer, detection
├── git/                 # Git status, unified diff parser, commit manager
├── workflow/            # Task Runner, DAG dependency resolver, workspace trust
├── plugins/             # Wasmi sandboxing, manifest parser, capability permissions
├── settings/            # 14 Settings categories, keybinding manager, conflict detection
└── icons/               # Nerd Font & ASCII icon mapping engines
```
