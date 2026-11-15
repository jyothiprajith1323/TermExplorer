#!/usr/bin/env bash
# TermExplorer Portable Archive Packaging Script
# Assembles standalone tarball containing the release binary, assets, docs, completions, and installer.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${REPO_ROOT}"

VERSION="$(grep -m1 '^version = ' Cargo.toml | cut -d '"' -f 2)"
ARCH="$(uname -m)"
OS="linux"

PACKAGE_NAME="termexplorer-v${VERSION}-${OS}-${ARCH}"
DIST_DIR="${REPO_ROOT}/dist"
STAGE_DIR="${REPO_ROOT}/target/staging/${PACKAGE_NAME}"
TARBALL="${DIST_DIR}/${PACKAGE_NAME}.tar.gz"

echo "=========================================================="
echo " Packaging Portable Linux Tarball"
echo "=========================================================="
echo "Package:   ${PACKAGE_NAME}"
echo "Version:   ${VERSION}"
echo "Arch:      ${ARCH}"
echo "OS:        ${OS}"
echo "Output:    ${TARBALL}"
echo "=========================================================="

# Ensure binary is built
if [ ! -f "${REPO_ROOT}/target/release/termexplorer" ]; then
    echo "Release binary not found. Running build-release.sh first..."
    "${SCRIPT_DIR}/build-release.sh"
fi

# Clean and prepare staging directory
rm -rf "${STAGE_DIR}"
mkdir -p "${STAGE_DIR}/bin"
mkdir -p "${STAGE_DIR}/share/man/man1"
mkdir -p "${STAGE_DIR}/share/applications"
mkdir -p "${STAGE_DIR}/share/icons/hicolor/scalable/apps"
mkdir -p "${STAGE_DIR}/share/bash-completion/completions"
mkdir -p "${STAGE_DIR}/share/zsh/site-functions"
mkdir -p "${STAGE_DIR}/share/fish/vendor_completions.d"
mkdir -p "${STAGE_DIR}/share/doc/termexplorer"
mkdir -p "${DIST_DIR}"

# Copy binary
install -m 0755 "${REPO_ROOT}/target/release/termexplorer" "${STAGE_DIR}/bin/termexplorer"

# Copy packaging assets
install -m 0644 "${REPO_ROOT}/packaging/termexplorer.1" "${STAGE_DIR}/share/man/man1/termexplorer.1"
install -m 0644 "${REPO_ROOT}/packaging/termexplorer.desktop" "${STAGE_DIR}/share/applications/termexplorer.desktop"
install -m 0644 "${REPO_ROOT}/packaging/termexplorer.svg" "${STAGE_DIR}/share/icons/hicolor/scalable/apps/termexplorer.svg"
install -m 0644 "${REPO_ROOT}/packaging/completions/termexplorer.bash" "${STAGE_DIR}/share/bash-completion/completions/termexplorer"
install -m 0644 "${REPO_ROOT}/packaging/completions/_termexplorer" "${STAGE_DIR}/share/zsh/site-functions/_termexplorer"
install -m 0644 "${REPO_ROOT}/packaging/completions/termexplorer.fish" "${STAGE_DIR}/share/fish/vendor_completions.d/termexplorer.fish"

# Copy documentation and licenses
install -m 0644 "${REPO_ROOT}/README.md" "${STAGE_DIR}/share/doc/termexplorer/README.md"
install -m 0644 "${REPO_ROOT}/LICENSE" "${STAGE_DIR}/share/doc/termexplorer/LICENSE"

# Copy standalone install / uninstall scripts
install -m 0755 "${REPO_ROOT}/packaging/install.sh" "${STAGE_DIR}/install.sh"
install -m 0755 "${REPO_ROOT}/packaging/uninstall.sh" "${STAGE_DIR}/uninstall.sh"

# Build reproducible tarball
echo "Assembling tarball with normalized timestamps and permissions..."
rm -f "${TARBALL}"

# Use tar with normalized options for reproducibility
SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-$(git log -1 --pretty=%ct 2>/dev/null || stat -c %Y "${REPO_ROOT}/Cargo.toml")}"
DATE_FORMAT="@${SOURCE_DATE_EPOCH}"

tar -C "${REPO_ROOT}/target/staging" \
    --owner=0 --group=0 --numeric-owner \
    --mtime="${DATE_FORMAT}" \
    --sort=name \
    -czf "${TARBALL}" \
    "${PACKAGE_NAME}"

ARCHIVE_SIZE=$(stat -c%s "${TARBALL}")
ARCHIVE_SIZE_HUMAN=$(numfmt --to=iec-i --suffix=B "${ARCHIVE_SIZE}" 2>/dev/null || ls -lh "${TARBALL}" | awk '{print $5}')
SHA256=$(sha256sum "${TARBALL}" | awk '{print $1}')

echo ""
echo "Portable Archive Created Successfully:"
echo "  Path:     ${TARBALL}"
echo "  Size:     ${ARCHIVE_SIZE} bytes (${ARCHIVE_SIZE_HUMAN})"
echo "  SHA-256:  ${SHA256}"
echo "=========================================================="
