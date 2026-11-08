# TermExplorer — Language Server Protocol (LSP) Architecture & Support

## Overview

TermExplorer provides an asynchronous, non-blocking Language Server Protocol (LSP) integration framework. The LSP client runs independently of terminal rendering, issuing asynchronous completion queries and diagnostic requests without blocking the 60fps UI loop.

---

## 4-Layer Completion Hierarchy

TermExplorer unifies code intelligence into 4 cooperative layers:

```text
┌─────────────────────────────────────────────────────────────────┐
│                    User Typing in Editor                        │
└────────────────────────────────┬────────────────────────────────┘
                                 │
                 ┌───────────────┴───────────────┐
                 ▼                               ▼
      Layer 1: Buffer Words            Layer 3: Snippets
      Local identifiers (<10ms)       Keyword templates (<1ms)
                 │                               │
                 └───────────────┬───────────────┘
                                 │
                 ┌───────────────┴───────────────┐
                 ▼                               ▼
    Layer 2: Workspace Symbols         Layer 4: Async LSP
    Project-wide symbols (<50ms)      Language Server Protocol
                 │                               │
                 └───────────────┬───────────────┘
                                 │
                                 ▼
                   Multi-Signal Ranking Engine
                    Deduplication & Top Max
                                 │
                                 ▼
                     Floating Completion Popup
```

---

## Supported Language Servers

TermExplorer automatically probes the system `PATH` for the following servers:

| Language | Primary Server Binary | Fallback Binary | File Extensions |
|---|---|---|---|
| **Rust** | `rust-analyzer` | — | `.rs` |
| **Python** | `pyright-langserver` | `pyright`, `pylsp` | `.py`, `.pyw` |
| **C / C++** | `clangd` | — | `.c`, `.cpp`, `.h`, `.hpp`, `.cc` |
| **Java** | `jdtls` | — | `.java` |
| **Go** | `gopls` | — | `.go` |
| **JavaScript / TypeScript** | `typescript-language-server` | `tsserver` | `.js`, `.ts`, `.jsx`, `.tsx` |
| **TOML** | `taplo` | — | `.toml` |
| **Markdown** | `marksman` | — | `.md`, `.markdown` |
| **SQL** | `sqls` | — | `.sql` |
| **Bash** | `bash-language-server` | — | `.sh`, `.bash` |

---

## Asynchronous Architecture, Framing & Cancellation (Sprint 2.1 Hardening)

In Sprint 2.1, the LSP engine was upgraded to a production-grade, asynchronous JSON-RPC 2.0 client (`src/editor/lsp.rs`):

### 1. HTTP-Style Header Framing
LSP specifications mandate HTTP-style header framing over stdio pipes:
* Format: `Content-Length: <size>\r\n\r\n<payload>`
* The client implements dedicated streaming buffer parsing (`parse_content_length`) capable of handling chunked and batched stdout streams without data loss.

### 2. UTF-16 Position Translation (LSP 3.17 Specification)
The LSP specification requires line/character positions to be encoded in **UTF-16 code units**, whereas Rust buffers operate on Unicode grapheme clusters and character offsets:
* `cursor_to_lsp_position(line, col)`: Converts buffer line and grapheme column into a zero-based LSP `Position { line, character }` where character is in UTF-16 code units (surrogate pairs count as 2).
* `lsp_position_to_cursor_col(line, lsp_character)`: Converts incoming LSP UTF-16 character offsets back to buffer grapheme cluster indices.

### 3. Tokio Actor Pattern
* **Child Process Management:** Spawns language server subprocesses using `tokio::process::Command` with piped `stdin`, `stdout`, and `stderr`.
* **Reader Task:** A dedicated background task reads stdio frames, routes server notifications (such as `textDocument/publishDiagnostics`), and fulfills pending request promises via `tokio::sync::oneshot`.
* **Writer Task:** Manages a thread-safe MPSC channel for outbound requests.
* **Pending Request Map:** Uses an `Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>` to match responses to request IDs with configurable timeouts (default 500ms for completions).

### 4. Cancellation Token Protocol
When a user types rapidly, previous completion queries are superseded:
* The client sends `$/cancelRequest` notification with `{ id: old_request_id }`.
* Stale in-flight results are cleanly dropped, preventing outdated completion menus from flashing onto the screen.

### 5. Verified Real Server Integration
The implementation is validated against the real host `rust-analyzer 1.99.1` binary (`tests/lsp_real_tests.rs`):
* `test_real_rust_analyzer_lifecycle_and_completions`: Full handshake (`initialize` -> `initialized` -> `textDocument/didOpen` -> `textDocument/completion` -> `shutdown` -> `exit`).
* `test_lsp_server_missing_binary_resilience`: Seamless, crash-free fallback when server binary is absent.
* `test_lsp_utf16_position_conversions`: Exact handling of ASCII, Cyrillic, CJK, and emoji surrogate pairs.

---

## Configuration

In `~/.config/termexplorer/config.toml`:

```toml
[editor]
show_completion = true
auto_pair = true
tab_width = 4

[editor.lsp]
enabled = true
rust_analyzer = "/usr/bin/rust-analyzer"
pyright = "/usr/local/bin/pyright-langserver"
clangd = "/usr/bin/clangd"
typescript_ls = "/usr/bin/typescript-language-server"
```
