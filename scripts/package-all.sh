#!/usr/bin/env bash
# TermExplorer Master Release Packaging Pipeline
# Orchestrates release compilation, portable tarball packaging, Debian package creation, and checksum generation.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "=========================================================="
echo " Starting Complete TermExplorer Packaging Pipeline"
echo "=========================================================="
echo "Start Time: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
echo "=========================================================="

# 1. Clean old release dist directory (except old archives if user wants, but clean ensures pure build)
mkdir -p "${REPO_ROOT}/dist"

# 2. Build Release Binary
echo ""
echo "[Step 1/4] Building production release binary..."
"${SCRIPT_DIR}/build-release.sh"

# 3. Package Portable Linux Tarball
echo ""
echo "[Step 2/4] Packaging portable Linux tarball..."
"${SCRIPT_DIR}/package-tarball.sh"

# 4. Package Debian .deb
echo ""
echo "[Step 3/4] Packaging Debian (.deb) package..."
"${SCRIPT_DIR}/package-deb.sh"

# 5. Generate Checksums and Manifest
echo ""
echo "[Step 4/4] Generating cryptographic checksums and manifest..."
"${SCRIPT_DIR}/generate-checksums.sh"

echo ""
echo "=========================================================="
echo " TermExplorer Packaging Pipeline Completed Successfully!"
echo " Artifacts located in: ${REPO_ROOT}/dist"
echo " Finish Time: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
echo "=========================================================="
