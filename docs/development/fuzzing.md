# Fuzzing & Robustness Architecture

TermExplorer incorporates fuzz testing to harden parsers, format sniffers, and WebAssembly boundary decoders against adversarial or malformed inputs.

---

## 1. Fuzzing Toolchain & Cargo-Fuzz

Fuzzing leverages `cargo-fuzz` and LLVM's `libFuzzer`:
```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# List available fuzz targets
cargo fuzz list
```

---

## 2. Target Coverage

| Fuzz Target | Tested Subsystem | Hardened Failure Modes |
|---|---|---|
| `fuzz_magic_detector` | `src/preview/detector.rs` | Truncated file headers, corrupted magic bytes, invalid MIME tokens |
| `fuzz_json_parser` | `src/preview/data.rs` | Deeply nested JSON/YAML objects, recursive arrays, stack overflows |
| `fuzz_diff_parser` | `src/git/diff.rs` | Malformed patch headers, negative line numbers, corrupted hunks |
| `fuzz_manifest_parser` | `src/plugins/manifest.rs` | Extremely large TOML tables, unexpected data types, injection strings |
| `fuzz_config_migrator` | `src/config/migration.rs` | Schema version boundary attacks, corrupted integers, unexpected keys |

---

## 3. Running a Fuzz Target

```bash
# Run fuzzing on configuration migrator for 1 hour
cargo fuzz run fuzz_config_migrator -- -max_total_time=3600

# Reproduce an identified crash artifact
cargo fuzz run fuzz_config_migrator fuzz/artifacts/fuzz_config_migrator/crash-xxxx
```
All parser crashes discovered during fuzzing are converted into deterministic regression tests in `tests/`.
