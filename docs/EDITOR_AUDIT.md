# TermExplorer — Editor, Completion & Preview Architecture Audit

**Date:** 2026-11-08  
**Engineer:** Principal Rust Systems Engineer & Text-Editor Architect  
**Scope:** In-depth audit of text buffer, syntax highlighting, autocompletion engine, workspace indexer, LSP integration, keyboard/mouse event handling, Unicode/emoji width calculations, and preview rendering.

---

## Executive Summary

TermExplorer includes an embedded editor, a preview panel with multi-format detectors, and a multi-layer autocomplete engine. However, our comprehensive audit reveals critical architectural limitations that prevent it from being a reliable daily-driver development environment:

1. **Text Buffer Architecture (`src/editor/buffer.rs`):**
   - The buffer currently uses `lines: Vec<String>` with full-vector snapshot undo/redo. On every single keystroke, `lines.clone()` duplicates the entire document in memory. For a 10,000-line file, inserting 100 characters triggers over 1,000,000 string allocations.
   - Slicing and indexing rely on byte offsets rather than Unicode grapheme clusters or char indices. Inserting characters into or backspacing over multi-byte UTF-8 sequences (e.g. emojis `🦀`, CJK characters, accented vowels `é`) causes byte-boundary panics or string corruption (`split_at` and `insert` panicking on non-char boundaries).
   - Search is limited to naive substring matching with no replace capability, regex, or selection replace.

2. **Syntax Highlighting & Rendering (`src/editor/syntax.rs` & `src/ui/editor.rs`):**
   - While `src/editor/syntax.rs` initializes `syntect`, `src/ui/editor.rs` **never actually invokes it**! Lines are rendered as `Span::raw(display_text)` with zero syntax coloring in the actual editor.
   - Syntect highlighting in previews re-parses from the start of lines without token caching or incremental invalidation.
   - Only a small subset of file extensions are recognized, lacking modern syntax mappings for 20+ required languages (SQL, Go, Bash, HTML, CSS, XML, YAML, Dockerfile, TypeScript, etc.).

3. **Autocompletion & LSP Architecture (`src/editor/lsp.rs`, `src/completion/`):**
   - `src/editor/lsp.rs` is a 52-line stub that merely checks `which` on the system PATH. It does not spawn language servers, initiate JSON-RPC communication, or request diagnostics/completions.
   - `src/completion/editor.rs` contains simulated hooks but lacks true asynchronous LSP lifecycle management, request timeouts, and process communication.
   - Snippet expansion in `Buffer::accept_completion` inserts only the short label rather than expanding multi-line snippet bodies with tabstops.
   - No manual trigger shortcut (`Ctrl+Space`) exists in the editor keymap.

4. **Unicode, Emoji & Display Width (`src/ui/`, `src/preview/`):**
   - Ratatui cursor and text alignment assume 1 byte = 1 char = 1 cell column. Multi-column emojis (e.g., `🚀`, `📦`, `🎨`) occupy 2 terminal columns but are treated as length 1 or 4, causing cursor misalignments and ghost characters.
   - Missing fallback mechanisms for Nerd Fonts: terminal emulators without Nerd Fonts render missing glyphs as empty tofu blocks (``, `□`, or whitespace).
   - In Markdown preview (`src/preview/markdown/mod.rs`), headings are hardcoded with block characters `█`, `▓▓`, `▒▒▒`, which appear as jarring black/gray rectangles across various terminal fonts.

5. **Preview Scrolling & Mouse Event Routing (`src/app.rs`, `src/ui/preview.rs`):**
   - Mouse wheel down on Preview (`MouseEventKind::ScrollDown`) increments `scroll_offset` without clamping to `total_lines`. As a result, scrolling down with a wheel sends `scroll_offset` into thousands, permanently clearing the preview viewport until manually reset.
   - Mouse wheel scrolling only affects the focused panel rather than the panel under the mouse cursor, frustrating users trying to scroll preview while explorer is active.
   - Horizontal scrolling is absent in the editor and preview.

---

## Detailed Component Audit Matrix

| Component | Current Implementation | Flaws & Inefficiencies | Required Upgrade |
|---|---|---|---|
| **Text Buffer** | `Vec<String>` line vector | `lines.clone()` on every insert; byte-offset slicing panics on multi-byte UTF-8 | Transition to Rope-backed buffer (`ropey`) with cached line access and delta undo/redo |
| **Cursor System** | `Cursor { row, col }` byte-based | Cursor column does not account for multi-byte UTF-8 or double-width cells | Grapheme/char-aware cursor with column visual cell mapping |
| **Undo / Redo** | `Vec<(Vec<String>, Cursor)>` | Duplicates entire document on every keystroke (O(N * M) memory overhead) | Delta / operational history (InsertText, DeleteRange) |
| **Syntax Highlighting** | Syntect loaded but unused in editor UI | Editor renders plain unstyled text; preview lacks caching | Full syntect highlighter with Catppuccin Mocha theme in editor UI |
| **Language Support** | ~4 languages (rs, py, c, java) | Missing SQL, Go, TS, Bash, HTML, CSS, JSON, YAML, TOML, Markdown, etc. | 20+ language definitions with fallback detection |
| **Autocomplete Engine** | Monolithic `src/completion/editor.rs` | Mixed concerns, hardcoded snippets, simulated LSP | Modular 4-layer engine: Engine, Provider, BufferWords, WorkspaceSymbols, Snippets, LSP |
| **LSP Integration** | PATH check stub (`which`) | No real LSP protocol, no async communication, no server process lifecycle | Non-blocking JSON-RPC LSP client with cancellation tokens and fallback |
| **Snippet Expander** | Inserts `label` only | Multi-line bodies ignored, no indentation preservation | Full snippet insertion with auto-indent and placeholder replacement |
| **Markdown Preview** | Hardcoded `█`, `▓▓`, `▒▒▒` headings | Causes ugly block artifacts in terminals | Clean styled headings (`# H1`, `## H2`) with theme colors and borders |
| **Preview Scrolling** | Unbounded mouse scroll offset | Scrolling past EOF leaves screen completely blank; no hover routing | Clamped scroll offsets, mouse hover routing, PageUp/PageDown, Home/End |
| **Unicode / Emoji** | Raw byte lengths | Panics on UTF-8 boundaries, cursor misplacement on double-width glyphs | `unicode-width` and `unicode-segmentation` throughout |

---

## Action Plan & Phased Implementation

1. **Phase 2 — High-Performance Text Buffer:**
   - Integrate `ropey = "1.6"` for rope data structures.
   - Maintain API compatibility with existing test suite while optimizing internal storage.
   - Implement delta-based undo/redo stack.
   - Support atomic writes, external modification detection, and deterministic crash recovery snapshots.

2. **Phase 3 — Syntax Highlighting & Theme Engine:**
   - Hook syntax highlighter into `render_editor`.
   - Implement Catppuccin Mocha color mappings (keywords, types, functions, strings, comments, numbers, operators).
   - Support 20+ programming, markup, and configuration languages.

3. **Phase 4 & 5 — 4-Layer Completion Engine & Popup UI:**
   - Refactor `src/completion/` into modular providers: `buffer_words`, `workspace_symbols`, `snippets`, `lsp_provider`, `ranking`, `cache`, `cancellation`.
   - Provide rich snippets for Rust, Python, C/C++, Java, SQL, TOML, Markdown, Go, Bash, HTML, CSS, JavaScript/TypeScript.
   - Implement asynchronous LSP client support with stale cancellation.
   - Enhance editor completion popup with source badges (`[LSP]`, `[Snip]`, `[Sym]`, `[Buf]`), documentation preview, and `Ctrl+Space` trigger.

4. **Phase 6 — Language Configurations:**
   - Language-specific indentation (spaces vs tabs, width 2 vs 4).
   - Smart auto-indent on newline.
   - Comment toggling (`Ctrl+/`).

5. **Phase 7 & 8 — Unicode & Preview Scrolling Fixes:**
   - Eliminate hardcoded `█`, `▓▓`, `▒▒▒` blocks in Markdown preview.
   - Fix mouse wheel scroll clamping and hover-based event dispatch.
   - Ensure grapheme-cluster safety across all text operations.

6. **Phase 9-14 — Comprehensive Verification:**
   - Add new test suites: `tests/unicode_and_emoji_tests.rs`, `tests/editor_integration_tests.rs`.
   - Ensure all existing 114 tests pass without regressions.
   - Complete strict compiler, Clippy (`-D warnings`), and release build validation.
