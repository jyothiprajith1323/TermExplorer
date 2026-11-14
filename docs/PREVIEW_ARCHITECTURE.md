# TermExplorer: Universal Preview Architecture

## 1. Executive Summary

TermExplorer Sprint 3 establishes an asynchronous, bounded, zero-leak universal file inspection engine designed specifically for terminal workstation environments. Occupying the primary 75% preview and inspector panel, this engine dynamically inspects, decodes, and presents rich textual, structured data, multimedia, and binary formats while strictly preserving sub-30ms responsiveness and $O(1)$ memory bounds.

```
+-----------------------------------------------------------------------------+
|                             TermExplorer Preview Engine                     |
+-----------------------------------------------------------------------------+
                                       |
                   [5-Layer Format Detection & MIME Classifier]
                 (Magic Bytes -> ZIP Disambiguation -> XML/SVG)
                                       |
             +-------------------------+-------------------------+
             |                                                   |
     [Textual & Code]                                   [Multimedia & Data]
  - Source Code (Syntect)                            - Images & SVG (resvg/tiny-skia)
  - Markdown AST (pulldown-cmark)                    - Audio & Metadata (lofty)
  - Virtual Hex Inspector                            - Video Keyframes & Metainfo
  - CSV / TSV Interactive Table                      - PDF & Office Documents
  - JSON / YAML Hierarchy Tree                       - SQLite Database (rusqlite)
                                                     - Hardened Archives (Zip-Slip safe)
             |                                                   |
             +-------------------------+-------------------------+
                                       |
                      [Two-Tier Cache & Memory Bounding]
                      (In-Memory LRU + BLAKE3 Disk Cache)
                                       |
                   [Monotonic Cancellation & Reactor Bridge]
```

---

## 2. Five-Layer Format Detector

File inspection begins with a bounded 8 KiB detection engine in `src/preview/detector.rs` that categorizes paths into 11 distinct preview handler types:

1. **Layer 1: Definitive Magic Bytes**
   - Executable Binaries: ELF (`7F 45 4C 46`), Mach-O (`FE ED FA CE`), PE/COFF (`4D 5A`), WASM (`00 61 73 6D`).
   - SQLite 3: `53 51 4C 69 74 65 20 66 6F 72 6D 61 74 20 33 00`.
   - Documents & Media: PDF (`%PDF-`), PNG, JPEG, GIF, WebP, BMP, TIFF, Matroska/MKV/WebM, MP4, AVI, MP3 (ID3/Sync), WAV, FLAC, OGG.
   - Archives: GZIP, BZIP2, XZ, 7Z, TAR.

2. **Layer 2: ZIP Container Disambiguation**
   Inspects central directory records within ZIP containers to disambiguate generic archives from modern Office Open XML and OpenDocument formats:
   - Microsoft Word (`word/document.xml`) -> `PreviewKind::Office`
   - Microsoft Excel (`xl/workbook.xml`) -> `PreviewKind::Office`
   - Microsoft PowerPoint (`ppt/presentation.xml`) -> `PreviewKind::Office`
   - OpenDocument Text (`mimetype: application/vnd.oasis.opendocument.text`) -> `PreviewKind::Office`
   - OpenDocument Spreadsheet (`application/vnd.oasis.opendocument.spreadsheet`) -> `PreviewKind::Office`
   - Default ZIP -> `PreviewKind::Archive`

3. **Layer 3: XML & Vector Graphics Inspection**
   Scans the first 4 KiB for `<svg` tags or `http://www.w3.org/2000/svg` namespaces to route vector graphics directly to the `resvg` in-process rasterizer.

4. **Layer 4: Extension & MIME Guessing**
   Leverages `mime_guess` and extension rules for clean categorization of JSON, YAML, TOML, CSV, TSV, Markdown, and source code files.

5. **Layer 5: Null-Byte Binary Heuristic**
   Scans the initial 8 KiB buffer for null bytes (`0x00`). If non-text binary bytes exceed 0.5% of total sample length, the file is safely routed to the virtualized `HexPreview` rather than risking terminal corruption with raw binary printing.

---

## 3. Asynchronous Monotonic Cancellation

Terminal file navigation is inherently rapid. Moving across directory entries via arrow keys or search result navigation generates dozens of preview requests per second.

- **Atomic Generation Advance**: Each navigation event increments an atomic sequence counter (`preview_ticket` / `cancellation.advance()`).
- **Reactor Token Verification**: Background parsing threads continuously check `token.load(Ordering::Relaxed) == ticket` at parsing boundaries (e.g. before syntax highlighting, image scaling, or database reading).
- **Instant Abort**: Outdated preview jobs abort in under 2 ms, freeing CPU and memory for the currently selected file.

---

## 4. Subprocess Isolation and Security Boundaries

TermExplorer adheres to zero-risk execution policies:
- **No Direct Shell Invocations**: All subprocess calls (e.g., `xdg-open`, `ffprobe`, `pdftoppm`) pass argument vectors directly without invoking `/bin/sh` or string interpolation, eliminating shell injection vulnerabilities.
- **Pure-Rust Priority**: SVG rasterization (`resvg`), audio tagging (`lofty`), SQLite querying (`rusqlite`), and Markdown parsing (`pulldown-cmark`) execute 100% in-process without relying on external system binaries.
- **Subprocess Timeouts**: External calls are wrapped with strict execution timeouts (typically 2.0s) to prevent runaway hanging processes from locking terminal state.
