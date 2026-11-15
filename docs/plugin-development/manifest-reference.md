# Plugin Manifest Reference (`plugin.toml`)

Every TermExplorer plugin must provide a valid `plugin.toml` manifest file in its root directory.

---

## 1. Complete Manifest Example

```toml
id = "my-sample-plugin"
name = "My Sample Extension"
version = "1.0.0"
author = "Jane Developer <jane@example.com>"
description = "Demonstrates custom commands and status telemetry"
entrypoint = "plugin.wasm"
permissions = ["commands", "file_metadata", "status_text"]
sha256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
```

---

## 2. Manifest Field Specification

| Field | Type | Required | Default | Description |
|---|---|:---:|---|---|
| `id` | `String` | **Yes** | — | Unique identifier string (kebab-case, e.g. `file-inspector`). Used in command IDs and config storage. |
| `name` | `String` | **Yes** | — | Human-readable display name shown in Plugin Center and settings. |
| `version` | `String` | **Yes** | — | Semantic version string (`MAJOR.MINOR.PATCH`). |
| `author` | `String` | No | `None` | Author name or email string. |
| `description` | `String` | No | `None` | One-sentence summary of plugin functionality. |
| `entrypoint` | `String` | No | `"plugin.wasm"` | Relative path to the compiled WebAssembly binary. |
| `permissions` | `Array<String>`| No | `[]` | List of requested capability permissions (e.g. `["status_text"]`). |
| `sha256` | `String` | No | `None` | Expected 64-character lowercase hexadecimal SHA-256 digest of the `.wasm` binary. |

---

## 3. Validation Rules

- `id` must be non-empty and must not contain path separators (`/`, `\`).
- `entrypoint` must resolve to an existing file relative to `plugin.toml`.
- If `sha256` is provided, TermExplorer calculates the actual file hash upon loading and halts if there is a mismatch.
- Any unrecognized permission names cause manifest loading to fail, alerting the user to invalid or outdated plugin manifests.
