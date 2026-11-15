# Plugin Capabilities & Permissions Guide

TermExplorer implements a **default-deny** permission model. A plugin has no capabilities by default; all access to workspace details and host interfaces must be explicitly requested in `plugin.toml`.

---

## 1. Supported Capabilities

The enum `PluginPermission` (`src/plugins/permissions.rs`) defines the supported capabilities:

| Permission Name | Token in `plugin.toml` | Capability Granted | What is NOT Granted |
|---|---|---|---|
| **Commands** | `"commands"` | Register custom actions and keybindings in the Command Palette | Execution of arbitrary shell scripts or host binaries |
| **Context Actions**| `"context_actions"` | Add items to file context menus | Modifying files directly without user confirmation |
| **Workspace Actions** | `"workspace_actions"` | Inspect workspace root path via `host_get_workspace` | Modifying workspace files or changing directories |
| **File Metadata** | `"file_metadata"` | Inspect active filename and size via `host_get_active_file` | Reading full file contents directly into plugin memory |
| **Preview Provider** | `"preview_provider"` | Register custom rendering widgets for specific file extensions | Modifying original file contents |
| **Status Text** | `"status_text"` | Update custom telemetry strings in workstation status bar | Altering other UI areas or overriding system status |

---

## 2. Host Enforcement

Capabilities are validated at runtime when host functions are invoked:

```rust
// Inside host_get_active_file:
if !caller.data().permissions.contains(&PluginPermission::FileMetadata) {
    return -1; // Denied: return error code to guest
}
```

If a plugin attempts to call an unpermitted host function, the function immediately returns `-1` without disclosing data.

---

## 3. User Review in Plugin Center

When users view plugins in the **Plugin Center** (`Ctrl+Shift+X`), all requested permissions are displayed with clear explanations. Users can inspect exactly what data each plugin can access before choosing to enable it.
