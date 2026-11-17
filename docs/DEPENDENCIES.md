# 📦 TermExplorer — Runtime & System Dependencies Specification

This document provides a comprehensive inventory of all system-level, dynamically linked, bundled, and optional runtime dependencies for **TermExplorer**.

---

## 🏛️ Dependency Classification Matrix

| Dependency | Category | Required / Optional | Detection Mechanism | Missing Dependency Behavior |
|:---|:---|:---|:---|:---|
| **GNU C Library (`libc.so.6`)** | Dynamic Shared Library | **Required** | ELF dynamic linker (`ld.so`) | Dynamic loader failure (`exit 127`) |
| **GCC Runtime (`libgcc_s.so.1`)** | Dynamic Shared Library | **Required** | ELF dynamic linker (`ld.so`) | Dynamic loader failure (`exit 127`) |
| **Math Library (`libm.so.6`)** | Dynamic Shared Library | **Required** | ELF dynamic linker (`ld.so`) | Dynamic loader failure (`exit 127`) |
| **Git (`git`)** | External CLI Tool | **Optional (Recommended)** | `$PATH` lookup via `which git` / `Command::new("git")` | Git panel displays clean degradation message: *"Git executable not found in $PATH"* |
| **FFmpeg (`ffmpeg`)** | External CLI Tool | **Optional** | `$PATH` lookup via `preview/registry.rs` | Video frames and audio waveforms show informative banner: *"Install ffmpeg for audio/video media preview"* |
| **Poppler Utils (`pdftoppm`)** | External CLI Tool | **Optional** | `$PATH` lookup via `preview/registry.rs` | PDF preview pane displays text fallback or install notice: *"Install poppler-utils for PDF rendering"* |
| **SQLite3 CLI (`sqlite3`)** | External CLI Tool | **Optional** | `$PATH` lookup via `preview/registry.rs` | Structured SQLite preview falls back to embedded `rusqlite` engine |
| **WebAssembly Sandbox** | Statically Bundled | **Bundled** | In-tree Rust crate (`wasmi v0.36`) | Always available; isolated WebAssembly memory sandbox |
| **Syntax Highlighting** | Statically Bundled | **Bundled** | In-tree Rust crate (`syntect v5.3`) | Always available; zero runtime external dependencies |
| **Vector Graphics (`resvg`)**| Statically Bundled | **Bundled** | In-tree Rust crate (`resvg v0.50`) | Always available; renders SVGs natively without external librsvg |

---

## 1. Required System Shared Libraries (ABI)

TermExplorer is compiled as a native Linux binary with statically compiled application logic and minimal dynamic linkage to the host C runtime:

### Dynamic Linkage Verification
Inspection via `readelf -d target/release/termexplorer`:
```
0x0000000000000001 (NEEDED)             Shared library: [libgcc_s.so.1]
0x0000000000000001 (NEEDED)             Shared library: [libm.so.6]
0x0000000000000001 (NEEDED)             Shared library: [libc.so.6]
0x0000000000000001 (NEEDED)             Shared library: [ld-linux-x86-64.so.2]
```

### Supported C Library Baseline
- **Minimum GLIBC:** `glibc >= 2.35` (supported on Ubuntu 22.04 LTS+, Debian 12 Bookworm+, Fedora 36+, Arch Linux, openSUSE Leap 15.4+). Older distributions such as Ubuntu 20.04 (Focal) or Debian 11 (Bullseye) with GLIBC < 2.35 are not supported by the official binaries.
- **Architecture:** `x86_64` (ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV)).
- **ARM64:** Fully supported for `aarch64-unknown-linux-gnu` cross-compilation.

---

## 2. Optional External CLI Integrations

TermExplorer is designed to be **resilient to missing external tools**. Missing optional tools will **never cause a crash or panic**. The application gracefully degrades and displays actionable install instructions in the relevant UI panes.

### A. Git (`git`)
- **Purpose:** Native Git status, branch switching, stage/unstage, diff viewer, and commit history.
- **Minimum Version:** `git >= 2.25.0`
- **Installation:**
  - Ubuntu/Debian: `sudo apt-get install git`
  - Fedora/RHEL: `sudo dnf install git`
  - Arch Linux: `sudo pacman -S git`

### B. FFmpeg (`ffmpeg`)
- **Purpose:** Extracting video preview keyframes, audio waveform visualization, and media metadata.
- **Minimum Version:** `ffmpeg >= 4.4`
- **Installation:**
  - Ubuntu/Debian: `sudo apt-get install ffmpeg`
  - Fedora/RHEL: `sudo dnf install ffmpeg`
  - Arch Linux: `sudo pacman -S ffmpeg`

### C. Poppler Utilities (`pdftoppm`)
- **Purpose:** High-resolution vector rasterization of PDF pages for terminal graphical rendering.
- **Minimum Version:** `poppler >= 20.0`
- **Installation:**
  - Ubuntu/Debian: `sudo apt-get install poppler-utils`
  - Fedora/RHEL: `sudo dnf install poppler-utils`
  - Arch Linux: `sudo pacman -S poppler`

### D. SQLite (`sqlite3`)
- **Purpose:** CLI database inspection fallback and schema dumping.
- **Minimum Version:** `sqlite3 >= 3.35`
- **Installation:**
  - Ubuntu/Debian: `sudo apt-get install sqlite3`
  - Fedora/RHEL: `sudo dnf install sqlite3`
  - Arch Linux: `sudo pacman -S sqlite`

---

## 3. Bundled & Statically Linked Components

All internal parsers and processing engines are statically compiled directly into the `termexplorer` binary to avoid dynamic library hell:

1. **WebAssembly Engine:** Statically linked `wasmi` (v0.36) WebAssembly interpreter with memory sandboxing and gas metering.
2. **SQLite Database Driver:** Statically linked `rusqlite` (bundled SQLite amalgamation).
3. **SVG Vector Renderer:** Statically linked `resvg` pure Rust vector renderer.
4. **Syntax Highlighter:** Statically linked `syntect` with embedded Sublime Text syntax definitions and themes.
5. **Media Metadata:** Statically linked `lofty` audio tag reader and `image` decoder.

---

## 4. Third-Party Licenses & Redistribution Inventory

All statically bundled dependencies are permissively licensed:

| Component | License | Permissible Redistribution | Notice Requirement |
|:---|:---|:---|:---|
| TermExplorer | MIT | Yes | Included in `LICENSE` |
| Ratatui / Crossterm | MIT | Yes | Included |
| Tokio / Wasmi / Resvg | Apache-2.0 / MIT | Yes | Included |
| Syntect / Sublime Syntaxes | MIT | Yes | Included |
| SQLite (Rusqlite) | Public Domain | Yes | Included |
| Image / Lofty | MIT / Apache-2.0 | Yes | Included |
