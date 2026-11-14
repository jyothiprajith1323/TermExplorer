# TermExplorer — Sprint 3 Implementation Audit
## Universal Preview Engine, Media Inspection & Unified Search

**Date:** 2026-11-14  
**Sprint:** Engineering Sprint 3  
**Status:** Audit Completed — Implementation Commenced  
**Author:** Principal Rust Systems Engineer & Architecture Specialist  

---

## 1. Executive Summary

This audit assesses the state of TermExplorer's file preview infrastructure, media inspection capabilities, and search engine against the requirements of the *TermExplorer Product Strategy and Systems Architecture Roadmap*.

The objective of Sprint 3 is to elevate the primary **75% Preview/Inspector panel** into an industry-grade, universal workstation inspection environment capable of safely, rapidly, and natively inspecting source code, documentation, images, vector graphics, video frames, audio metadata, PDFs, office documents, structured data (CSV/TSV/JSON/YAML), SQLite databases, compressed archives, and raw binary formats. Concurrently, a unified search engine is required to unify workspace fuzzy filename discovery and asynchronous live content grep searching with live preview integration.

---

## 2. Component-by-Component Capability Audit

Every preview and search capability has been audited directly against the codebase and categorized into one of five states:
- **COMPLETE:** Fully implemented, verified with tests, and meets all quality/performance targets.
- **PARTIAL:** Basic implementation exists, but missing critical formats, edge-case handling, cancellation, or interactive controls.
- **BROKEN:** Implemented but failing invariants, crashing on edge cases, or introducing regressions.
- **MISSING:** Not implemented in the codebase; needs to be built from scratch.
- **BLOCKED:** Dependent on an external unfulfilled architectural prerequisite.

| Subsystem / Feature | Current State | Audit Findings & Gap Analysis | Action Plan for Sprint 3 |
|---|---|---|---|
| **Universal Preview Architecture** | **PARTIAL** | Hardcoded `match format_info.category` inside `PreviewManager`. Lacks a clean, extensible, trait-based `PreviewHandler` abstraction with capability reporting and resource deallocation. | Build modular `src/preview/registry.rs`, `src/preview/capabilities.rs`, and modular format handlers under `src/preview/`. |
| **Layered Format Detection** | **PARTIAL** | Basic extension and short magic-byte checks. Missing 8 KiB bounded header sampling, MIME metadata integration, misleading filename detection (e.g. ZIP named `.txt`), and user override associations. | Implement robust layered detector (`detector.rs`) inspecting magic bytes (ELF, ZIP, SQLite, PDF, PNG, JPEG, etc.), MIME, and extensions. |
| **Image Preview (Bitmaps)** | **PARTIAL** | Reads dimensions and renders half-block truecolor cells via `image` crate. Lacks zoom in/out, pan, reset, fit-to-panel vs original-size, and EXIF extraction. | Add zoom/pan state, original vs fit toggle, EXIF/color-space metadata, and Kitty/Sixel graphics capability detection. |
| **Vector Graphics (SVG)** | **PARTIAL** | Only parses width/height via regex on XML attributes. Zero visual preview or rasterization. | Implement safe in-process SVG rasterization using pure-Rust stack (`resvg`/`tiny-skia`), disabling external references and scripts. |
| **Terminal Graphics Protocols** | **MISSING** | No Kitty graphics protocol, Sixel, or iTerm2 inline image support. Relies entirely on half-block cells. | Implement terminal graphics detection (Kitty `_Gi=`, Sixel device attribute queries, iTerm2 OSC 1337) with clean half-block fallback. |
| **Video Preview** | **PARTIAL** | Synchronous `ffprobe`/`ffmpeg` shell-out for 1 thumbnail. UI blocks if FFmpeg hangs. No multiple frame extraction, no timestamp seeking, no process timeout/cancellation. | Implement safe asynchronous process invocation with 500ms timeouts, multi-frame extraction (0%, 25%, 50%, 75%), duration/codec display, and mpv playback. |
| **Audio Preview** | **PARTIAL** | Basic `ffprobe` metadata extraction. Requires external tools; no pure-Rust metadata parsing, no embedded album art extraction, no playback controls. | Integrate pure-Rust metadata parsing (`lofty`), extract embedded ID3/Vorbis cover art for half-block rendering, and provide interactive playback actions. |
| **Markdown Rendering & Viewport** | **PARTIAL** | Naive line-by-line `#` and ``` parser. No inline formatting (bold/italic/code), no tables, no syntax-highlighted code blocks, no search within doc. | Build AST-based renderer (`pulldown-cmark`) with Syntect code block highlighting, markdown table formatting, TOC navigation, and scrollable viewport. |
| **PDF Preview** | **PARTIAL** | Naive regex on PDF header + optional `pdftotext` stream. No page navigation, no rendered page thumbnail, no search in extracted text. | Implement page-by-page text navigation, page count extraction, optional `pdftoppm` thumbnail generation, and text search. |
| **Office Documents (DOCX/XLSX/PPTX)** | **PARTIAL** | Naive ZIP reading of `word/document.xml`. No spreadsheet cell grid, no slide titles for PPTX, no ODT/ODS support. | Enhance DOCX text/table extraction, implement XLSX sheet & cell grid viewer with row/col limits, and PPTX slide title extraction. |
| **CSV / TSV Data Inspector** | **PARTIAL** | Basic delimiter sniffing and static table. No column navigation, no horizontal scrolling, no sorting, no regex row filter, no cell copying. | Build interactive spreadsheet-like data inspector with horizontal/vertical scrolling, column sorting, regex filtering, and cell/row clipboard copy. |
| **JSON & YAML Inspector** | **PARTIAL** | Only pretty-prints text. No collapsible object/array tree, no YAML support, no jq-style key path copying, no key/value search. | Implement collapsible hierarchical tree viewer, YAML support, key/value search, and JSON path (`.users[0].name`) clipboard copy. |
| **SQLite Database Inspector** | **PARTIAL** | Header validation + external `sqlite3` CLI command. Fails completely if `sqlite3` CLI is missing. | Integrate pure-Rust read-only SQLite inspection (`rusqlite`) for zero-dependency table list, schema, column types, row previews, and read-only queries. |
| **Archive Inspector & Security** | **PARTIAL** | Basic ZIP and TAR list. Only basic `..` check. Missing TAR.GZ, BZIP2, XZ, 7Z, archive bomb protection, and nested text preview. | Implement TAR.GZ/BZIP2/XZ support, strict archive bomb decompression limits (max 500 MB ratio 100:1), and inner file text previews. |
| **Binary & Hex Viewer** | **PARTIAL** | Static 16-byte hex dump of first 100 rows. No arbitrary offset seek, no hex pattern search, no ELF/PE/Mach-O header breakdown. | Build virtualized hex viewer with arbitrary offset navigation, byte pattern search, and executable header detection (ELF, PE, Mach-O, WASM). |
| **Unified Search Engine** | **MISSING** | `src/filesystem/search.rs` is merely a 33-line in-memory filter. No workspace-wide fuzzy file search or live content search. | Architect `src/search/` with asynchronous parallel file walker (`ignore`), live streaming ripgrep-style content search, and instant preview jump. |
| **Search Result Preview Integration**| **MISSING** | No connection between search matches and the 75% preview pane. | Highlight matching search results in the preview/editor pane with exact line and column coordinates. |
| **Search History & Persistence** | **MISSING** | No persistent query history. | Implement XDG state search history persistence (`~/.local/state/termexplorer/search_history.json`). |
| **Preview Cache & Resource Management**| **PARTIAL** | In-memory LRU cache of 64 entries. No disk caching of rendered thumbnails/SVGs, no BLAKE3/SHA-256 fingerprinting. | Implement bounded XDG disk cache (`~/.cache/termexplorer/`) with two-stage mtime/BLAKE3 verification and atomic writes. |
| **Inspector UI & Keyboard Focus** | **PARTIAL** | Basic tab pills and scrolling. Viewport doesn't support fullscreen toggle, search in preview, or dedicated interactive mode keys. | Add fullscreen toggle (`f`), search within preview (`/`), interactive inspector controls, and responsive error/empty states. |

---

## 3. Implementation Phasing Strategy

To deliver a rock-solid, production-grade universal preview and search environment without destabilizing existing functionality, Sprint 3 will be executed across five focused engineering phases:

1. **Phase 1: Universal Preview Core & Modular Registry**
   - Refactor `src/preview/` into a trait-driven architecture (`PreviewHandler`).
   - Layered format detection with 8 KiB magic byte inspection, MIME, and misleading extension handling.
   - Bounded XDG disk and memory cache with BLAKE3 fingerprinting.
   - Monotonic cancellation tokens.

2. **Phase 2: Multimedia & Vector Inspection**
   - In-process safe SVG rasterization (`resvg`/`tiny-skia`).
   - Image inspection with Zoom, Pan, Fit vs Original, and Kitty/Sixel capability detection.
   - Video multi-frame extraction with safe process timeouts and mpv playback.
   - Pure-Rust audio metadata extraction (`lofty`) and embedded artwork preview.

3. **Phase 3: Documents & Structured Data**
   - High-fidelity Markdown rendering (`pulldown-cmark`) with Syntect code syntax highlighting and TOC.
   - PDF page navigation, metadata, and optional thumbnail rendering.
   - Office document text & table extraction (DOCX, XLSX, PPTX).
   - Interactive CSV/TSV table inspector with column sorting, filtering, and cell copy.
   - Collapsible JSON/YAML hierarchical tree viewer with jq path copying.
   - Pure-Rust SQLite read-only inspector (`rusqlite`) with schema, table browser, and query execution.

4. **Phase 4: Archive Security & Hex Inspection**
   - Multi-format archive viewer (ZIP, TAR, GZ, BZ2, XZ) with strict bomb & traversal protections.
   - Virtualized binary hex inspector with offset seeking and byte search.

5. **Phase 5: Unified Workspace Search Engine**
   - High-performance parallel fuzzy file search (`ignore`, `fuzzy-matcher`).
   - Live asynchronous content search (regex, case sensitivity, .gitignore respect, streaming results).
   - Direct search match preview with line-number jumping into the editor.
   - Persistent search history in XDG state directory.
   - Integration into workstation UI (`Ctrl+P` file search, `Ctrl+Shift+F` live content search).
