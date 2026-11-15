#!/usr/bin/env bash
# TermExplorer Checksum and Manifest Generator
# Generates cryptographic SHA-256 checksums and comprehensive artifact manifest for release artifacts.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
DIST_DIR="${REPO_ROOT}/dist"

cd "${DIST_DIR}"

echo "=========================================================="
echo " Generating Cryptographic Checksums & Artifact Manifest"
echo "=========================================================="
echo "Directory: ${DIST_DIR}"
echo "Timestamp: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
echo "=========================================================="

# Remove old checksum files before regenerating
rm -f SHA256SUMS MANIFEST.txt *.sha256

# Collect release artifacts (tar.gz, deb)
ARTIFACTS=$(find . -maxdepth 1 -type f \( -name "*.tar.gz" -o -name "*.deb" \) -exec basename {} \; | sort)

if [ -z "${ARTIFACTS}" ]; then
    echo "Warning: No release artifacts (*.tar.gz, *.deb) found in ${DIST_DIR}" >&2
    exit 1
fi

echo "Artifacts to hash:"
for art in ${ARTIFACTS}; do
    echo "  - ${art}"
done

# Generate SHA256SUMS
sha256sum ${ARTIFACTS} > SHA256SUMS

# Also create individual .sha256 files for convenience
for art in ${ARTIFACTS}; do
    sha256sum "${art}" > "${art}.sha256"
done

# Generate detailed release MANIFEST.txt
MANIFEST="${DIST_DIR}/MANIFEST.txt"
cat > "${MANIFEST}" <<EOF
# TermExplorer Release Artifact Manifest
# Generated at: $(date -u +"%Y-%m-%dT%H:%M:%SZ")
# Repository:   https://github.com/jyothiprajith1323/TermExplorer
# Git Commit:   $(git -C "${REPO_ROOT}" rev-parse HEAD 2>/dev/null || echo "unknown")
# Rust Toolchain: $(rustc --version)

| Artifact Filename | Format | Arch | Size (Bytes) | SHA-256 Checksum |
|-------------------|--------|------|--------------|-------------------|
EOF

for art in ${ARTIFACTS}; do
    SIZE=$(stat -c%s "${art}")
    SHA=$(sha256sum "${art}" | awk '{print $1}')
    case "${art}" in
        *.tar.gz)
            FMT="tar.gz"
            ARCH=$(echo "${art}" | sed -E 's/.*-([^-]+)\.tar\.gz/\1/')
            ;;
        *.deb)
            FMT="deb"
            ARCH=$(echo "${art}" | sed -E 's/.*_([^_]+)\.deb/\1/')
            ;;
        *)
            FMT="binary"
            ARCH="unknown"
            ;;
    esac
    printf "| %-35s | %-6s | %-7s | %12d | %s |\n" "${art}" "${FMT}" "${ARCH}" "${SIZE}" "${SHA}" >> "${MANIFEST}"
done

echo ""
echo "Checksum verification:"
sha256sum -c SHA256SUMS

echo ""
echo "Manifest successfully written to: ${MANIFEST}"
cat "${MANIFEST}"
echo "=========================================================="
