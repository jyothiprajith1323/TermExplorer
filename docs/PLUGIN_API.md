# TermExplorer V1.1 Plugin Host-Guest API Reference

This document details the ABI contract between TermExplorer (Host) and WebAssembly Plugins (Guest).

---

## 1. Exported Host Functions (`env` module)

Plugins import these functions from the host environment `env`:

### `host_log(ptr: *const u8, len: usize)`
Sends a UTF-8 log message to TermExplorer's diagnostic tracing pipeline.

---

### `host_get_workspace(ptr: *mut u8, max_len: usize) -> i32`
- **Requires Permission:** `workspace_actions`
- Writes the current workspace directory path into guest memory at `ptr`.
- Returns the number of bytes written, or `-1` if permission is missing or buffer is insufficient.

---

### `host_get_active_file(ptr: *mut u8, max_len: usize) -> i32`
- **Requires Permission:** `file_metadata`
- Writes the currently highlighted file's name into guest memory at `ptr`.
- Returns the number of bytes written, or `-1` on error.

---

### `host_set_status(ptr: *const u8, len: usize) -> i32`
- **Requires Permission:** `status_text`
- Sets the live status bar text associated with this plugin.
- Returns `0` on success, `-1` on permission denial.

---

## 2. Exported Guest Functions (Plugin entrypoints)

Plugins must or may export these functions for the host:

### `init() -> i32`
- Called once when the plugin is loaded and verified by `PluginManager`.
- Return `0` to signal successful initialization.

---

### `on_command(cmd_ptr: *const u8, cmd_len: usize) -> i32`
- **Requires Permission:** `commands`
- Invoked when the user triggers a command associated with this plugin from the Command Palette or a keybinding.
- Return `0` on success, non-zero error code on failure.

---

### `get_status_text(out_ptr: *mut u8, max_len: usize) -> i32`
- **Requires Permission:** `status_text`
- Invoked periodically by the status bar renderer to collect custom telemetry.
- Writes up to `max_len` bytes to `out_ptr` and returns the byte length.

---

## 3. Developing a Rust WASM Plugin

To compile a plugin in Rust:

```rust
#![no_std]

extern "C" {
    fn host_set_status(ptr: *const u8, len: usize) -> i32;
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

static STATUS: &[u8] = b"My Plugin Ready";

#[no_mangle]
pub extern "C" fn init() -> i32 {
    unsafe {
        host_set_status(STATUS.as_ptr(), STATUS.len());
    }
    0
}

#[no_mangle]
pub extern "C" fn on_command(_cmd_ptr: *const u8, _cmd_len: usize) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn get_status_text(out_ptr: *mut u8, max_len: usize) -> i32 {
    let len = STATUS.len().min(max_len);
    unsafe {
        core::ptr::copy_nonoverlapping(STATUS.as_ptr(), out_ptr, len);
    }
    len as i32
}
```

### Build Command:
```bash
rustc --target wasm32-unknown-unknown \
      --crate-type cdylib \
      -O -Clink-arg=--allow-undefined \
      -o plugin.wasm src/lib.rs
```
