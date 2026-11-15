# Caching Architecture & Stale Result Prevention

TermExplorer employs multi-tiered caching strategies across previews, syntax tokens, directory listings, and file metadata.

---

## 1. Cache Hierarchy

| Cache Layer | Storage Medium | Capacity Limits | Eviction Policy | Invalidation Trigger |
|---|---|---|---|---|
| **Preview LRU Cache** | In-Memory (Heap) | 100 items / 64 MiB | Least Recently Used (LRU) | File `mtime` or file size change |
| **Disk Preview Cache** | `~/.cache/termexplorer/` | Bounded per format | Oldest access time | File modification timestamp change |
| **Syntax Scope Cache** | In-Memory (Heap) | 500 scope states | Line range boundaries | Text buffer edits |
| **Filesystem Viewport** | In-Memory | Active directory items | Directory navigation | `notify` watcher event or `F5` refresh |

---

## 2. Invalidation Mechanics & Stale Prevention

Stale cache prevention is strictly enforced using compound cache keys:
```rust
pub struct CacheKey {
    pub path: PathBuf,
    pub mtime: SystemTime,
    pub file_size: u64,
}
```
If a file is updated externally (e.g. by git checkout or another editor), the modification timestamp differs, causing immediate cache miss and re-generation.

---

## 3. Memory Pressure Management

To ensure TermExplorer remains lightweight (<100 MiB RAM under normal workloads):
- Image previews use downsampled pixel matrices rather than keeping high-resolution RGBA bitmaps in RAM.
- Hex dumps and large file previews are memory-mapped or read in bounded slices rather than reading entire gigabyte files into memory.
