# WebAssembly Plugin Platform Overview

TermExplorer includes a sandboxed WebAssembly extension platform that allows third-party developers to extend the workstation safely.

---

## 1. Why WebAssembly?

Traditional native plugin systems (such as C/Rust `.so` or `.dylib` dynamic libraries) expose the host process to critical stability and security risks:
- A null pointer dereference or panic inside a native plugin crashes the entire workstation.
- Native plugins have unconstrained access to filesystem, network, and system calls.
- Binary incompatibilities arise across compiler versions, glibc versions, and platforms.

By executing plugins inside an in-process **`wasmi`** WebAssembly sandbox, TermExplorer achieves:
1. **Fault Isolation**: A memory error or panic in a plugin cannot crash TermExplorer.
2. **Default-Deny Capabilities**: Plugins cannot access files or host system interfaces without explicit capability permissions.
3. **Deterministic Resource Limits**: Plugins are subject to instruction fuel limits (5M fuel) and strict memory ceilings (16 MiB).
4. **Cross-Platform Portability**: A single compiled `.wasm` binary runs identically on Linux x86_64, aarch64, and WSL2.

---

## 2. Anatomy of a Plugin

A TermExplorer plugin consists of a directory containing:
```text
my-plugin/
├── plugin.toml          # Plugin manifest (ID, metadata, permissions, SHA-256)
└── plugin.wasm          # Compiled WebAssembly binary targeting wasm32-unknown-unknown
```

---

## 3. Extension Points

TermExplorer exposes several extension points for plugins:
- **Status Telemetry**: Provide real-time string telemetry to the workstation status bar.
- **Custom Commands**: Register actions in the Command Palette and map keybindings.
- **Context Actions**: Add contextual commands when right-clicking or inspecting files.
- **File Metadata**: Inspect active file attributes and file paths.
- **Workspace Actions**: React to workspace directory changes.
