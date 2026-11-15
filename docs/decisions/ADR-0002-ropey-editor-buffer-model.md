# ADR-0002: Rope Data Structure for Text Editor Buffer

- **Status**: Accepted
- **Context**: Reconstructed from Current Implementation (Sprint 2)

---

## 1. Context & Problem Statement

The integrated code editor must support opening, inspecting, and editing source code files ranging from small scripts to multi-megabyte datasets (100,000+ lines).
Using a single contiguous `String` causes $O(N)$ memory copies on every character insertion or deletion, leading to unacceptable typing latency on large files.
Using a line array (`Vec<String>`) causes high memory fragmentation and slow multi-line paste operations.

## 2. Decision

TermExplorer uses **`ropey::Rope`** (`src/editor/buffer.rs`):
- Text is represented as a balanced B-tree of string chunks.
- Insertions and deletions execute in $O(\log N)$ time.
- Line indexing and char indexing are provided efficiently by tree metadata.
- Text slicing and Unicode scalar boundaries are preserved.

## 3. Alternatives Considered

1. **Gap Buffer**: Fast for localized linear typing, but expensive when moving cursor across distant sections of large files.
2. **Piece Table (VS Code style)**: Excellent for append-only undo histories, but introduces implementation complexity for character-width measurements and multi-language Unicode segmentation.

## 4. Consequences

- **Positive**: Sub-millisecond insertion latency even in 10 MB text files; no contiguous memory allocation spikes.
- **Negative**: Iterating lines requires traversal through rope chunks rather than pointer offsets; bridged cleanly using `RopeSlice`.
- **Security Impact**: Eliminates buffer overrun vulnerabilities and memory exhaustion panics during large pastes.
