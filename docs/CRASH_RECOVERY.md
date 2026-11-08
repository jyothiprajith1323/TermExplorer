# TermExplorer Crash Recovery & Persistence

## Overview

TermExplorer incorporates fault-tolerance mechanisms designed to protect unsaved editor work, restore terminal modes after sudden failures, and facilitate prompt recovery following unexpected workstation termination.

---

## 1. Dirty Buffer Snapshots (`create_recovery_snapshot`)

When files are edited in the built-in editor, TermExplorer creates background recovery snapshots:

- **Storage Location**: `~/.local/share/termexplorer/recovery/`
- **Hash-Partitioned Paths**:
  ```text
  ~/.local/share/termexplorer/recovery/<sha256(canonical_path)>.recovery
  ```
- **Lifecycle**:
  - Generated periodically when the buffer is dirty.
  - Removed cleanly (`clear_recovery_snapshot`) immediately upon successful file save (`save()` / `save_force()`).
  - Preserved on disk in the event of SIGKILL, OS crash, or power interruption.

---

## 2. Emergency Panic Hook & Terminal Restoration

A common issue with terminal-based workstations is leaving the user's terminal in an unreadable state (raw mode active, cursor hidden, alternate screen locked) if the program panics.

TermExplorer installs a global panic hook in `src/main.rs`:

```rust
let default_hook = std::panic::take_hook();
std::panic::set_hook(Box::new(move |panic_info| {
    let _ = disable_raw_mode();
    let _ = execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    default_hook(panic_info);
}));
```

### Guarantees
- Raw mode is disabled immediately.
- Terminal leaves the alternate screen buffer, restoring the user's original shell history.
- Mouse capture is disabled.
- Standard panic backtrace is printed to stdout/stderr cleanly.

---

## 3. Atomic Configuration Backups

Configuration changes (`Config::save`) follow atomic replacement semantics:
1. User settings are serialized to temporary files in `~/.config/termexplorer/`.
2. Flushed and synced to non-volatile storage.
3. Overwritten atomically using POSIX rename semantics.
4. If an invalid or corrupted configuration file is encountered upon launch, TermExplorer logs the corruption, creates a `.corrupted` backup copy, and initializes safe defaults without crashing.
