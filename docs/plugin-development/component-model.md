# WebAssembly Component & ABI Model

This document explains the binary interface, linear memory layout, and string passing mechanics between TermExplorer and WebAssembly guest modules.

---

## 1. Linear Memory Architecture

WebAssembly instances in TermExplorer operate over a single linear byte array (`Memory`):
- Pointers passed across host/guest boundaries are 32-bit unsigned integer offsets (`i32`) into this linear memory.
- The host inspects and validates pointer boundaries before reading or writing to prevent out-of-bounds reads/writes:
  ```rust
  let (data, _store) = memory.data_and_store_mut(&mut caller);
  if ptr < 0 || (ptr as usize + len as usize) > data.len() {
      return None; // Out of bounds
  }
  ```

---

## 2. String Transfer Protocol

Because WebAssembly 32-bit linear memory is isolated from the host process heap, strings cannot be passed as raw 64-bit pointers. TermExplorer uses standard C-style pointer + length conventions:

### Guest to Host (e.g. Logging or Status Update)
1. Guest module places UTF-8 bytes into its linear memory.
2. Guest passes `ptr: *const u8` and `len: usize` to the host function (`host_log(ptr, len)`).
3. Host validates that `ptr + len` lies within guest linear memory bounds.
4. Host reads bytes, validates UTF-8, and processes the string.

### Host to Guest (e.g. Querying Active File)
1. Guest allocates a receiving byte buffer in its linear memory (e.g. `let mut buf = [0u8; 512]`).
2. Guest passes `ptr: *mut u8` and `max_len: usize` to the host function (`host_get_active_file(ptr, max_len)`).
3. Host checks permissions (`PluginPermission::FileMetadata`).
4. Host copies at most `max_len` bytes into the guest's buffer and returns the actual number of bytes written (or `-1` on error).
