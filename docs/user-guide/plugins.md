# WebAssembly Plugins & Extension Center

TermExplorer includes a sandboxed WebAssembly extension platform that allows developers to extend the workstation without giving third-party code unrestricted access to system files, processes, or the network.

---

## 1. Opening the Plugin Center (`Ctrl+Shift+X`)

Press `Ctrl+Shift+X` (or open via Command Palette) to access the Plugin Center:
- **Tab 1: Installed Plugins**: Inspect all currently loaded WebAssembly extensions.
- **Tab 2: Permissions Review**: Inspect granted permissions for each plugin.
- **Tab 3: Marketplace / Available**: Browse bundled plugins.

---

## 2. Bundled Example Plugins

TermExplorer ships with two reference plugins located in `plugins/`:

### A. `workspace-tools`
- **ID**: `workspace-tools`
- **Description**: Workspace management actions, project metrics, and status bar telemetry.
- **Requested Permissions**: `commands`, `workspace_actions`, `status_text`.
- **Integrity**: Verified via SHA-256 checksum in `plugin.toml`.

### B. `file-inspector`
- **ID**: `file-inspector`
- **Description**: Fast file size, extension, and binary heuristic telemetry inspector.
- **Requested Permissions**: `file_metadata`, `status_text`, `commands`.
- **Integrity**: Verified via SHA-256 checksum in `plugin.toml`.

---

## 3. Sandboxing & Capability Boundaries

Plugins run inside a strict in-process `wasmi` sandbox:
- **No Unrestricted Filesystem Access**: Plugins cannot read arbitrary host paths. File metadata is provided only when explicitly permitted.
- **No Host Command Execution**: Plugins cannot spawn child processes or execute shell commands.
- **No Network Sockets**: Network access is completely disabled.
- **Memory Ceiling**: Strict 16 MiB linear memory limit. Any attempt to allocate beyond 16 MiB causes the plugin to be terminated.
- **Fuel Metering**: Execution is metered with 5,000,000 fuel units per invocation. Infinite loops are halted automatically without hanging the application.

---

## 4. Enabling, Disabling, and Uninstalling

- **Toggle Enable / Disable**: Highlight a plugin and press `Enter` or `Space` in the Plugin Center.
- **Reload Plugins**: Press `r` to rescan `plugins/` and `~/.local/share/termexplorer/plugins/` from disk.
- **Uninstall**: Highlight an installed plugin and press `d` to remove the plugin files and revoke granted permissions.
