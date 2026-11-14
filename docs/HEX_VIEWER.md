# TermExplorer: Virtualized Hex Viewer & Binary Inspection Architecture

## 1. Overview

TermExplorer provides a virtualized hex inspector capable of examining multi-gigabyte binary files, disk images, and compiled objects without memory spikes or terminal stalling.

---

## 2. Virtualized $O(1)$ Seeking

Unlike basic terminal tools that read files entirely into memory, TermExplorer implements virtualized arbitrary offset seeking:
```rust
pub fn load_at_offset(path: &Path, offset: usize, max_rows: usize) -> Self
```
- **Constant Memory Consumption**: Only the bytes necessary for the visible viewport rows (`max_rows * 16` bytes, typically 512 bytes to 1 KiB) are read from disk using `std::io::SeekFrom::Start(offset)`.
- **Sub-Millisecond Paging**: Paging through a 50 GiB core dump or disk image executes in under 1 ms.

---

## 3. Row Representation & Layout

Each line displays:
1. **8-Digit Hex Offset**: `00000200`
2. **Left 8-Byte Hex Dump**: `7f 45 4c 46 02 01 01 00`
3. **Right 8-Byte Hex Dump**: `00 00 00 00 00 00 00 00`
4. **ASCII Representation**: `.ELF............` with unprintable bytes sanitized as `.`.

---

## 4. Executable Format Identification & Pattern Search

- **Header Identification**:
  - ELF 32-bit and 64-bit binaries
  - Mach-O 32-bit, 64-bit, and Universal Fat binaries
  - PE32 and PE32+ (Windows Portable Executable)
  - WebAssembly (WASM) modules
  - Java Class bytecode
  - SQLite 3 databases
- **Byte Pattern Search**:
  `HexPreview::search_pattern(path, pattern, max_matches)` allows searching for specific byte sequences across files, returning exact byte offsets.
