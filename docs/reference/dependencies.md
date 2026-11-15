# System Dependencies & Package Reference

TermExplorer is engineered with minimal external library requirements to ensure painless deployment.

---

## 1. Runtime Shared Library Dependencies

The compiled Linux release binary links against standard POSIX C libraries:

| Library | Minimum Version | Purpose | Included by Default? |
|---|---|---|---|
| `glibc` | 2.31+ (Ubuntu 20.04+) | C standard library, POSIX calls, threading (`pthreads`) | Yes (standard Linux) |
| `libgcc_s` | 1 | GCC low-level runtime | Yes (standard Linux) |
| `libm` | 6 | Math library | Yes (standard Linux) |

TermExplorer does **not** dynamically link against GUI toolkits (GTK, Qt), X11, Wayland, or Cairo.

---

## 2. Optional CLI Tool Integrations

These command-line tools are detected in `$PATH` at runtime if installed:

| Tool | Recommended Package | Feature Enabled | Fallback Behavior if Missing |
|---|---|---|---|
| `git` | `git` | Git status bar, diff viewer, staging, branch switching | Git status indicator disabled cleanly |
| `ffmpeg` | `ffmpeg` | First-frame video thumbnail image rendering | Video container metadata displayed without frame preview |
| `pdftotext` | `poppler-utils` | High-fidelity PDF document text stream rendering | Pure-Rust basic stream extractor |
| `sqlite3` | `sqlite3` | SQLite schema tables and sample query executor | Pure-Rust SQLite header decoder |
| `trash-cli` | `trash-cli` | FreeDesktop Trash compliance (`gio trash` also supported) | Internal archive fallback in `.termexplorer/trash/` |
| `rust-analyzer` | `rust-analyzer` | Language Server Protocol completion and diagnostics | Layer 1-3 buffer/workspace word autocompletion |
