# TermExplorer: Two-Tier Preview Cache Architecture

## 1. Overview

To deliver sub-30ms first-viewport display times and 60 FPS scrolling across large file trees, TermExplorer implements a two-tier caching hierarchy:
1. **Tier 1 (In-Memory)**: Bounded Least Recently Used (`PreviewLruCache<T>`) storing decoded preview structures for instantaneous tab switching and panel redrawing.
2. **Tier 2 (Persistent Disk)**: Bounded file cache in `$XDG_CACHE_HOME/termexplorer/previews/` indexed by BLAKE3 cryptographic hashes.

---

## 2. In-Memory LRU Cache (`PreviewLruCache<T>`)

- **Capacity & Eviction**: Configured with a default limit of 64 preview records (`PreviewState::full_data`).
- **Access Promotion**: Every cache hit moves the item to the front of the eviction list.
- **Eviction Boundary**: When capacity is reached, the least recently viewed entry is purged from RAM.
- **Zero-Copy Re-rendering**: When scrolling past previously previewed files, preview rendering is $O(1)$ with zero file I/O or decode overhead.

---

## 3. Persistent Disk Cache (`DiskPreviewCache`)

For computationally heavy assets (such as SVG vector rasterization, video thumbnail grids, and large syntax caches), TermExplorer writes cached artifacts to disk:

- **Storage Location**: `$XDG_CACHE_HOME/termexplorer/previews/` (or `~/.cache/termexplorer/previews/`).
- **BLAKE3 Content Fingerprinting**:
  Cached files are named according to the BLAKE3 digest of their source file path, mtime, and file size:
  ```rust
  pub fn compute_fingerprint(path: &Path, mtime_secs: u64, size: u64) -> String {
      let mut hasher = blake3::Hasher::new();
      hasher.update(path.to_string_lossy().as_bytes());
      hasher.update(&mtime_secs.to_le_bytes());
      hasher.update(&size.to_le_bytes());
      hasher.finalize().to_hex().to_string()
  }
  ```
- **Atomic Disk Writes via NamedTempFile**:
  Cached payloads are written to a temporary file within the cache directory and atomically renamed to their final fingerprint destination. This guarantees zero file corruption during unexpected crashes or terminal closure.
- **Disk Quota Enforcement**:
  `DiskPreviewCache` enforces a default quota of 100 MiB. When total disk cache bytes exceed this ceiling, oldest cache entries (determined by file access timestamp) are pruned until disk usage returns within bounds.
