# TermExplorer Architecture

For the complete in-depth architectural blueprint, subsystem breakdowns, data flow diagrams, and memory management guarantees, please consult:

👉 **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)**

---

## Architectural Summary

TermExplorer is architected as an asynchronous, event-driven terminal workstation built on Ratatui, Tokio, and Crossterm.

```text
┌────────────────────────────────────────────────────────────────────────┐
│                          Ratatui TUI Engine                            │
│  ┌─────────────────────────┬────────────────────────────────────────┐  │
│  │   File Explorer (25%)   │      Contextual Preview (75%)          │  │
│  │   Directory Tree        │      Source / Markdown / Media         │  │
│  │   Navigation / Sorting  │      [Preview|Info|Hex|Actions]        │  │
│  └─────────────────────────┴────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │   Integrated Linux Terminal Drawer (15% height, hidden by default)│  │
│  └──────────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────┬───────────────────────────────────┘
                                     │
           ┌─────────────────────────┼─────────────────────────┐
           ▼                         ▼                         ▼
┌────────────────────┐    ┌────────────────────┐    ┌────────────────────┐
│   Editor Buffer    │    │ 4-Layer Completion │    │    Wasm Sandbox    │
│   Rope Data Backing│    │ Words/Symbols/Snips│    │ 16 MiB Linear Mem  │
│   Atomic POSIX Save│    │ Non-blocking Tokio │    │ 5M Instruction Fuel│
└────────────────────┘    └────────────────────┘    └────────────────────┘
```

### Core Subsystems
1. **Application Shell (`src/app.rs`):** Central event routing, 6-tier adaptive layout, modal dialogs, and key translation.
2. **Contextual Preview (`src/preview/`):** Multi-format sniffer, syntax tokenization, 24-bit Truecolor half-block image renderer, and bounded LRU caching.
3. **Advanced Editor (`src/editor/`):** `ropey` rope buffer, character-safe indexing, visual selections, search/replace, and bounded undo/redo.
4. **Autocomplete Engine (`src/completion/`):** 4-layer provider architecture with multi-signal ranking, prefix caching, and async LSP integration.
5. **Interactive Terminal (`src/terminal/`):** Pseudo-terminal session manager (`portable-pty`) with non-blocking channel polling and clean child process reaping.
6. **Wasm Plugin Engine (`src/plugin/`):** Capability-governed, fuel-metered WebAssembly interpreter (`wasmi`) isolated in user memory.
