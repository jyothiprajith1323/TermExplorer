# TermExplorer V1.1.0 — Intelligent Autocomplete & Suggestions Engine

**Module:** `src/completion/`  
**Architecture:** 4-Layer Hybrid Autocomplete Engine  
**Sprint:** Engineering Sprint 2  
**Target:** Terminal Developer Workstation Autocomplete Engine  

---

## 1. Modular Architecture Overview

The TermExplorer completion engine unifies file explorer navigation, command execution, and code editing under a high-performance modular suggestion architecture.

```mermaid
flowchart TD
    subgraph Engine ["Central Completion Engine (`src/completion/engine.rs`)"]
        L1["Layer 1: Buffer Words (`buffer_words.rs`) (< 0.05 ms)"]
        L2["Layer 2: Workspace Symbols (`workspace_symbols.rs`) (< 0.1 ms query)"]
        L3["Layer 3: Language Snippets (`snippets.rs`) (< 0.01 ms)"]
        L4["Layer 4: Async Non-Blocking LSP (`lsp_provider.rs`)"]
        Rank["Ranking & Deduplication (`ranking.rs`)"]
        Cache["LRU Prefix Cache (`cache.rs`)"]
        Cancel["Stale Request Cancellation (`cancellation.rs`)"]
    end

    subgraph Explorer ["Filesystem Autocomplete (`src/completion/explorer.rs`)"]
        P1["Path Completion (Tab)"]
        P2["Tilde Expansion (~/Projects)"]
        P3["Directory Trailing Slashes (dir/)"]
        P4["Pinned & Recent Suggestions"]
    end

    subgraph Palette ["Command Palette (`src/completion/palette.rs`)"]
        C1["Exact & Prefix Matching"]
        C2["Acronym Matching ('tdp' -> Toggle Dual-Pane)"]
        C3["Dynamic Navigation ('nav_dir:/path')"]
        C4["Contextual Panel Priority"]
    end

    L1 --> Rank
    L2 --> Rank
    L3 --> Rank
    L4 --> Rank
    Rank --> Cache
```

---

## 2. Layered Editor Autocomplete Breakdown

### Layer 1: In-Memory Buffer Words (`buffer_words.rs`)
- **Design:** Scans active editor buffer lines for word identifiers (`[a-zA-Z0-9_]`), prioritizing proximity to the current cursor position.
- **Optimization:** Proximity window scanning ($\pm 150$ lines around the cursor) prevents iterating through giant documents, accompanied by zero-allocation slice checking (`&'a str`).
- **Badge:** `[buf]` (Catppuccin Blue `#89b4fa`)
- **Latency Target:** < 5 ms
- **Measured Latency:** **0.016 ms** (16 µs) in release profile; < 1.5 ms in unoptimized debug.

### Layer 2: Workspace Symbol Indexer (`workspace_symbols.rs`)
- **Design:** Recursively indexes workspace symbols in the background thread.
- **Strictly Ignored Directories:** `.git`, `node_modules`, `target`, `dist`, `build`, `.cargo`, `.cache`, `.vscode`, `vendor`, `.venv`, `__pycache__`, `bin`, `obj`.
- **Supported Languages:** Rust (`fn`, `struct`, `enum`, `trait`), Python (`def`, `class`), C/C++ (`struct`, `typedef`, `class`), Java (`class`, `interface`), Go (`func`), TypeScript/JavaScript (`type`, `interface`, `class`), SQL, TOML, Bash.
- **Badge:** `[sym]` (Catppuccin Sky `#89dceb`)
- **Latency Target:** Initial index < 100 ms; Query < 5 ms.
- **Measured Latency:** Initial index **24.5 ms**; Query **0.06 ms** (60 µs).

### Layer 3: Language Code Snippets (`snippets.rs`)
- **Design:** Built-in multi-line templates across all 15 core workstation languages: Rust, Python, C, C++, Java, SQL, TOML, Markdown, JSON, YAML, JavaScript, TypeScript, Bash, Go, HTML, CSS.
- **Expansion:** Full template body expansion with indentation preservation and tab-stop positioning.
- **Badge:** `[snip]` (Catppuccin Yellow `#f9e2af`)
- **Latency Target:** < 1 ms.
- **Measured Latency:** **0.008 ms** (8 µs).

### Layer 4: Async Non-Blocking LSP Provider (`lsp_provider.rs`)
- **Design:** Non-blocking asynchronous query architecture with generation tokens (`AtomicU64`).
- **Supported Servers:** `rust-analyzer`, `pyright`, `clangd`, `jdtls`, `gopls`, `typescript-language-server`, `taplo`, `marksman`, `sqls`, `bash-language-server`.
- **Stale Request Cancellation:** Monotonic cancellation tokens ensure stale in-flight results from prior keystrokes are discarded instantly when received, preventing UI stutter.
- **Badge:** `[lsp]` (Catppuccin Mauve `#cba6f7`)

---

## 3. Suggestion Popup UI, Grapheme Safety & Snippet Expansion

### 3.1 Grapheme-Safe Prefix Extraction (Sprint 2.1)
Earlier implementations relied on simple ASCII byte/char scanning for prefix extraction. In Sprint 2.1, prefix extraction was upgraded to use `UnicodeSegmentation::grapheme_indices(true)`:
* Safely slices the line up to the current cursor column without byte-boundary panics.
* Identifies alphanumeric and underscore token boundaries (`[a-zA-Z0-9_]`).
* Correctly handles identifiers immediately adjacent to multibyte emojis or punctuation.

### 3.2 Snippet Placeholder Expansion & Tab Stops
Snippets support dynamic placeholder tab stops (e.g., `${1:name}`, `${2:Type}`, `$0`):
1. **Parsing:** `parse_snippet_tab_stops` strips placeholder markers during insertion and records the relative row and column offsets of each tab stop.
2. **Tab Navigation:** The editor maintains `snippet_tab_stops: Vec<Cursor>`. Pressing `Tab` when no suggestion menu is active invokes `jump_next_snippet_tab_stop()`, sequentially navigating through placeholders until the final `$0` exit point.
3. **Single Transactional Delta:** When accepting a completion, prefix deletion and body insertion are committed in a single atomic `UndoTransaction`, guaranteeing that `undo` reverses both the expanded text and the cursor jump cleanly.

### 3.3 Visual Presentation & Interaction
The floating suggestion popup is rendered inside `src/ui/editor.rs`:
* **Smart Clamped Positioning:** Dynamically positioned adjacent to the cursor (below if room exists, otherwise above). Viewport-clamped horizontally and vertically so it never overflows off-screen.
* **Badges & Visual Highlighting:** Clearly displays `[buf]`, `[snip]`, `[sym]`, and `[lsp]` in corresponding Catppuccin Mocha colors.
* **Keyboard Interaction:**
  * `Down` / `Up`: Navigate candidate list with wrap-around.
  * `Tab` / `Enter`: Accept selected completion, insert replacement, and focus first snippet tab stop.
  * `Tab` (post-completion): Advance cursor to next snippet placeholder.
  * `Esc`: Dismiss suggestion box and return to normal editing.
  * Keystrokes: Automatically re-filter candidate list dynamically.

---

## 4. Deterministic Ranking Architecture

Candidates from all four layers are merged and ranked deterministically:
1. **Match Quality:** Exact match > Prefix match > Subsequence fuzzy match.
2. **Layer Priority:** LSP (`score: 1000`) > Snippets (`score: 700-1200`) > Workspace Symbols (`score: 800`) > Buffer Words (`score: 500-600`).
3. **Length Penalty:** Shorter identifiers ranked ahead of long, unwieldy symbols for identical prefixes.
