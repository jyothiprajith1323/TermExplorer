# WebAssembly Plugin Sandbox Architecture

The Plugin subsystem (`src/plugins/`) embeds a secure, sandboxed extension platform powered by `wasmi`.

---

## 1. Subsystem Architecture

```text
src/plugins/
├── manager.rs           # PluginManager: discovery, loading, lifecycle, status collection
├── manifest.rs          # PluginManifest (plugin.toml): parsing, field validation
├── permissions.rs       # PluginPermission: capability enum & description
└── sandbox.rs           # PluginInstance, HostState, fuel limits, memory boundaries
```

---

## 2. In-Process Sandboxing via `wasmi`

Rather than relying on heavy out-of-process runtimes or native `.so` dynamic libraries (which could segfault or compromise host memory), TermExplorer uses **`wasmi`**:
- **Pure-Rust Interpreter**: Completely isolated execution environment running in safe user-space.
- **Linear Memory Ceiling**: Strictly clamped to `MAX_ALLOWED_MEMORY_BYTES = 16 * 1024 * 1024` (16 MiB). Any attempted growth beyond 16 MiB returns an allocation error.
- **Instruction Fuel Metering**: Every WebAssembly instruction consumes fuel. Each plugin call is initialized with `DEFAULT_FUEL_LIMIT = 5_000_000` fuel units. Infinite loops consume fuel and abort safely.
- **Table Bounds**: Clamped to `MAX_ALLOWED_TABLE_ELEMENTS = 10_000`.

---

## 3. Host / Guest Linker Contract (`sandbox.rs`)

The WebAssembly engine links only specific, controlled host functions under module namespace `"env"`:

| Host Function | Signature | Required Permission | Description |
|---|---|---|---|
| `host_log` | `(ptr: i32, len: i32)` | None (Universal) | Appends log message to plugin session log |
| `host_get_workspace` | `(ptr: i32, max_len: i32) -> i32` | `PluginPermission::WorkspaceActions` | Copies active workspace path into guest memory |
| `host_get_active_file` | `(ptr: i32, max_len: i32) -> i32` | `PluginPermission::FileMetadata` | Copies current selected file path into guest memory |
| `host_set_status` | `(ptr: i32, len: i32) -> i32` | `PluginPermission::StatusText` | Updates custom text displayed in workstation status bar |

Any host function called without its required capability returns `-1` immediately.

---

## 4. Guest Plugin ABI Exports

Every valid TermExplorer plugin module exports:
- `init() -> i32`: Invoked upon loading to initialize internal plugin state.
- `on_command(cmd_ptr: i32, cmd_len: i32) -> i32`: Invoked when a user triggers an associated command action.
- `get_status_text(out_ptr: i32, max_len: i32) -> i32`: Invoked to retrieve telemetry strings for status bar rendering.

---

## 5. SHA-256 Integrity Verification

Before loading any `.wasm` binary into memory, `PluginInstance::new()` computes its SHA-256 digest:
- If `sha256` is declared in `plugin.toml`, the calculated digest must match exactly.
- If tampering or disk corruption is detected, the plugin is rejected with an integrity error.
