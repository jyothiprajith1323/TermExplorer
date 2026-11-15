# WebAssembly Host & Guest Interface Specification (WIT / ABI Reference)

This specification defines the host-guest interface contracts implemented in `src/plugins/sandbox.rs`.

---

## 1. Guest Exported Functions

Every TermExplorer plugin module must export the following C-ABI functions:

### `init`
```rust
#[no_mangle]
pub extern "C" fn init() -> i32
```
- **Description**: Invoked immediately after the module is loaded and instantiated.
- **Parameters**: None.
- **Returns**: `0` on success, or non-zero error code.

### `on_command`
```rust
#[no_mangle]
pub extern "C" fn on_command(cmd_ptr: *const u8, cmd_len: usize) -> i32
```
- **Description**: Invoked when a user triggers an associated command action from the Command Palette or a keybinding.
- **Parameters**:
  - `cmd_ptr`: Pointer in guest linear memory to UTF-8 command identifier string.
  - `cmd_len`: Length of the command identifier in bytes.
- **Returns**: `0` on success, or non-zero error code.

### `get_status_text`
```rust
#[no_mangle]
pub extern "C" fn get_status_text(out_ptr: *mut u8, max_len: usize) -> i32
```
- **Description**: Invoked periodically by the workstation to collect status text for the status bar.
- **Parameters**:
  - `out_ptr`: Pointer to destination buffer in guest linear memory.
  - `max_len`: Maximum bytes the destination buffer can accept.
- **Returns**: The number of bytes written, or `-1` if unavailable.

---

## 2. Host Imported Functions (`"env"` Namespace)

Plugins can import the following host functions:

### `host_log`
```rust
extern "C" {
    fn host_log(ptr: *const u8, len: usize);
}
```
- **Required Permission**: None (Universal).
- **Description**: Emits a log line to the plugin's internal log buffer and tracing logs.

### `host_get_workspace`
```rust
extern "C" {
    fn host_get_workspace(ptr: *mut u8, max_len: usize) -> i32;
}
```
- **Required Permission**: `workspace_actions`
- **Description**: Copies the current workspace root path into `ptr`.
- **Returns**: Number of bytes written, or `-1` if permission is denied.

### `host_get_active_file`
```rust
extern "C" {
    fn host_get_active_file(ptr: *mut u8, max_len: usize) -> i32;
}
```
- **Required Permission**: `file_metadata`
- **Description**: Copies the path of the currently selected file into `ptr`.
- **Returns**: Number of bytes written, or `-1` if permission is denied.

### `host_set_status`
```rust
extern "C" {
    fn host_set_status(ptr: *const u8, len: usize) -> i32;
}
```
- **Required Permission**: `status_text`
- **Description**: Updates the custom status bar telemetry string for this plugin.
- **Returns**: `0` on success, or `-1` if permission is denied.
