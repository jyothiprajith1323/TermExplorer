# Application Lifecycle & Shutdown Mechanics

This document explains TermExplorer's execution lifecycle from process initialization to clean terminal restoration.

---

## 1. Initialization Sequence

```mermaid
sequenceDiagram
    autonumber
    actor User
    participant Main as main.rs
    participant Tracing as Tracing Logger
    participant Config as Config Manager
    participant Hook as Panic Hook
    participant Term as Terminal / Crossterm
    participant App as App::new()
    participant Loop as Event Loop

    User->>Main: termexplorer [OPTIONS] [PATH]
    Main->>Main: Parse CLI flags (--version, --health, etc.)
    Main->>Tracing: Initialize daily rolling log (~/.local/share/termexplorer/termexplorer.log)
    Main->>Config: Config::load_or_recover() (Validate, Clamp, or Migrate)
    Main->>Hook: Install emergency panic recovery hook
    Main->>Term: enable_raw_mode(), EnterAlternateScreen, EnableMouseCapture
    Main->>App: Initialize App state, scan working directory
    App->>App: Detect FirstRunStatus (show Welcome Surface if fresh)
    Main->>Loop: Run tokio event loop (Render -> Poll Input -> Dispatch Action)
```

### Steps Explained:
1. **CLI Parsing**: Checks flags like `--help`, `--version`, `--health`, `--cwd-file`, `--chooser-file`.
2. **Logging**: Initializes file-based non-blocking tracing. Standard output/stderr are never written to directly during execution to avoid corrupting the TUI.
3. **Configuration**: Loads `config.toml` via `Config::load_or_recover()`. If corrupted, archives the file and loads defaults. Migrates `v1` to `v2` if needed.
4. **Emergency Panic Hook**: Wraps the default panic handler. If a panic occurs anywhere in the process, the hook immediately invokes `disable_raw_mode()`, `LeaveAlternateScreen`, and `DisableMouseCapture` so the user's terminal is never left in an unusable state.
5. **Terminal Setup**: Puts the terminal into raw mode and switches to the alternate screen buffer.
6. **State Initialization**: Initializes `App` state, starts the filesystem scanner, sets up the PTY manager, and detects first-run status.

---

## 2. Main Event Loop

The event loop runs asynchronously on Tokio:
- **Tick Frequency**: Driven by a periodic tick timer (every 16–33 ms depending on activity) and immediate wakeups on input events.
- **Render Phase**: Invokes `terminal.draw(|f| ui::render(f, &mut app))`. Ratatui evaluates layout rects and updates only dirty terminal cells using double buffering.
- **Event Phase**: Awaits Crossterm events (`Event::Key`, `Event::Mouse`, `Event::Resize`).
- **Dispatch Phase**: Routes keys to the currently focused modal or panel.
- **Background Phase**: Drains completed async tasks (directory chunks, preview generations, git status updates, task runner output).

---

## 3. Clean Shutdown & Restoration

When the user exits (via `Ctrl+Q`, `q`, or closing all panels):
1. **Child Process Termination**: The PTY manager sends `SIGHUP`/`SIGTERM` to the shell process and waits for exit, escalating to `SIGKILL` if necessary to prevent zombie processes.
2. **Buffer Flush**: Any pending log entries or configuration changes are synchronized to disk.
3. **CWD File Output**: If `--cwd-file <PATH>` was specified, writes the last navigated directory to the file (enabling shell wrappers like `cd $(termexplorer --cwd-file /tmp/cwd)`).
4. **Terminal Restoration**:
   - `disable_raw_mode()`
   - `execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)`
   - `terminal.show_cursor()`
5. **Clean Exit**: The process exits with code 0.
