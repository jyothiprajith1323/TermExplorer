#!/usr/bin/env bash
# Generate a static APT repository structure for TermExplorer Debian packages.
# Usage: scripts/generate_apt_repo.sh <deb-file-or-dir> <repo-output-dir>

set -euo pipefail

if [[ $# -lt 2 ]]; then
  echo "Usage: $0 <deb-file-or-dir> <repo-output-dir>" >&2
  exit 2
fi

INPUT_PATH=$(realpath "$1")
REPO_DIR=$(realpath -m "$2")

mkdir -p "$REPO_DIR"

POOL_DIR="$REPO_DIR/pool/main/t/termexplorer"
DIST_DIR="$REPO_DIR/dists/stable/main/binary-amd64"
mkdir -p "$POOL_DIR" "$DIST_DIR"

# Collect deb packages
if [[ -d "$INPUT_PATH" ]]; then
  DEB_FILES=( $(find "$INPUT_PATH" -maxdepth 1 -name '*.deb') )
elif [[ -f "$INPUT_PATH" ]]; then
  DEB_FILES=("$INPUT_PATH")
else
  echo "Error: $INPUT_PATH does not exist." >&2
  exit 1
fi

if [[ ${#DEB_FILES[@]} -eq 0 ]]; then
  echo "Error: No .deb packages found in $INPUT_PATH" >&2
  exit 1
fi

for deb in "${DEB_FILES[@]}"; do
  cp -u "$deb" "$POOL_DIR/" || cp -f "$deb" "$POOL_DIR/"
done

echo "Generating Packages file..."
PACKAGES_FILE="$DIST_DIR/Packages"
: > "$PACKAGES_FILE"

for deb in "$POOL_DIR"/*.deb; do
  [[ -f "$deb" ]] || continue
  PKG_NAME=$(dpkg-deb -f "$deb" Package)
  PKG_VER=$(dpkg-deb -f "$deb" Version)
  PKG_ARCH=$(dpkg-deb -f "$deb" Architecture)
  PKG_MAINT=$(dpkg-deb -f "$deb" Maintainer)
  PKG_DEP=$(dpkg-deb -f "$deb" Depends || echo "")
  PKG_SUGG=$(dpkg-deb -f "$deb" Suggests || echo "")
  PKG_SECT=$(dpkg-deb -f "$deb" Section || echo "utils")
  PKG_PRIO=$(dpkg-deb -f "$deb" Priority || echo "optional")
  PKG_DESC=$(dpkg-deb -f "$deb" Description)
  PKG_HOME=$(dpkg-deb -f "$deb" Homepage || echo "")
  FILE_SIZE=$(stat -c%s "$deb")
  SHA256=$(sha256sum "$deb" | awk '{print $1}')
  REL_PATH="pool/main/t/termexplorer/$(basename "$deb")"

  cat >> "$PACKAGES_FILE" <<EOF
Package: $PKG_NAME
Version: $PKG_VER
Architecture: $PKG_ARCH
Maintainer: $PKG_MAINT
Installed-Size: $(dpkg-deb -f "$deb" Installed-Size || echo "15000")
Depends: $PKG_DEP
Suggests: $PKG_SUGG
Section: $PKG_SECT
Priority: $PKG_PRIO
Homepage: $PKG_HOME
Filename: $REL_PATH
Size: $FILE_SIZE
SHA256: $SHA256
Description: $PKG_DESC

EOF
done

gzip -9 -n -c "$PACKAGES_FILE" > "$DIST_DIR/Packages.gz"

echo "Generating Release file..."
RELEASE_FILE="$REPO_DIR/dists/stable/Release"
CURR_DATE=$(LC_ALL=C date -Ru)

PKGS_SIZE=$(stat -c%s "$PACKAGES_FILE")
PKGS_SHA256=$(sha256sum "$PACKAGES_FILE" | awk '{print $1}')
PKGS_GZ_SIZE=$(stat -c%s "$DIST_DIR/Packages.gz")
PKGS_GZ_SHA256=$(sha256sum "$DIST_DIR/Packages.gz" | awk '{print $1}')

cat > "$RELEASE_FILE" <<EOF
Origin: TermExplorer
Label: TermExplorer
Suite: stable
Codename: stable
Architectures: amd64
Components: main
Description: TermExplorer Development / Static APT Repository (Unsigned)
Date: $CURR_DATE
SHA256:
 $PKGS_SHA256 $PKGS_SIZE main/binary-amd64/Packages
 $PKGS_GZ_SHA256 $PKGS_GZ_SIZE main/binary-amd64/Packages.gz
EOF

# GPG Signing if key is available
if [[ -n "${GPG_KEY_ID:-}" ]] && command -v gpg >/dev/null 2>&1; then
  echo "Signing Release with GPG key: $GPG_KEY_ID"
  gpg --batch --yes --armor --detach-sign -u "$GPG_KEY_ID" -o "$REPO_DIR/dists/stable/Release.gpg" "$RELEASE_FILE"
  gpg --batch --yes --clearsign -u "$GPG_KEY_ID" -o "$REPO_DIR/dists/stable/InRelease" "$RELEASE_FILE"
else
  echo "Notice: GPG_KEY_ID not specified; repository generated unsigned."
  echo "To use this repository on client systems, specify [trusted=yes] in sources.list:"
  echo "  deb [trusted=yes] https://<host>/apt/ stable main"
fi

echo "APT repository successfully created at $REPO_DIR"
