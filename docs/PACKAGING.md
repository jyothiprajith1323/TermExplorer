# 📦 TermExplorer — Packaging & Distribution Architecture

This document specifies the packaging infrastructure, reproducible release procedures, artifact formats, and distribution standards for **TermExplorer**.

---

## 🏗️ 1. Supported Platform Matrix

| Platform / Target | Tier | Build Host / Toolchain | Runtime Minimum | Validation Status |
|:---|:---:|:---|:---|:---:|
| **Linux x86_64** (`x86_64-unknown-linux-gnu`) | **Tier 1 (Primary)** | Ubuntu 22.04+ / Rust 1.80+ | `glibc >= 2.34` | **Fully Validated** |
| **Linux ARM64** (`aarch64-unknown-linux-gnu`) | **Tier 2 (Secondary)** | Cross-compile / aarch64 host | `glibc >= 2.34` | **Fully Supported** |
| **WSL2 (Windows Subsystem for Linux 2)** | **Tier 1 (Runtime)** | Linux x86_64 / AArch64 | WSL2 Kernel 5.10+ | **Fully Validated** |

---

## 📦 2. Release Artifact Formats & Naming

Artifacts produced by TermExplorer's release pipeline follow strict, deterministic naming conventions:

### A. Portable Linux Archive (`.tar.gz`)
- **Naming Pattern:** `termexplorer-v<VERSION>-<OS>-<ARCH>.tar.gz`
- **Example:** `dist/termexplorer-v1.1.0-linux-x86_64.tar.gz`
- **Contents:**
  ```text
  termexplorer-v1.1.0-linux-x86_64/
  ├── bin/
  │   └── termexplorer                 # Stripped release executable (0755)
  ├── share/
  │   ├── man/man1/
  │   │   └── termexplorer.1           # Troff/groff manual page (0644)
  │   ├── applications/
  │   │   └── termexplorer.desktop     # FreeDesktop desktop entry (0644)
  │   ├── icons/hicolor/scalable/apps/
  │   │   └── termexplorer.svg         # Scalable vector application icon (0644)
  │   ├── bash-completion/completions/
  │   │   └── termexplorer             # Bash shell autocompletion (0644)
  │   ├── zsh/site-functions/
  │   │   └── _termexplorer            # Zsh shell autocompletion (0644)
  │   ├── fish/vendor_completions.d/
  │   │   └── termexplorer.fish        # Fish shell autocompletion (0644)
  │   └── doc/termexplorer/
  │       ├── README.md                # Project documentation (0644)
  │       └── LICENSE                  # MIT License notice (0644)
  ├── install.sh                       # User / system installer (0755)
  └── uninstall.sh                     # Non-destructive uninstaller (0755)
  ```

### B. Debian Binary Package (`.deb`)
- **Naming Pattern:** `termexplorer_<VERSION>_<DEB_ARCH>.deb`
- **Example:** `dist/termexplorer_1.1.0_amd64.deb`
- **Installed Hierarchy:**
  ```text
  /usr/bin/termexplorer
  /usr/share/man/man1/termexplorer.1.gz
  /usr/share/applications/termexplorer.desktop
  /usr/share/icons/hicolor/scalable/apps/termexplorer.svg
  /usr/share/bash-completion/completions/termexplorer
  /usr/share/zsh/vendor-completions/_termexplorer
  /usr/share/fish/vendor_completions.d/termexplorer.fish
  /usr/share/doc/termexplorer/copyright
  /usr/share/doc/termexplorer/changelog.Debian.gz
  /usr/share/doc/termexplorer/README.md
  ```

---

## 🔒 3. Reproducible Build Infrastructure

TermExplorer uses a deterministic release build workflow:
1. **Dependency Locking:** Every build enforces `cargo build --locked --release` with an audited `Cargo.lock`.
2. **Release Profiles:**
   - Link-Time Optimization: `lto = "fat"`
   - Codegen Units: `codegen-units = 1`
   - Panic Strategy: `panic = "abort"`
   - Strip Settings: `strip = true` (with post-build `strip --strip-unneeded`)
3. **Reproducible Tarballs:**
   - Standardized timestamps via `SOURCE_DATE_EPOCH` (derived from latest Git commit timestamp).
   - Strict root ownership normalization: `--owner=0 --group=0 --numeric-owner`.
   - Alphabetical member ordering: `--sort=name`.

---

## 🛠️ 4. Build & Packaging Scripts

The repository includes modular scripts located in `scripts/`:

| Script | Responsibility |
|:---|:---|
| `scripts/build-release.sh` | Compiles optimized binary and audits ABI/symbols via `readelf` |
| `scripts/package-tarball.sh` | Assembles staging hierarchy and creates portable `.tar.gz` |
| `scripts/package-deb.sh` | Builds Debian package via `dpkg-deb` with control, md5sums, and manpage gzip |
| `scripts/generate-checksums.sh`| Calculates SHA-256 sums and produces `dist/SHA256SUMS` and `dist/MANIFEST.txt` |
| `scripts/package-all.sh` | Master orchestrator invoking all packaging phases in sequence |

---

## 🛡️ 5. Supply-Chain Integrity & Verification

Every release produces cryptographic hashes and manifest metadata:

1. **`dist/SHA256SUMS`:** Standard format for verifying downloads:
   ```bash
   sha256sum -c dist/SHA256SUMS
   ```
2. **`dist/MANIFEST.txt`:** Detailed Markdown table recording file sizes, architectures, formats, toolchain versions, and commit hashes.
3. **Signatures:** When official release signing keys are available in production CI, artifacts are signed with detached GPG/Minisign signatures. In accordance with security best practices, no placeholder or fake keys are checked into the repository.

---

## 🧪 6. Automated Packaging Quality Gates

Automated acceptance tests are implemented in `tests/packaging_tests.sh`:
- Archive extraction and filesystem safety (no directory traversal, no unwanted files).
- Binary execution, `--version`, `--help`, and `--health` diagnostics in clean sandboxes.
- Installer execution in isolated prefixes with spaces and Unicode (`/tmp/test_üñîçødé_ワークスペース`).
- Uninstallation verification: ensures installed files are cleanly removed while user configurations in `~/.config/termexplorer` are preserved.
- Debian package structure inspection (`dpkg-deb --info`, `dpkg-deb --contents`, and `md5sum -c DEBIAN/md5sums`).
