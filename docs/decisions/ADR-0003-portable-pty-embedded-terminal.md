# ADR-0003: Embedded PTY Subprocess Supervision via portable-pty

- **Status**: Accepted
- **Context**: Reconstructed from Current Implementation (Sprint 4A)

---

## 1. Context & Problem Statement

TermExplorer aims to be an integrated developer workstation. Users frequently need to run builds, git commands, and shell utilities without leaving the workstation.
Simulating command execution by capturing stdout via pipes (`std::process::Command`) fails for interactive tools (`vim`, `less`, `ssh`, `htop`, interactive prompts) because processes detect they are not connected to a TTY.

## 2. Decision

TermExplorer integrates a full Unix pseudo-terminal (PTY) using **`portable-pty`** (`src/shell/pty.rs`):
- Allocates a master/slave PTY pair via `/dev/ptmx`.
- Spawns an interactive login shell (`$SHELL -l`).
- Dedicated background threads read from and write to the master file descriptor.
- Dynamic terminal resizing communicates window dimensions via `ioctl(TIOCSWINSZ)`.
- Terminal escape sequences are parsed into an in-memory `TerminalScreen` cell buffer.

## 3. Alternatives Considered

1. **Direct `nix::pty::openpty`**: Low-level POSIX bindings requiring substantial boilerplate for child process supervision, signal masking, and cross-platform WSL compatibility.
2. **External Multiplexer (Tmux / Zellij integration)**: Requires external tools to be installed and forces users into complex keybinding namespaces.

## 4. Consequences

- **Positive**: True interactive shell behavior; supports curses programs, readline editing, and shell aliases.
- **Negative**: High output throughput (e.g. `cat /dev/urandom`) requires non-blocking channel throttling to avoid starving the UI thread.
- **Security Impact**: The shell runs with standard user permissions; child processes are cleaned up upon application exit via `SIGHUP`/`SIGKILL`.
