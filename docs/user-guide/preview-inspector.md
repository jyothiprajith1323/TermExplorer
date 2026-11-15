# Contextual Preview & Inspector Engine

The Contextual Preview panel occupies the right **75%** of the workstation screen by default, providing instantaneous, deep inspection across code, documents, media, structured data, and binary files.

---

## 1. Supported Preview Formats & Providers

The table below reflects the actual implementations in `src/preview/`:

| Format / Category | Extensions & Detection | Supported Behavior | Requirements & Limitations |
|---|---|---|---|
| **Source Code** | `.rs`, `.py`, `.js`, `.ts`, `.c`, `.cpp`, `.go`, `.html`, `.css`, etc. | Full syntax highlighting via `syntect` with line numbers and token caching. | Viewport-bounded tokenization; files >10 MB are truncated to first 1,000 lines. |
| **Markdown** | `.md`, `.markdown` | Formatted headers, bullet lists, blockquotes, code blocks, checklists, and auto-generated Table of Contents. | Rendered via `pulldown-cmark`. Complex HTML tables inside markdown are rendered as preformatted text. |
| **Images** | `.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`, `.bmp`, `.ico` | High-resolution 24-bit TrueColor rendering using Unicode half-block characters (`▀`), image dimensions, and aspect ratio calculation. | Requires TrueColor terminal emulator. Animated GIFs render the first frame. |
| **Tabular Data** | `.csv`, `.tsv` | Automatic delimiter detection, aligned column layout, zebra striping, and row counters. | Evaluated via `csv` crate; memory reads capped at first 5,000 rows. |
| **Structured JSON/YAML** | `.json`, `.yaml`, `.yml` | Pretty-printed hierarchical syntax tree, key/value colorization, data type indicators, and syntax error alerts. | Parsed via `serde_json` and `serde_yaml`. |
| **SQLite 3 Databases** | `.sqlite`, `.sqlite3`, `.db` | Decodes header metadata (page size, change counter), lists all tables via `sqlite_master`, and samples rows. | Pure-Rust header inspection + optional `sqlite3` CLI tool for advanced queries. |
| **PDF Documents** | `.pdf` | Header inspection (page count, title, author) and formatted text extraction. | Uses `pdftotext` (`poppler-utils`) if installed; falls back to pure-Rust stream extractor. |
| **Audio Media** | `.mp3`, `.flac`, `.ogg`, `.wav`, `.m4a` | ID3/Vorbis tag inspection (title, artist, album, year), bitrate, sample rate, channels, and duration. | Uses pure-Rust `lofty` crate; no external audio player required. |
| **Video Media** | `.mp4`, `.mkv`, `.webm`, `.avi`, `.mov` | Container metadata (resolution, frame rate, audio/video codecs, duration) and first-frame image thumbnail. | First-frame thumbnail extraction requires `ffmpeg` in `$PATH`. |
| **Office Documents** | `.docx`, `.xlsx`, `.pptx` | Pure-Rust XML parsing: extracts `.docx` paragraphs, `.xlsx` spreadsheet rows/cells, and `.pptx` slide counts. | No LibreOffice or external dependencies required. Password-protected files not supported. |
| **Archives** | `.zip`, `.tar`, `.tar.gz`, `.tar.bz2`, `.tar.xz` | Nested archive table listing compressed files, original sizes, compression ratios, and safety checks against path traversal (`..`). | Pure-Rust decoding via `zip` and `flate2`. |
| **Canonical Hex** | Any binary / unknown format | 16-byte memory offset, hex byte pairs, and ASCII sidebar. | Displayed automatically when a file fails UTF-8 validation. |

---

## 2. The 4 Inspector Tabs

The inspector pane features four tabs accessible via hotkeys `1` through `4`:

1. **`[1] Preview`**: The primary contextual rendering (syntax highlighted code, Markdown view, pixel image, tabular data).
2. **`[2] Information`**: Filesystem metadata:
   - Absolute canonical path
   - Exact size in bytes and human-readable format
   - POSIX file mode permissions (`rwxr-xr-x` / `0755`)
   - File owner and group names
   - Creation, modification, and access timestamps
   - Cryptographic SHA-256 and BLAKE3 checksums (calculated asynchronously)
   - MIME classification
3. **`[3] Hex / Metadata`**: Low-level hexadecimal byte dump showing memory offsets and printable characters.
4. **`[4] Actions`**: Context-sensitive operations based on file type (e.g. "Extract Archive", "Run Script", "Open in External Editor"). Trigger actions using keys `1`–`9`.

---

## 3. Caching and Performance Limits

- **Memory LRU Cache (`PreviewLruCache`)**: Retains up to 100 recent preview artifacts in memory (capped at 64 MB). Navigating between previously inspected files is instantaneous (<1 ms).
- **Disk Cache (`DiskPreviewCache`)**: Persistent disk cache in `~/.cache/termexplorer/previews/` for expensive assets such as video frame extractions and PDF text conversions.
- **Monotonic Cancellation**: If you rapidly scroll down a list of 500 files, in-flight preview generation jobs are cancelled automatically so only the currently selected file consumes CPU resources.
