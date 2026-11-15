#!/usr/bin/env bash
# TermExplorer Debian Package (.deb) Packaging Script
# Assembles standard Debian binary package adhering to Debian Policy.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${REPO_ROOT}"

VERSION="$(grep -m1 '^version = ' Cargo.toml | cut -d '"' -f 2)"
HOST_ARCH="$(uname -m)"

case "${HOST_ARCH}" in
    x86_64)
        DEB_ARCH="amd64"
        ;;
    aarch64|arm64)
        DEB_ARCH="arm64"
        ;;
    *)
        DEB_ARCH="${HOST_ARCH}"
        ;;
esac

PACKAGE_NAME="termexplorer"
DEB_FILE="${REPO_ROOT}/dist/${PACKAGE_NAME}_${VERSION}_${DEB_ARCH}.deb"
STAGE_DIR="${REPO_ROOT}/target/staging/debian-${PACKAGE_NAME}"

echo "=========================================================="
echo " Packaging Debian Package (.deb)"
echo "=========================================================="
echo "Package:       ${PACKAGE_NAME}"
echo "Version:       ${VERSION}"
echo "Architecture:  ${DEB_ARCH}"
echo "Output:        ${DEB_FILE}"
echo "=========================================================="

# Ensure binary is built
if [ ! -f "${REPO_ROOT}/target/release/termexplorer" ]; then
    echo "Release binary not found. Running build-release.sh first..."
    "${SCRIPT_DIR}/build-release.sh"
fi

if ! command -v dpkg-deb >/dev/null 2>&1; then
    echo "Error: 'dpkg-deb' utility is required to build .deb packages." >&2
    exit 1
fi

rm -rf "${STAGE_DIR}"
mkdir -p "${STAGE_DIR}/DEBIAN"
mkdir -p "${STAGE_DIR}/usr/bin"
mkdir -p "${STAGE_DIR}/usr/share/man/man1"
mkdir -p "${STAGE_DIR}/usr/share/applications"
mkdir -p "${STAGE_DIR}/usr/share/icons/hicolor/scalable/apps"
mkdir -p "${STAGE_DIR}/usr/share/bash-completion/completions"
mkdir -p "${STAGE_DIR}/usr/share/zsh/vendor-completions"
mkdir -p "${STAGE_DIR}/usr/share/fish/vendor_completions.d"
mkdir -p "${STAGE_DIR}/usr/share/doc/${PACKAGE_NAME}"
mkdir -p "${REPO_ROOT}/dist"

# Install executable
install -m 0755 "${REPO_ROOT}/target/release/termexplorer" "${STAGE_DIR}/usr/bin/termexplorer"

# Compress and install manual page
gzip -9 -n -c "${REPO_ROOT}/packaging/termexplorer.1" > "${STAGE_DIR}/usr/share/man/man1/termexplorer.1.gz"
chmod 0644 "${STAGE_DIR}/usr/share/man/man1/termexplorer.1.gz"

# Desktop integration & icon
install -m 0644 "${REPO_ROOT}/packaging/termexplorer.desktop" "${STAGE_DIR}/usr/share/applications/termexplorer.desktop"
install -m 0644 "${REPO_ROOT}/packaging/termexplorer.svg" "${STAGE_DIR}/usr/share/icons/hicolor/scalable/apps/termexplorer.svg"

# Shell completions
install -m 0644 "${REPO_ROOT}/packaging/completions/termexplorer.bash" "${STAGE_DIR}/usr/share/bash-completion/completions/termexplorer"
install -m 0644 "${REPO_ROOT}/packaging/completions/_termexplorer" "${STAGE_DIR}/usr/share/zsh/vendor-completions/_termexplorer"
install -m 0644 "${REPO_ROOT}/packaging/completions/termexplorer.fish" "${STAGE_DIR}/usr/share/fish/vendor_completions.d/termexplorer.fish"

# Documentation
install -m 0644 "${REPO_ROOT}/README.md" "${STAGE_DIR}/usr/share/doc/${PACKAGE_NAME}/README.md"

# Copyright file (Debian machine-readable copyright format 1.0)
cat > "${STAGE_DIR}/usr/share/doc/${PACKAGE_NAME}/copyright" <<EOF
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: termexplorer
Upstream-Contact: Jyothi Prajith <jyothiprajith1323@users.noreply.github.com>
Source: https://github.com/jyothiprajith1323/TermExplorer

Files: *
Copyright: 2024-2026 Jyothi Prajith and TermExplorer Contributors
License: MIT
 Permission is hereby granted, free of charge, to any person obtaining a copy
 of this software and associated documentation files (the "Software"), to deal
 in the Software without restriction, including without limitation the rights
 to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 copies of the Software, and to permit persons to whom the Software is
 furnished to do so, subject to the following conditions:
 .
 The above copyright notice and this permission notice shall be included in all
 copies or substantial portions of the Software.
 .
 THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 SOFTWARE.
EOF
chmod 0644 "${STAGE_DIR}/usr/share/doc/${PACKAGE_NAME}/copyright"

# Changelog
cat > "${STAGE_DIR}/usr/share/doc/${PACKAGE_NAME}/changelog.Debian" <<EOF
termexplorer (${VERSION}) stable; urgency=medium

  * Sprint 6A Release Packaging: Portable archive and Debian package.
  * WebAssembly plugin sandbox architecture and runtime support.
  * High-performance 2-pane TUI workstation (25% Explorer / 75% Preview).
  * Embedded PTY interactive shell drawer (15% height).
  * Native Git status, diff viewer, and branch manager.
  * Multi-format preview engine (Markdown, Code, Images, PDF, Media).

 -- Jyothi Prajith <jyothiprajith1323@users.noreply.github.com>  $(date -R)
EOF
gzip -9 -n "${STAGE_DIR}/usr/share/doc/${PACKAGE_NAME}/changelog.Debian"
chmod 0644 "${STAGE_DIR}/usr/share/doc/${PACKAGE_NAME}/changelog.Debian.gz"

# Calculate Installed-Size in KiB
INSTALLED_SIZE=$(du -sk --apparent-size "${STAGE_DIR}/usr" | awk '{print $1}')

# Generate control file
cat > "${STAGE_DIR}/DEBIAN/control" <<EOF
Package: ${PACKAGE_NAME}
Version: ${VERSION}
Architecture: ${DEB_ARCH}
Maintainer: Jyothi Prajith <jyothiprajith1323@users.noreply.github.com>
Installed-Size: ${INSTALLED_SIZE}
Depends: libc6 (>= 2.34), libgcc-s1
Recommends: git
Suggests: ffmpeg, poppler-utils, sqlite3
Section: utils
Priority: optional
Homepage: https://github.com/jyothiprajith1323/TermExplorer
Description: Keyboard-driven terminal developer workstation in pure Rust
 TermExplorer is a lightweight, modal developer workstation for Unix terminals.
 It integrates a 2-pane workspace combining a 25% hierarchical filesystem
 explorer with a 75% inspector and previewer pane, an embedded PTY interactive
 shell drawer, native Git status and branch visualization, project-aware task
 runner, and a secure WebAssembly plugin sandbox.
EOF
chmod 0644 "${STAGE_DIR}/DEBIAN/control"

# Generate MD5 sums for packaging integrity
(
    cd "${STAGE_DIR}"
    find usr -type f -exec md5sum {} + > DEBIAN/md5sums
    chmod 0644 DEBIAN/md5sums
)

# Build debian package
echo "Building Debian package with dpkg-deb..."
rm -f "${DEB_FILE}"
dpkg-deb --build --root-owner-group "${STAGE_DIR}" "${DEB_FILE}"

DEB_SIZE=$(stat -c%s "${DEB_FILE}")
DEB_SIZE_HUMAN=$(numfmt --to=iec-i --suffix=B "${DEB_SIZE}" 2>/dev/null || ls -lh "${DEB_FILE}" | awk '{print $5}')
SHA256=$(sha256sum "${DEB_FILE}" | awk '{print $1}')

echo ""
echo "Debian Package Created Successfully:"
echo "  Path:     ${DEB_FILE}"
echo "  Size:     ${DEB_SIZE} bytes (${DEB_SIZE_HUMAN})"
echo "  SHA-256:  ${SHA256}"
echo ""
echo "Debian Package Info:"
dpkg-deb --info "${DEB_FILE}"
echo "=========================================================="
