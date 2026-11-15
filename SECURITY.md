# Security Policy

TermExplorer takes security, reliability, and data integrity seriously across its terminal user interface, embedded shell, file management operations, task execution, and WebAssembly plugin ecosystem.

---

## 1. Supported Versions

| Version | Supported | Security Maintenance Status |
|---|:---:|---|
| **1.1.x** | :white_check_mark: | Current active release branch |
| **1.0.x** | :white_check_mark: | Critical security patches only |
| **< 1.0** | :x: | Unsupported |

---

## 2. Security Architecture & Threat Boundaries

### A. Data Loss Prevention & Safe Filesystem Operations
- **Atomic File Saving**: Every buffer edit and configuration update is staged via temporary files in the target directory and committed via atomic rename (`save_file_atomic`), preserving original POSIX file permissions.
- **External Modification Detection**: The editor buffer tracks file `loaded_mtime` and checks the on-disk timestamp prior to write operations. Concurrent changes by external processes (e.g. Git checkouts, compilers) trigger collision guards rather than silent overwrites.
- **Unsaved Buffer Crash Recovery**: Unsaved modifications are persisted to an emergency snapshot store (`~/.local/share/termexplorer/recovery/`) upon critical events, enabling seamless restoration without data loss.
- **Cross-Filesystem Safety**: Moves between separate mounts/filesystems (`EXDEV = 18`) use copy-then-verify-then-remove semantics with automatic rollback on error.
- **FreeDesktop Trash Conformance**: Deletions generate compliant `.trashinfo` metadata within `~/.local/share/Trash/`, preventing accidental irreversible loss.
- **Archive Path Traversal (Zip Slip) Protection**: Compressed archive inspectors and extractors sanitize entry paths, rejecting directory traversal sequences (`../`, absolute path escapes) and unbounded zip bombs.

### B. Workspace Trust Boundary & Task Execution
- **Untrusted Repositories**: Cloned repositories containing custom task definitions (`.termexplorer/tasks.toml`) are not executed automatically.
- **Default-Deny Execution**: TermExplorer prompts the user for explicit trust confirmation before running tasks or loading workspace task definitions.
- **Trust Persistence**: Approved workspaces are hashed and recorded in `~/.local/share/termexplorer/trusted_workspaces.json`.

### C. Terminal Escape Sequence Sanitization
- **Escape Stripping**: Untrusted file contents displayed in the preview pane pass through `sanitize_preview_line()`, which strips raw ANSI/VT control codes (ASCII 0–31 except `\n` and `\t`) and disables dangerous OSC sequences that could exploit host terminal emulators.

### D. WebAssembly Plugin Sandboxing
- **Pure-Rust Sandboxed Runtime**: Executed inside an isolated `wasmi` WebAssembly interpreter with zero host-level syscall access.
- **Strict Memory Ceilings**: A hard 16 MiB linear memory limit is enforced per plugin instance. Allocation breaches result in immediate guest trap termination without affecting the host.
- **Instruction Fuel Metering**: Every host-guest call allocates a bounded execution fuel quota (`DEFAULT_FUEL_LIMIT = 5_000_000` steps), eliminating infinite loop or CPU denial-of-service vulnerabilities.
- **Capability-Based Permissions**: Plugins must declare explicitly required capabilities (`file_metadata`, `commands`, `workspace_actions`, `context_actions`, `preview_provider`, `status_text`). Undeclared operations are rejected by the runtime.
- **Integrity Checksums**: Manifests declare binary SHA-256 digests; mismatched binaries are rejected prior to execution.

### E. PTY Terminal & Child Process Lifecycle
- **PTY Process Isolation**: Terminal sessions run inside an isolated pseudo-terminal pair (`portable-pty`). Non-blocking channel I/O prevents terminal deadlocks.
- **Child Process Cleanup**: Terminal sessions cleanly terminate child process trees upon drop or application exit via `SIGHUP`/`SIGKILL` to eliminate orphan processes.
- **Emergency Panic Hook**: A custom panic hook guarantees that terminal raw mode is disabled (`disable_raw_mode()`), alternate screens are exited (`LeaveAlternateScreen`), and mouse capture is released even in the event of an unhandled Rust panic, preventing broken or unresponsive user shells.

---

## 3. Reporting a Vulnerability

If you discover a security vulnerability in TermExplorer, **please do not open a public issue**.

Instead, email the maintainer privately:
- **Email:** `jyothiprajith1323@gmail.com`
- **Subject:** `[SECURITY] Potential vulnerability in TermExplorer`

Please include:
1. Vulnerability description and vector (e.g., path traversal, memory exhaustion, escape injection).
2. Reproduction steps or a minimal Proof of Concept (PoC).
3. Assessed impact.

We will acknowledge receipt within 48 hours and coordinate remediation prior to public disclosure.
