# TermExplorer V1.1.0 — Intelligent Autocomplete & Suggestions Engine

**Module:** `src/completion/`  
**Architecture:** 4-Layer Hybrid Autocomplete Engine  

---

## 1. Modular Architecture Overview

The TermExplorer completion engine unifies file explorer navigation, command execution, and code editing under a high-performance modular suggestion architecture.

```mermaid
flowchart TD
    subgraph Engine ["Central Completion Engine (`src/completion/engine.rs`)"]
        L1["Layer 1: Buffer Words (`buffer_words.rs`) (< 0.2 ms)"]
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
- **Design:** Scans active editor buffer lines for word identifiers (`[a-zA-Z0-9_]`).
- **Optimization:** Zero-allocation ASCII comparison using `as_bytes().get(..prefix_len).eq_ignore_ascii_case()`. Early break after collecting 4x candidate ceiling.
- **Badge:** `[buf]` (Catppuccin Blue `#89b4fa`)
- **Latency Target:** < 20 ms
- **Measured Latency:** **0.12 ms** (120 µs) on 2,000 lines.

### Layer 2: Workspace Symbol Indexer (`workspace_symbols.rs`)
- **Design:** Scans workspace directory tree in background thread or cache.
- **Ignored Directories:** `.git`, `node_modules`, `target`, `dist`, `build`, `.cargo`, `.cache`, `.vscode`, `vendor`, `.venv`, `__pycache__`, `bin`, `obj`.
- **Supported Languages:** Rust (`fn`, `struct`, `enum`, `trait`), Python (`def`, `class`), C/C++ (`struct`, `typedef`, `class`), Java (`class`, `interface`), Go (`func`), TypeScript/JavaScript (`type`, `interface`, `class`), SQL, TOML, Bash.
- **Badge:** `[sym]` (Catppuccin Sky `#89dceb`)
- **Latency Target:** Initial index < 100 ms; Query < 5 ms.
- **Measured Latency:** Initial index **24.5 ms**; Query **0.06 ms** (60 µs).

### Layer 3: Language Code Snippets (`snippets.rs`)
- **Design:** Built-in templates for Rust, Python, C/C++, Java, SQL, TOML, Markdown, Go, Bash, HTML, CSS, JavaScript/TypeScript.
- **Expansion:** Full template body expansion with indentation preservation.
- **Badge:** `[snip]` (Catppuccin Yellow `#f9e2af`)
- **Latency Target:** < 1 ms.
- **Measured Latency:** **0.008 ms** (8 µs).

### Layer 4: Async Non-Blocking LSP Provider (`lsp_provider.rs`)
- **Design:** Asynchronous architecture using monotonic request IDs (`AtomicU64`).
- **Supported Servers:** `rust-analyzer`, `pyright`, `clangd`, `jdtls`, `gopls`, `typescript-language-server`, `taplo`, `marksman`, `sqls`, `bash-language-server`.
- **Stale Request Cancellation:** If the user continues typing while a request is pending, the request ID increments. When old results arrive, their ID no longer matches `current_request_id`, immediately dropping stale candidates without UI flicker.
- **Badge:** `[lsp]` (Catppuccin Mauve `#cba6f7`)

---

## 3. Path & Location Autocompletion (`explorer.rs`)

- **Tilde Resolution:** Automatically expands `~/...` to the user's home directory using `shellexpand::tilde`.
- **Directory Formatting:** Appends `/` to directories so pressing Tab repeatedly walks through directory trees seamlessly (`src/` -> `src/ui/` -> `src/ui/layout.rs`).
- **Latency Target:** < 50 ms.
- **Measured Latency:** **0.17 ms** (171 µs).

---

## 4. Command Palette Ranking (`palette.rs`)

- **Composite Scoring Formula:**
  $$\text{Score} = \text{BaseMatch} + \text{PrefixBonus} + \text{AcronymBonus} + \text{ContextBonus}$$
  - Exact Title Match: **+2000**
  - Prefix Match: **+1200**
  - Acronym Match (e.g., `tdp`): **+900**
  - Subsequence Fuzzy Match: **+400**
  - Active Panel Context Bonus: **+40**
- **Latency Target:** < 10 ms.
- **Measured Latency:** **0.84 ms** (842 µs).
