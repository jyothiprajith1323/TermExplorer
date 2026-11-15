# Error Codes & Failure Remediation Reference

This reference details error states, diagnostic indicators, and remediation steps.

---

## 1. Configuration & Storage Errors

| Error Indicator | Cause | Remediation Procedure |
|---|---|---|
| `FutureSchemaVersion` | Configuration file specifies `schema_version > 2`. An older TermExplorer binary was launched on a configuration updated by a newer version. | Upgrade TermExplorer to the latest version, or downgrade `schema_version = 2` if you have intentionally reverted. |
| `ConfigParseError` | Malformed syntax in `config.toml` (unclosed quotes, invalid tables). | TermExplorer automatically archives the file to `config.toml.corrupted.<timestamp>` and loads defaults. Inspect the archived file to recover custom values. |
| `AtomicSaveFailed` | Target disk full, read-only filesystem, or permission denied in directory. | Check free disk space with `df -h` and ensure your user owns the workspace directory (`chmod u+w`). |

---

## 2. Terminal & Shell Errors

| Error Indicator | Cause | Remediation Procedure |
|---|---|---|
| `PTYAllocError` | `/dev/ptmx` or `/dev/pts` device cannot be opened. | Ensure the host user belongs to the `tty` group. Inside Docker containers, run with `--privileged` or ensure devpts is mounted. |
| `ShellNotFound` | Configured `default_shell` binary does not exist or is not executable. | In **Settings Studio** -> **Terminal**, reset `default_shell` to empty string (`""`) to restore automated POSIX shell discovery. |

---

## 3. WebAssembly Plugin Errors

| Error Indicator | Cause | Remediation Procedure |
|---|---|---|
| `PluginIntegrityMismatch` | Calculated SHA-256 hash of `.wasm` file does not match manifest. | Rebuild the plugin and re-calculate `sha256sum plugin.wasm`, updating `plugin.toml`. |
| `FuelLimitExceeded` | Plugin executed more than 5,000,000 instructions in a single invocation (possible infinite loop). | Inspect the plugin's loop conditions and avoid heavy computational work inside `init()` or `get_status_text()`. |
| `MemoryLimitExceeded` | Plugin attempted to allocate more than 16 MiB of linear memory. | Ensure the plugin is compiled with `#![no_std]` and optimizes memory buffers. |
| `PermissionDenied` | Plugin invoked a host function without declaring the corresponding capability. | Add the required permission token to `permissions = [...]` in `plugin.toml`. |
