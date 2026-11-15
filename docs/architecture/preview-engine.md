# Preview Engine & Format Detection Architecture

The Universal Preview Engine (`src/preview/`) provides instantaneous, deep inspection across a wide variety of file formats while guaranteeing memory safety and UI responsiveness.

---

## 1. Multi-Tier Architecture

```mermaid
flowchart TD
    File["Selected File Path"] --> MagicSniffer["Magic-Byte & MIME Sniffer\n(src/preview/detector.rs)"]
    
    MagicSniffer --> Registry["Capability Registry\n(src/preview/registry.rs)"]
    
    Registry --> CheckLRU{"In LRU Cache?\n(PreviewLruCache)"}
    CheckLRU -->|Hit (<1ms)| Render["Render Preview Widget"]
    
    CheckLRU -->|Miss| Dispatch{"Format Category?"}
    Dispatch -->|Source Code| Syntect["syntect tokenization"]
    Dispatch -->|Markdown| Pulldown["pulldown-cmark"]
    Dispatch -->|Images| Halfblocks["24-bit RGB half-blocks (▀)"]
    Dispatch -->|Data Tables| Tabular["CSV/TSV/JSON pretty-printer"]
    Dispatch -->|Database| SQLite["SQLite header & schema parser"]
    Dispatch -->|PDF| PDF["pdftotext / Poppler"]
    Dispatch -->|Audio/Video| Media["lofty metadata / ffmpeg thumbnail"]
    Dispatch -->|Office XML| Office["DOCX/XLSX pure-Rust unpacker"]
    Dispatch -->|Archives| Archive["zip / tar.gz directory tree"]
    Dispatch -->|Binary/Unknown| Hex["Canonical 16-byte hex dump"]

    Syntect & Pulldown & Halfblocks & Tabular & SQLite & PDF & Media & Office & Archive & Hex --> StoreLRU["Store in LRU & Disk Cache"]
    StoreLRU --> Render
```

---

## 2. Magic-Byte Sniffer (`detector.rs`)

Relying solely on file extensions is vulnerable to misnamed files (e.g. an executable named `document.pdf` or a PNG named `image.txt`).

TermExplorer inspects initial byte signatures:
- `%PDF-`: PDF Document
- `\x89PNG\r\n\x1a\n`: PNG Image
- `\xff\xd8\xff`: JPEG Image
- `GIF87a` / `GIF89a`: GIF Image
- `SQLite format 3\0`: SQLite 3 Database
- `PK\x03\x04`: ZIP / DOCX / XLSX Archive Container
- UTF-8 Validation check: Distinguishes plain text source files from binary byte streams.

---

## 3. Monotonic Cancellation (`cancellation.rs`)

When users rapidly hold down `j` or scroll with the mouse wheel across 500 files:
- Each navigation step increments a monotonic generation counter (`u64`).
- In-flight background preview tasks check this counter before initiating expensive operations (such as tokenizing a 5 MB Rust file or calling `ffmpeg`).
- If the token is superseded, the worker terminates immediately, ensuring CPU resources are focused exclusively on the file currently under the cursor.

---

## 4. Multi-Tier Cache Architecture (`cache.rs`)

1. **`PreviewLruCache` (In-Memory)**:
   - Cap: 100 entries / 64 MiB total.
   - Preserves rendered Ratatui widget data in RAM.
   - Cache key includes: canonical path, file modification timestamp, and file length.
2. **`DiskPreviewCache` (Persistent)**:
   - Stored in `~/.cache/termexplorer/previews/`.
   - Caches expensive disk artifacts (extracted PDF text, video thumbnails, high-resolution downsampled images).
   - Invalidation occurs automatically when file `mtime` changes.
