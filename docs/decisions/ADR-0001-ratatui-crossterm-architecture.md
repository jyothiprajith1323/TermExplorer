# ADR-0001: Ratatui & Crossterm Immediate-Mode TUI Architecture

- **Status**: Accepted
- **Context**: Reconstructed from Current Implementation (Sprint 1)

---

## 1. Context & Problem Statement

TermExplorer requires a terminal rendering engine capable of:
1. Low-latency, flicker-free rendering at up to 60 FPS.
2. Cross-platform raw mode input handling (keyboard, mouse, terminal resizing).
3. Complex split-panel layout geometry (Explorer, Preview, Editor, Terminal drawer, overlays).
4. ANSI/VT100 escape sequence generation with minimal bandwidth consumption over SSH.

## 2. Decision

TermExplorer uses **Ratatui** (v0.30) with the **Crossterm** (v0.28) backend in raw terminal mode:
- Immediate-mode rendering construct: each frame is rendered from state into double-buffered cell grids.
- Ratatui computes diffs between previous and current frames, emitting ANSI escape codes only for mutated cells.
- Crossterm manages raw mode, alternate screen buffers (`EnterAlternateScreen`), and mouse capture (`EnableMouseCapture`).

## 3. Alternatives Considered

1. **Cursive (Retained-Mode UI)**: Required complex component lifecycle management and interior mutability (`Rc<RefCell<...>>`) that conflicted with safe asynchronous state mutations.
2. **Custom ANSI Renderer**: Would require writing and maintaining terminal parser abstractions, cursor addressing, and double buffering from scratch.

## 4. Consequences

- **Positive**: High redraw performance (<2 ms per frame), safe memory model, responsive layout resizing.
- **Negative**: Full-screen redrawing occurs when switching large views; requires careful viewport bounding for large text files.
- **Security Impact**: Isolates stdout writes from application logic, preventing unescaped terminal injection.
- **Performance Impact**: Frame diffing reduces network bandwidth over remote terminal sessions.
