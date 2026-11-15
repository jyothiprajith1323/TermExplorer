# Build, Test & Packaging Workflows

This document outlines the standard commands for compiling, testing, and packaging TermExplorer.

---

## 1. Compilation Commands

```bash
# Debug build (with debug symbols and fast compile times)
cargo build

# Optimized release binary (LTO enabled, stripped symbols)
cargo build --release --locked

# Check compilation across all targets (bins, tests, benchmarks)
cargo check --locked --all-targets
```

---

## 2. Test Execution

TermExplorer maintains 26 integration and unit test suites:

```bash
# Run all automated tests
cargo test --all-targets

# Run a specific test suite (e.g. configuration & first-run tests)
cargo test --test config_and_first_run_tests

# Run tests with stdout output enabled
cargo test --test config_and_first_run_tests -- --nocapture

# Run adversarial filesystem tests
cargo test --test adversarial_filesystem_tests
```

---

## 3. Code Quality & Static Analysis Gates

All pull requests must pass the project's strict quality gates:

```bash
# 1. Formatting check (enforces rustfmt.toml)
cargo fmt --all -- --check

# 2. Strict Clippy lint check (0 warnings tolerated)
cargo clippy --all-targets --all-features -- -D warnings

# 3. Security audit of dependencies
cargo deny check

# 4. Generate API documentation without dependencies
cargo doc --no-deps
```

---

## 4. Release Packaging Scripts

Sprint 6A introduced automated packaging scripts located in `scripts/`:

```bash
# Build optimized release binary
./scripts/build-release.sh

# Generate standalone tarball (.tar.gz)
./scripts/package-tarball.sh

# Generate Debian package (.deb)
./scripts/package-deb.sh

# Generate SHA256 checksums and manifest
./scripts/generate-checksums.sh

# Run end-to-end packaging pipeline
./scripts/package-all.sh
```

All generated distribution artifacts are placed in the `dist/` directory alongside `dist/SHA256SUMS`.
