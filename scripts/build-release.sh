#!/usr/bin/env bash
# TermExplorer Reproducible Release Build Script
# Builds the production release binary using locked dependencies and optimization profiles.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${REPO_ROOT}"

echo "=========================================================="
echo " Building TermExplorer Release Binary"
echo "=========================================================="
echo "Repository:    ${REPO_ROOT}"
echo "Rustc Version: $(rustc --version)"
echo "Cargo Version: $(cargo --version)"
echo "Git Commit:    $(git rev-parse --short HEAD 2>/dev/null || echo "unknown")"
echo "Git Status:    $([ -z "$(git status --porcelain 2>/dev/null)" ] && echo "Clean" || echo "Dirty (local modifications)")"
echo "Timestamp:     $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
echo "=========================================================="

# Build with locked dependencies and release profile
cargo build --locked --release

TARGET_BIN="${REPO_ROOT}/target/release/termexplorer"

if [ ! -f "${TARGET_BIN}" ]; then
    echo "Error: Release binary not found at ${TARGET_BIN}" >&2
    exit 1
fi

# Ensure binary is stripped of unnecessary debug symbols
if command -v strip >/dev/null 2>&1; then
    strip --strip-unneeded "${TARGET_BIN}" || true
fi

BIN_SIZE=$(stat -c%s "${TARGET_BIN}")
BIN_SIZE_HUMAN=$(numfmt --to=iec-i --suffix=B "${BIN_SIZE}" 2>/dev/null || ls -lh "${TARGET_BIN}" | awk '{print $5}')

echo ""
echo "Release Binary Built Successfully:"
echo "  Path: ${TARGET_BIN}"
echo "  Size: ${BIN_SIZE} bytes (${BIN_SIZE_HUMAN})"
echo "  Version: $("${TARGET_BIN}" --version)"

# Verify ABI and linked shared libraries
echo ""
echo "Dynamic Linkage Analysis (ldd / readelf):"
if command -v readelf >/dev/null 2>&1; then
    readelf -d "${TARGET_BIN}" | grep "NEEDED" || true
elif command -v ldd >/dev/null 2>&1; then
    ldd "${TARGET_BIN}"
fi

echo "=========================================================="
