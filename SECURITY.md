# Security Policy

TermExplorer takes security, reliability, and data integrity seriously across its terminal user interface, embedded shell, file management operations, and WebAssembly plugin ecosystem.

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.1.x   | :white_check_mark: |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

---

## Security Architecture & Threat Model

### 1. Data Loss Prevention & Safe Filesystem Operations
- **Atomic File Saving**: Every buffer edit and configuration update is staged via temporary files in the target directory and committed via atomic rename (`save_file_atomic`), preserving original POSIX file permissions.
- **External Modification Detection**: The editor buffer tracks file `loaded_mtime` and checks the on-disk timestamp prior to write operations. Concurrent changes by external processes (e.g. Git checkouts, compilers) trigger collision guards rather than silent overwrites.
- **Unsaved Buffer Crash Recovery**: Unsaved modifications are persisted to an emergency snapshot store (`~/.local/share/termexplorer/recovery/`) upon critical events, enabling seamless restoration without data loss.
- **Cross-Filesystem Safety**: Moves between separate mounts/filesystems (`EXDEV = 18`) use copy-then-verify-then-remove semantics with automatic rollback on error.
- **FreeDesktop Trash Conformance**: Deletions generate compliant `.trashinfo` metadata (`[Trash Info]` path and timestamp headers) within `~/.local/share/Trash/`, preventing accidental irreversible loss.
- **Archive Path Traversal (Zip Slip) Protection**: Compressed archive inspectors and extractors sanitize entry paths, rejecting directory traversal sequences (`../`, absolute path escapes) and unbounded zip bombs.

### 2. WebAssembly (WASM) Plugin Sandboxing
- **Pure-Rust Sandboxed Runtime**: Executed inside an isolated `wasmi` WebAssembly interpreter with zero host-level syscall access.
- **Strict Memory Ceilings**: A hard 16 MiB linear memory limit is enforced per plugin instance (`memory.data(&store).len() <= 16 * 1024 * 1024`). Allocation breaches result in immediate guest trap termination without affecting the host.
- **Instruction Fuel Metering**: Every host-guest call allocates a bounded execution fuel quota (`DEFAULT_FUEL_LIMIT = 5_000_000` steps), eliminating infinite loop or CPU denial-of-service vulnerabilities.
- **Capability-Based Permissions**: Plugins must declare explicitly required capabilities (`file_metadata`, `commands`, `workspace_actions`, `context_actions`, `preview_provider`, `status_text`). Undeclared operations are rejected by the runtime.
- **Panic Isolation**: Guest execution errors, traps, and memory violations are caught within the host wrapper and logged to the diagnostics facility; the host TUI remains unaffected.

### 3. PTY Terminal & Child Process Lifecycle
- **PTY Process Isolation**: Terminal sessions run inside an isolated pseudo-terminal pair (`portable-pty`). Non-blocking channel I/O prevents terminal deadlocks.
- **Child Reaping on Drop**: Terminal sessions cleanly terminate child process trees upon drop or application exit to eliminate orphan or zombie processes.
- **Emergency Terminal Panic Hook**: A custom panic hook guarantees that terminal raw mode is disabled (`disable_raw_mode()`), alternate screens are exited (`LeaveAlternateScreen`), and mouse capture is released even in the event of an unhandled Rust panic, preventing broken or unresponsive user shells.

---

## Reporting a Vulnerability

If you discover a security vulnerability in TermExplorer, **please do not open a public issue**.

Instead, email the maintainer privately:
- **Email:** `jyothiprajith1323@gmail.com`
- **Subject:** `[SECURITY] Potential vulnerability in TermExplorer`

Please include:
1. Vulnerability description and vector (e.g., path traversal, memory exhaustion, shell injection).
2. Reproduction steps or a minimal Proof of Concept (PoC).
3. Assessed impact.

We will acknowledge receipt within 48 hours and coordinate remediation prior to public disclosure.
