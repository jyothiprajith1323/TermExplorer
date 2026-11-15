# Supported File Formats & Detection Signatures

TermExplorer uses magic byte detection and extension heuristics to classify and render file contents.

---

## 1. File Format Capabilities Matrix

| Category | File Extensions | Magic Bytes / Header | Preview Capabilities | External Tool |
|---|---|---|---|---|
| **Rust Source** | `.rs` | UTF-8 Text | Syntax highlighting, line numbers, autocomplete, LSP | None (Pure Rust) |
| **Python** | `.py` | UTF-8 Text | Syntax highlighting, line numbers, auto-indent | None |
| **JavaScript / TypeScript** | `.js`, `.ts`, `.jsx`, `.tsx` | UTF-8 Text | Syntax highlighting, JSON tokenization | None |
| **C / C++** | `.c`, `.cpp`, `.h`, `.hpp` | UTF-8 Text | Syntax highlighting, line numbers | None |
| **Go** | `.go` | UTF-8 Text | Syntax highlighting, line numbers | None |
| **HTML / CSS** | `.html`, `.css`, `.scss` | UTF-8 Text | Syntax highlighting, tag matching | None |
| **Markdown** | `.md`, `.markdown` | UTF-8 Text | Blockquotes, code fences, headers, table of contents | None (`pulldown-cmark`) |
| **JSON** | `.json` | `{`, `[` | Tree view, key/value colorization, syntax checking | None (`serde_json`) |
| **YAML** | `.yaml`, `.yml` | UTF-8 Text | Structured data tree, data types | None (`serde_yaml`) |
| **CSV / TSV** | `.csv`, `.tsv` | Text with `,` or `\t` | Table view with zebra striping, aligned columns | None (`csv`) |
| **SQLite 3** | `.sqlite`, `.sqlite3`, `.db` | `SQLite format 3\0` | Header metadata, table lists, sample row viewer | Optional `sqlite3` CLI |
| **Images** | `.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`, `.bmp`, `.ico` | `\x89PNG`, `\xff\xd8\xff`, `GIF89a` | 24-bit TrueColor pixel halfblocks (`▀`), zoom, fit | None (`image` crate) |
| **SVG** | `.svg` | `<svg` | XML structure viewer, dimensions, element counts | None |
| **PDF** | `.pdf` | `%PDF-` | Page count, document metadata, extracted text | Optional `pdftotext` |
| **Audio** | `.mp3`, `.flac`, `.ogg`, `.wav`, `.m4a` | `ID3`, `fLaC`, `OggS`, `RIFF` | Title, artist, album, bitrate, duration, channels | None (`lofty`) |
| **Video** | `.mp4`, `.mkv`, `.webm`, `.avi` | `ftyp`, `\x1a\x45\xdf\xa3` | Container metadata, resolution, first-frame image | Optional `ffmpeg` |
| **Office DOCX** | `.docx` | `PK\x03\x04` | Paragraph text extraction, word counts | None (Pure Rust) |
| **Office XLSX** | `.xlsx` | `PK\x03\x04` | Sheet list, row/cell table extraction | None (Pure Rust) |
| **Archives** | `.zip`, `.tar`, `.tar.gz`, `.tar.bz2`, `.tar.xz` | `PK\x03\x04`, `\x1f\x8b` | Nested file tree, uncompressed sizes, path safety | None (`zip`, `flate2`) |
| **Binary / Hex** | Any unknown extension | Arbitrary binary | 16-byte hex dump with ASCII sidebar | None |
