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

## Asynchronous Architecture & Stale Cancellation

1. **Non-Blocking Execution**: Completion requests are dispatched asynchronously. The terminal UI never waits or blocks on LSP socket or pipe communication.
2. **Generation Tokens (`CancellationToken`)**: Each keystroke increments an atomic generation token. When an asynchronous LSP response arrives, its token is compared against the active token. Stale responses from previous keystrokes are discarded immediately.
3. **Graceful Fallback**: If no LSP binary is installed for a language, TermExplorer gracefully falls back to Layer 1 (Buffer Words), Layer 2 (Workspace Symbols), and Layer 3 (Snippets) without showing error banners or disrupting workflow.

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
