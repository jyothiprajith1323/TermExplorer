# Architecture Decision Records (ADRs)

This directory documents the significant architectural decisions that govern TermExplorer's design, subsystems, and security boundaries.

Decisions are labeled as **Reconstructed from Current Implementation** where historical discussions were not formally logged prior to Sprint 6.

---

## Decision Log

| ADR | Title | Status | Date |
|---|---|:---:|---|
| **[ADR-0001](ADR-0001-ratatui-crossterm-architecture.md)** | Ratatui & Crossterm Immediate-Mode TUI Architecture | Accepted | Reconstructed (Sprint 1) |
| **[ADR-0002](ADR-0002-ropey-editor-buffer-model.md)** | Rope Data Structure for Text Editor Buffer | Accepted | Reconstructed (Sprint 2) |
| **[ADR-0003](ADR-0003-portable-pty-embedded-terminal.md)** | Embedded PTY Subprocess Supervision via portable-pty | Accepted | Reconstructed (Sprint 4A) |
| **[ADR-0004](ADR-0004-wasmi-sandboxed-plugin-engine.md)** | In-Process WebAssembly Sandboxing via wasmi | Accepted | Reconstructed (Sprint 5B) |
| **[ADR-0005](ADR-0005-schema-v2-configuration-migration.md)** | Schema Version 2 Configuration & Backward-Compatible Migration Engine | Accepted | 2026-11-14 (Sprint 6B) |
| **[ADR-0006](ADR-0006-multi-tier-preview-architecture.md)** | Multi-Tier Preview Engine with Magic-Byte Sniffing | Accepted | Reconstructed (Sprint 3A) |
| **[ADR-0007](ADR-0007-monotonic-cancellation-and-concurrency.md)** | Monotonic Generation Tokens for Async Cancellation | Accepted | Reconstructed (Sprint 5C) |
