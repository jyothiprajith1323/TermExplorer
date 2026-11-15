# Release Engineering & Packaging Infrastructure

TermExplorer features a reproducible packaging system established in Sprint 6A.

---

## 1. Supported Release Deliverables

| Deliverable | Target System | Path / Filename | Description |
|---|---|---|---|
| **Standalone Tarball** | Generic Linux `x86_64` | `dist/termexplorer-v1.1.0-linux-x86_64.tar.gz` | Binary, manpage, desktop entry, icon, completions, installer scripts |
| **Debian / Ubuntu Package** | Ubuntu 20.04+, Debian 11+ | `dist/termexplorer_1.1.0_amd64.deb` | Native `.deb` package with dependencies and system integration |
| **Checksum Manifest** | All Releases | `dist/SHA256SUMS` | Cryptographic SHA-256 hashes for supply-chain security |
| **Artifact Manifest** | All Releases | `dist/MANIFEST.txt` | Complete manifest of included files and metadata |

---

## 2. Release Build Flags

Release binaries are built using Cargo with optimizations tuned for maximum runtime speed and minimal binary size:

```toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
panic = "unwind"
strip = true
```

---

## 3. Automated Packaging Pipeline

The entire packaging pipeline can be executed locally or via CI:

```bash
# Execute end-to-end release pipeline
./scripts/package-all.sh
```

### Pipeline Sequence:
1. `scripts/build-release.sh`: Compiles release binary with `--locked`.
2. `scripts/package-tarball.sh`: Assembles payload directory, packages manpage, completions, desktop file, icon, and produces `.tar.gz`.
3. `scripts/package-deb.sh`: Generates `DEBIAN/control`, verifies package directory layout, and executes `dpkg-deb --build`.
4. `scripts/generate-checksums.sh`: Calculates SHA-256 digests and outputs `dist/SHA256SUMS`.
5. `tests/packaging_tests.sh`: Executes 49 automated quality tests verifying tarball integrity, `.deb` archive layout, manpage gzipping, and installer/uninstaller idempotency.
