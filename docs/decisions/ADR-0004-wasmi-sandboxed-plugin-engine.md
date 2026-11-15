# ADR-0004: In-Process WebAssembly Sandboxing via wasmi

- **Status**: Accepted
- **Context**: Reconstructed from Current Implementation (Sprint 5B)

---

## 1. Context & Problem Statement

To provide an extensible plugin ecosystem, TermExplorer requires a runtime that:
1. Prevents untrusted third-party code from accessing arbitrary files or network sockets.
2. Prevents memory corruption or crashes in plugins from affecting the host workstation.
3. Limits execution time and memory consumption deterministically.
4. Operates in-process with minimal startup latency and zero external C++ runtime dependencies.

## 2. Decision

TermExplorer implements an in-process WebAssembly interpreter using **`wasmi`** (`src/plugins/sandbox.rs`):
- Interprets standard WebAssembly core modules (`wasm32-unknown-unknown`).
- **Memory Limiter**: Enforces a strict 16 MiB ceiling on linear memory growth.
- **Instruction Fuel**: Meters execution with 5,000,000 fuel units per invocation, aborting infinite loops.
- **Host Linker**: Exposes only vetted host functions under `"env"`, guarded by default-deny capabilities.
- **Integrity**: Enforces SHA-256 binary checksum validation declared in `plugin.toml`.

## 3. Alternatives Considered

1. **Native Dynamic Libraries (`libloading` / `.so`)**: High performance, but any crash segfaults TermExplorer; no sandbox or permission boundaries possible.
2. **Wasmtime (JIT Compiler)**: Excellent execution throughput, but requires heavy compilation machinery, adds several megabytes to binary size, and has platform compilation constraints.
3. **External Process Plugins (IPC / JSON-RPC)**: Good isolation, but high IPC serialization overhead and complex process management.

## 4. Consequences

- **Positive**: Complete fault isolation, small footprint, pure-Rust implementation, deterministic fuel metering.
- **Negative**: Interpreter execution is slower than JIT compilation; mitigated by restricting plugins to lightweight UI hooks, commands, and telemetry rather than heavy number crunching.
- **Security Impact**: Default-deny capability model prevents unauthorized file or network access.
