#!/usr/bin/env bash
# TermExplorer Master Release Packaging Pipeline
# Orchestrates release compilation, package assembly (.deb, .rpm, .tar.gz), and checksum generation.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "=========================================================="
echo " Starting Complete TermExplorer Packaging Pipeline"
echo "=========================================================="
echo "Start Time: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
echo "=========================================================="

mkdir -p "${REPO_ROOT}/dist"

# 1. Build Release Binary
if [ ! -f "${REPO_ROOT}/target/release/termexplorer" ]; then
    echo ""
    echo "[Step 1/2] Building production release binary..."
    "${SCRIPT_DIR}/build-release.sh"
else
    echo ""
    echo "[Step 1/2] Using existing release binary: ${REPO_ROOT}/target/release/termexplorer"
fi

# 2. Execute consolidated packaging pipeline
VERSION="$(grep -m1 '^version = ' "${REPO_ROOT}/Cargo.toml" | cut -d '"' -f 2)"
echo ""
echo "[Step 2/2] Executing package_linux.sh for version ${VERSION}..."
"${SCRIPT_DIR}/package_linux.sh" "${REPO_ROOT}/target/release/termexplorer" "${VERSION}" "${REPO_ROOT}/dist"

echo ""
echo "=========================================================="
echo " TermExplorer Packaging Pipeline Completed Successfully!"
echo " Artifacts located in: ${REPO_ROOT}/dist"
echo " Finish Time: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
echo "=========================================================="
