# TermExplorer: Archive Security & Extraction Hardening

## 1. Threat Model & Overview

Compressed archives (ZIP, TAR, TAR.GZ, TGZ, 7Z) downloaded from the internet represent significant security attack vectors:
1. **Path Traversal / Zip Slip**: Malicious file paths (e.g. `../../../../etc/shadow` or absolute paths `/root/.ssh/id_rsa`) designed to escape the extraction target and overwrite critical system files.
2. **Decompression Bombs**: Highly compressed nested files (e.g. 42.zip) designed to expand into gigabytes or petabytes of data, exhausting disk space or memory.
3. **Symlink Escalation**: Symlink entries pointing outside the extraction boundary.

TermExplorer Sprint 3 implements proactive security mitigations across both preview and extraction pipelines.

---

## 2. Zip Slip Prevention Algorithm

Every entry in an archive is subjected to algorithmic path verification in `src/preview/archive/mod.rs`:

```rust
pub fn is_safe_archive_path(path_str: &str) -> bool {
    let path = PathBuf::from(path_str);
    if path.is_absolute() {
        return false;
    }
    for comp in path.components() {
        if let std::path::Component::ParentDir = comp {
            return false;
        }
    }
    true
}
```

During archive extraction via `extract_all`:
1. `entry.enclosed_name()` strips invalid relative or absolute prefixes.
2. The normalized destination path `outpath = target_dir.join(enclosed)` is verified to strictly satisfy:
   ```rust
   if !outpath.starts_with(target_dir) {
       continue; // Skip dangerous escape attempt
   }
   ```
3. Any archive containing unsafe entries is flagged with `has_unsafe_entries: true`, displaying prominent UI warnings in the inspector header.

---

## 3. Decompression Bomb & Resource Exhaustion Protection

TermExplorer enforces strict hard limits on archive extraction:
- **Maximum File Count**: `MAX_EXTRACT_FILES = 10_000`. Extraction terminates immediately if an archive contains more than 10,000 entries.
- **Maximum Decompression Allowance**: `MAX_EXTRACT_BYTES = 500 * 1024 * 1024` (500 MiB).
- **Bounded Stream Reading**: Each file stream is wrapped in `std::io::Read::take(remaining_allowance + 1)`. If total decompressed bytes exceed 500 MiB, the extraction aborts instantly and deletes the partial target file, preventing disk exhaustion.
