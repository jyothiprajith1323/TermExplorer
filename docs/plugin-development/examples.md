# Built-in Plugin Examples

TermExplorer includes two reference plugin implementations located in the `plugins/` directory.

---

## 1. Example 1: `file-inspector`

- **Directory**: `plugins/file-inspector/`
- **Purpose**: Inspects active file attributes and provides telemetry status text.
- **Manifest (`plugin.toml`)**:
  ```toml
  id = "file-inspector"
  name = "File Inspector"
  version = "0.1.0"
  author = "TermExplorer Core Team"
  description = "Fast file size, extension, and binary heuristic telemetry inspector"
  entrypoint = "plugin.wasm"
  permissions = ["file_metadata", "status_text", "commands"]
  sha256 = "9a89d0c9588b9a9c7ec18cac26a8cc9e65818e709ce6b45a99050821a8abebcd"
  ```
- **Source Code (`src/lib.rs`)**:
  Uses `#![no_std]` Rust to export `init()`, `on_command()`, and `get_status_text()`. Calls `host_set_status()` upon initialization to register `"Inspector Ready"`.

---

## 2. Example 2: `workspace-tools`

- **Directory**: `plugins/workspace-tools/`
- **Purpose**: Workspace management actions, project metrics, and status bar telemetry.
- **Manifest (`plugin.toml`)**:
  ```toml
  id = "workspace-tools"
  name = "Workspace Tools"
  version = "0.1.0"
  author = "TermExplorer Core Team"
  description = "Workspace management actions, project metrics, and status bar telemetry"
  entrypoint = "plugin.wasm"
  permissions = ["commands", "workspace_actions", "status_text"]
  sha256 = "9016b629c2e145b4ce8f64ee478966b2c011800b42e39d0fe7d481ce65631af9"
  ```
- **Source Code (`src/lib.rs`)**:
  Imports `host_get_workspace()` to query workspace path details and sets status text to `"Workspace Tools Active"`.

---

## 3. Rebuilding the Example Plugins

To recompile both reference plugins:

```bash
# Rebuild file-inspector
cd plugins/file-inspector
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/file_inspector.wasm plugin.wasm
# Update SHA-256 in plugin.toml:
sha256sum plugin.wasm

# Rebuild workspace-tools
cd ../workspace-tools
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/workspace_tools.wasm plugin.wasm
# Update SHA-256 in plugin.toml:
sha256sum plugin.wasm
```
