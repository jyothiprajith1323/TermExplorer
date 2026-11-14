# TermExplorer: Unified Search Engine Architecture

## 1. Overview

TermExplorer Sprint 3 introduces the Unified Workspace Search modal, synthesizing two fundamental discovery workflows into a cohesive, responsive UI overlay:
1. **Fuzzy Filename Discovery** (`Ctrl+P`): Instantly locate files and directories across large workspaces.
2. **Live Workspace Content Grep** (`Ctrl+Shift+F`): Real-time ripgrep-style substring and regex searching across file contents.

---

## 2. Architecture & Concurrency Model

```
[User Input]  --->  [SearchCancellation::advance()]  --->  [tokio::task::spawn_blocking]
                           |                                        |
                   (Atomic Generation)                  [WalkBuilder & Ignore]
                           |                                        |
                           +<---------[mpsc channel]<---------------+
                           |
               [SearchEngine::poll_events]
                           |
            +--------------+--------------+
            |                             |
     [Render Results]            [Auto-Update Preview]
```

1. **Non-Blocking Execution**: All workspace filesystem walking and regex evaluation runs in background worker threads via `tokio::task::spawn_blocking`. The UI thread remains at a steady 60+ FPS without dropped frames.
2. **Monotonic Cancellation Tokens**: As the user types query characters, the cancellation token sequence advances. Worker tasks checking `cancellation.is_cancelled(generation)` abort obsolete searches in under 2 ms.
3. **MPSC Channel Integration**: Results are streamed via unbounded channels to `SearchEngine::poll_events()` which runs during UI ticks.

---

## 3. Filters and Navigation Modes

- **Search Mode Toggle (`Tab`)**: Flips between `SearchMode::Files` and `SearchMode::Content`.
- **Search Filters**:
  - `Alt+C`: Case sensitivity toggle (`[Aa]`).
  - `Alt+R`: Regular expression syntax toggle (`[.*]`).
  - `Alt+W`: Whole word boundary matching (`[\b]`).
  - `.gitignore` Respect: Automatically ignores `.git/`, `target/`, `node_modules/`, and `.cargo/` unless explicitly overridden.
- **Search History**:
  - Automatically deduplicates and stores recent file and content queries in `$XDG_STATE_HOME/termexplorer/search_history.json`.

---

## 4. Seamless Editor and Preview Integration

- **Instant Preview on Selection**: Navigating results (`↑` / `↓` or `Ctrl+P` / `Ctrl+N`) immediately triggers preview loading in the 75% inspector panel.
- **Matched Line Centering**: In Content Grep mode, the preview panel automatically centers on the matched line (`scroll_offset = line.saturating_sub(5)`).
- **Direct Editor Jump (`Enter`)**:
  - Pressing `Enter` on a file result navigates the file explorer directly to that file.
  - Pressing `Enter` on a content match opens the file in the integrated editor and positions the cursor exactly at `(line - 1, 0)`.
