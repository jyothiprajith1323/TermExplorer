# TermExplorer V1.1 Plugin Architecture & Sandbox

TermExplorer V1.1 introduces an extensible, sandboxed **WebAssembly (WASM)** plugin architecture powered by `wasmi` and `sha2`.

Plugins run inside a deterministic virtual machine with strict fuel limits, memory caps, explicit capability permissions, and cryptographic integrity verification.

---

## 1. Safety & Sandboxing Guarantees

- **No Native Dynamic Libraries:** Plugins are compiled to `.wasm` bytecode and executed within a pure-Rust virtual machine. Plugins cannot issue arbitrary native syscalls or execute malicious shell commands directly.
- **Fuel / Instruction Limits:** Every call to a plugin is budgeted with a bounded execution fuel limit (`5,000,000` steps). If an infinite loop or heavy calculation occurs, execution halts safely with an error without freezing the TUI.
- **Memory Boundaries:** Plugins run with dedicated memory isolated from TermExplorer's heap.
- **Panic & Fault Isolation:** If a WebAssembly module crashes or triggers a trap, TermExplorer catches the error, marks the plugin as `PluginState::Error`, logs the stack fault, and continues running normally.
- **Cryptographic SHA-256 Checksums:** Plugin manifests can specify a `sha256` digest of their `plugin.wasm`. If the binary does not match, loading is rejected.

---

## 2. Capability Permissions Model

Plugins must declare requested capabilities in their manifest. The host will refuse unauthorized calls:

| Permission | Capability Scope |
|:---|:---|
| `commands` | Register developer commands and keyboard triggers |
| `workspace_actions` | Query workspace path, project type, and trigger build hooks |
| `file_metadata` | Inspect active file name, size, extension, and permissions |
| `context_actions` | Add custom actions to the right-click / context menu |
| `preview_provider` | Provide custom file content rendering |
| `status_text` | Display telemetry text in the status bar |

---

## 3. Directory Layout & Discovery

TermExplorer automatically discovers plugins in:
```
~/.local/share/termexplorer/plugins/
```

Each plugin resides in its own subfolder:
```
~/.local/share/termexplorer/plugins/
└── workspace-tools/
    ├── plugin.toml
    └── plugin.wasm
```

### Manifest Format (`plugin.toml`):
```toml
id = "workspace-tools"
name = "Workspace Tools"
version = "0.1.0"
author = "Developer Name"
description = "Project metrics and workspace build actions"
entrypoint = "plugin.wasm"
permissions = ["commands", "workspace_actions", "status_text"]
sha256 = "9016b629c2e145b4ce8f64ee478966b2c011800b42e39d0fe7d481ce65631af9"
```

---

## 4. Built-in Example Plugins

TermExplorer includes two working reference plugins located in `plugins/`:

1. **`workspace-tools`**:
   - Queries workspace path and registers build commands.
   - Emits live status telemetry into the status bar: `󰡨 [Workspace Tools: Workspace Tools Active]`.

2. **`file-inspector`**:
   - Inspects the active file context dynamically as the cursor moves through the file explorer.
   - Computes heuristics on binary flags and file size limits.
