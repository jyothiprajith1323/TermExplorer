# Resource Limits & Sandboxing Constraints

TermExplorer enforces deterministic resource limits to prevent rogue, buggy, or malicious WebAssembly plugins from exhausting host system resources.

---

## 1. Linear Memory Ceiling (16 MiB)

WebAssembly instances allocate linear memory in 64 KiB pages:
- **Safety Ceiling**: `MAX_ALLOWED_MEMORY_BYTES = 16 * 1024 * 1024` (16 MiB / 256 pages).
- **Static Validation**: Upon loading, if the WebAssembly module declares an initial memory or maximum memory greater than 16 MiB, instantiation fails immediately.
- **Dynamic Growth Limiter**: If a plugin attempts to grow its memory dynamically at runtime (`memory.grow`) beyond 16 MiB, the `PluginLimiter` rejects the growth request, causing the WebAssembly allocation to fail gracefully.

---

## 2. Instruction Fuel Metering (5M Instructions)

To prevent infinite loops from hanging the application event loop:
- TermExplorer enables `wasmi::Config::consume_fuel(true)`.
- Before each guest function call (`init`, `on_command`, `get_status_text`), the store is allocated `DEFAULT_FUEL_LIMIT = 5_000_000` fuel units.
- Every executed opcode decrements the fuel counter.
- If fuel reaches zero, execution aborts with a trap error, and TermExplorer marks the plugin as faulted without freezing the workstation.

---

## 3. Table Limits

WebAssembly function tables are restricted to:
- `MAX_ALLOWED_TABLE_ELEMENTS = 10_000` elements.
- Prevents table expansion attacks.

---

## 4. SHA-256 Integrity Verification

If `sha256` is defined in `plugin.toml`:
- TermExplorer calculates the cryptographic SHA-256 digest of the `.wasm` binary on disk.
- If the calculated digest does not match the manifest value, the plugin is rejected with an integrity check error.
