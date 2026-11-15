# ADR-0006: Multi-Tier Preview Engine Architecture

- **Status**: Accepted
- **Context**: Reconstructed from Current Implementation (Sprint 3A)

---

## 1. Context & Problem Statement

TermExplorer is an inspection-first workstation that must display previews across diverse file formats (source code, Markdown, images, video containers, audio tags, PDFs, office documents, tabular data, SQLite databases, and binary files).
Relying solely on file extensions leads to incorrect rendering or crashes when files are misnamed or disguised.
Re-decoding previews on every keystroke exhausts CPU and battery.

## 2. Decision

TermExplorer implements a **Multi-Tier Preview Engine** (`src/preview/`):
1. **Magic-Byte Sniffer**: Reads initial byte signatures to accurately classify format categories regardless of extension.
2. **Multi-Tier Cache**:
   - `PreviewLruCache`: In-memory LRU cache retaining up to 100 recent preview artifacts (64 MiB ceiling).
   - `DiskPreviewCache`: Persistent cache for expensive operations (PDF text, video thumbnails).
3. **Monotonic Cancellation**: Asynchronous workers monitor a generation counter; in-flight jobs are cancelled immediately when the user navigates to a different file.

## 3. Alternatives Considered

1. **Synchronous File Loading on Render**: Freezes the TUI frame rate when navigating across multi-megabyte files or external tool invocations.
2. **Spawning External Viewers (e.g. `xdg-open`)**: Disrupts terminal workflow by launching separate GUI windows.

## 4. Consequences

- **Positive**: Instantaneous preview switching (<1 ms cache hits); accurate format identification; zero UI freezing.
- **Negative**: High memory footprint if cache bounds are not enforced; solved by hard 64 MiB ceiling and LRU eviction.
