# TermExplorer Asynchronous Task Model

## Overview

TermExplorer employs an asynchronous runtime built on `tokio` that integrates synchronous terminal I/O (via `crossterm`), non-blocking filesystem notifications, background PTY shell process streams, and asynchronous multimedia preview generation.

---

## 1. Event Loop Architecture

The application loop in `src/app.rs` coordinates events through a centralized `tokio::select!` structure:

```rust
let event = tokio::select! {
    // 1. PTY shell bytes from background PTY thread
    Some(bytes) = pty_rx.recv() => { ... },

    // 2. Debounced filesystem change notifications
    Some(()) = fs_rx.recv() => AppEvent::FsChange,

    // 3. UI, keyboard, resize, and async completion events
    e = events.recv() => match e {
        Some(ev) => ev,
        None => break,
    },
};
```

### Channel Selection Rationale
- **Crossterm & Application Events (`UnboundedSender<AppEvent>`)**:
  Low volume, high priority events (keystrokes, window resizes, async completion notifications). An unbounded channel ensures user input is never dropped or blocked by buffer full conditions.
- **PTY Stream (`UnboundedSender<Vec<u8>>`)**:
  Raw PTY output from `portable-pty` background reader. Capped in memory by a 1 MB ring buffer (`VecDeque<u8>`).
- **Directory Chunk Streaming (`Sender<DirChunk>`)**:
  Bounded channel (capacity 16) ensuring backpressure between the background reader and the terminal renderer.

---

## 2. Monotonic Ticket Cancellation Pattern

To prevent race conditions, UI stutter, and redundant computation, all asynchronous background tasks (preview loading and directory streaming) use monotonic ticket counters:

```
[User Navigates: Item 1] ---> Ticket 101 Spawned (Token = 101)
[User Navigates: Item 2] ---> Ticket 102 Spawned (Token = 102)
                                 │
                                 ▼
                     Ticket 101 checks Token:
                     Token is 102 != 101
                     ===> Task 101 Aborts Immediately!
```

### Advantages
1. **Zero Resource Leaks**: Heavy operations (syntect syntax highlighting, PDF parsing, image decoders) terminate within microseconds of a new user keypress.
2. **Deterministic UI State**: The UI never renders stale preview data from an earlier cursor position if a newer file has already been highlighted.
3. **No Heavy Cancellation Primitives**: Relies solely on an `Arc<AtomicU64>` with `Ordering::Relaxed` loads, avoiding locks and thread contention.

---

## 3. Asynchronous Preview Scheduler

In `src/preview/manager.rs`:
- **Instant LRU Cache Check**: If a preview has already been decoded and its file `mtime` matches the cached timestamp, `get_cached` returns immediately with 0 ms latency.
- **Background Dispatch**: Uncached files trigger `PreviewManager::generate_preview_cancellable` on `tokio::task::spawn_blocking`.
- **Cooperative Checkpoints**: Before every CPU-intensive stage (file identification, metadata extraction, syntax highlighting, image decoders), `generate_preview_cancellable` verifies the cancellation token. If superseded, it exits immediately and returns `None`.
- **UI Notification**: When complete, an `AppEvent::PreviewReady` message is delivered to the main loop, where it is verified against `self.preview_ticket` before rendering.

---

## 4. Lifecycle Guarantees & Cleanup

- **PTY Cleanup**: `TerminalSession` implements `Drop` to kill child shell processes and close pseudoterminal handles immediately upon workstation shutdown or drawer toggle.
- **Terminal State Restoration**: An emergency panic hook installed in `main.rs` guarantees `disable_raw_mode()` and `LeaveAlternateScreen` are invoked even in the event of an unhandled panic.
