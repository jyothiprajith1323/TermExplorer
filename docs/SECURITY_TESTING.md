# TermExplorer Security Architecture & Adversarial Testing

## Overview

Terminal emulators and text workstations are vulnerable to terminal injection attacks, clipboard exfiltration, archive directory traversal, and resource exhaustion bombs. TermExplorer employs multi-layered defenses verified by automated adversarial test suites.

---

## 1. Terminal Escape Sanitization (GAP-04)

Previewing untrusted files (downloaded source code, logs, binary blobs) in a terminal can execute arbitrary terminal escape sequences if rendered raw.

### Defense Implementation (`src/preview/sanitizer.rs`)
TermExplorer sanitizes all preview lines prior to feeding them to the ratatui rendering pipeline:
- **OSC 52 Clipboard Hijacking**: Strips `\x1b]52;...` sequences that attempt to read or overwrite the user's desktop clipboard.
- **OSC 8 Hyperlink Spoofing**: Strips hidden URI payloads (`\x1b]8;;http://evil.com\x07`) while retaining safe visible label text.
- **OSC Window & Palette Manipulation**: Strips `\x1b]0;`, `\x1b]2;`, and palette redefinition sequences.
- **CSI Screen/Cursor Manipulation**: Strips cursor repositioning (`\x1b[H`, `\x1b[row;colH`), clear screen (`\x1b[2J`), and mode toggles.
- **DCS & Device Status**: Strips Device Control Strings (`\x1bP`).
- **C0 Non-Printable Controls**: Strips bell (`\x07`), backspace (`\x08`), form-feed (`\x0c`), while preserving necessary formatting characters (`\t`, `\n`, `\r`).

---

## 2. Archive Safety & Decompression Bombs

Archive extraction (`src/preview/archive/mod.rs`) enforces strict bounds:
- **Decompression Bomb Ceilings**:
  - `MAX_EXTRACT_BYTES`: Capped at **500 MiB**. Extraction tracks cumulative uncompressed bytes across all entries and aborts with a clear error if this ceiling is exceeded.
  - `MAX_EXTRACT_FILES`: Capped at **10,000 files**.
- **Path Traversal Prevention (`Zip Slip`)**:
  - Prevents paths starting with `/` or containing `..` components.
  - Verifies canonicalized destination paths are strictly contained within `target_dir` using `starts_with(target_dir)`.
  - Rejects attempts to extract symlinks pointing outside the target directory.

---

## 3. WebAssembly Plugin Isolation

TermExplorer plugins execute inside a sandboxed WebAssembly runtime (`wasmtime`):
- **Fuel Metering**: Infinite loops and CPU-spinning plugins are halted within bounded milliseconds.
- **Memory Growth Limits**: Initial and maximum WebAssembly heap allocations are capped to prevent OOM denial-of-service.
- **No Undeclared Syscalls**: Plugins have zero direct access to the host filesystem, network sockets, or OS processes unless explicitly granted capabilities in the plugin manifest.
