# Debugging & Diagnostics Guide

Because TermExplorer operates an interactive terminal UI in raw mode, standard `println!()` statements will garble the screen buffer. This guide explains how to effectively debug the application.

---

## 1. File-Based Tracing & Log Inspection

TermExplorer uses `tracing` with a non-blocking background appender that writes directly to disk:
- **Log Location**: `~/.local/share/termexplorer/termexplorer.log`
- **Configuring Log Level**: Set `RUST_LOG` when launching:
  ```bash
  RUST_LOG=debug termexplorer
  # Or target a specific subsystem:
  RUST_LOG=termexplorer::preview=trace termexplorer
  ```

To monitor logs in real-time from a second terminal window:
```bash
tail -f ~/.local/share/termexplorer/termexplorer.log
```

---

## 2. Emergency Panic Recovery

TermExplorer installs an emergency panic hook at process startup. If a panic occurs:
1. Terminal raw mode is disabled immediately.
2. The alternate screen buffer is restored.
3. The mouse capture protocol is disabled.
4. The panic location and backtrace are printed cleanly to standard error.

To enable verbose backtraces:
```bash
RUST_BACKTRACE=1 termexplorer
```

---

## 3. Headless & Mock Terminal Testing

Integration tests in `tests/` execute without requiring a real GUI or terminal emulator:
- Ratatui's `TestBackend` simulates an in-memory terminal buffer `(width × height)`.
- Key events are delivered programmatically via `App::handle_key()`.
- The in-memory buffer can be asserted line-by-line or character-by-character.
