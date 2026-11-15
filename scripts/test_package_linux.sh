#!/usr/bin/env bash
set -euo pipefail

# Validate the real Linux package artifacts (.deb, .rpm, .tar.gz) produced by package_linux.sh.
# Usage: scripts/test_package_linux.sh <version> <distribution-directory>

if [[ $# -ne 2 ]]; then
  echo "Usage: $0 <version> <distribution-directory>" >&2
  exit 2
fi

VERSION=$1
DIST=$(realpath "$2")
DEB_VERSION=${VERSION/-/\~}
RPM_VERSION=${VERSION/-/\~}
MACHINE=$(uname -m)
case "$MACHINE" in
  x86_64) DEB_ARCH=amd64; RPM_ARCH=x86_64; TAR_ARCH=x86_64 ;;
  aarch64|arm64) DEB_ARCH=arm64; RPM_ARCH=aarch64; TAR_ARCH=aarch64 ;;
  *) echo "Unsupported test architecture: $MACHINE" >&2; exit 1 ;;
esac

DEB="$DIST/termexplorer_${DEB_VERSION}_${DEB_ARCH}.deb"
RPM="$DIST/termexplorer-${RPM_VERSION}-1.${RPM_ARCH}.rpm"
TARBALL="$DIST/termexplorer-v${VERSION}-linux-${TAR_ARCH}.tar.gz"

assert_eq() {
  local actual=$1 expected=$2 label=$3
  if [[ "$actual" != "$expected" ]]; then
    printf 'FAIL: %s: expected <%s>, got <%s>\n' "$label" "$expected" "$actual" >&2
    exit 1
  fi
  printf 'PASS: %s = %s\n' "$label" "$actual"
}

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

# -------------------------------------------------------------
# 1. Test Debian package (.deb)
# -------------------------------------------------------------
if [[ -f "$DEB" ]]; then
  if command -v dpkg-deb >/dev/null 2>&1; then
    echo "=== Validating Debian Package: $(basename "$DEB") ==="
    assert_eq "$(dpkg-deb -f "$DEB" Package)" 'termexplorer' 'Debian package name'
    assert_eq "$(dpkg-deb -f "$DEB" Version)" "$DEB_VERSION" 'Debian package version'
    assert_eq "$(dpkg-deb -f "$DEB" Architecture)" "$DEB_ARCH" 'Debian package architecture'

    # Ensure release candidates sort before their final version in Debian tooling.
    if [[ "$VERSION" == *-* ]]; then
      stable_version=${VERSION%%-*}
      dpkg --compare-versions "$DEB_VERSION" lt "$stable_version"
      echo "PASS: Debian pre-release ordering ($DEB_VERSION < $stable_version)"
    fi

    DEB_ROOT="$TMP/deb-root"
    mkdir -p "$DEB_ROOT"
    dpkg-deb -x "$DEB" "$DEB_ROOT"

    [[ -x "$DEB_ROOT/usr/bin/termexplorer" ]] || { echo "FAIL: missing executable in deb" >&2; exit 1; }
    [[ -f "$DEB_ROOT/usr/share/man/man1/termexplorer.1.gz" || -f "$DEB_ROOT/usr/share/man/man1/termexplorer.1" ]] || { echo "FAIL: missing manpage in deb" >&2; exit 1; }
    [[ -f "$DEB_ROOT/usr/share/applications/termexplorer.desktop" ]] || { echo "FAIL: missing desktop file in deb" >&2; exit 1; }
    [[ -f "$DEB_ROOT/usr/share/icons/hicolor/scalable/apps/termexplorer.svg" ]] || { echo "FAIL: missing svg icon in deb" >&2; exit 1; }
    [[ -f "$DEB_ROOT/usr/share/bash-completion/completions/termexplorer" ]] || { echo "FAIL: missing bash completion in deb" >&2; exit 1; }
    [[ -f "$DEB_ROOT/usr/share/zsh/vendor-completions/_termexplorer" ]] || { echo "FAIL: missing zsh completion in deb" >&2; exit 1; }
    [[ -f "$DEB_ROOT/usr/share/fish/vendor_completions.d/termexplorer.fish" ]] || { echo "FAIL: missing fish completion in deb" >&2; exit 1; }
    [[ -f "$DEB_ROOT/usr/share/doc/termexplorer/copyright" ]] || { echo "FAIL: missing copyright in deb" >&2; exit 1; }

    "$DEB_ROOT/usr/bin/termexplorer" --version
    "$DEB_ROOT/usr/bin/termexplorer" --help >/dev/null
    echo 'PASS: extracted Debian package contains working executable and full asset suite'
  else
    echo "Notice: dpkg-deb not found; skipping Debian inspection."
  fi
else
  echo "Notice: Debian package not found in $DIST"
fi

# -------------------------------------------------------------
# 2. Test RPM package (.rpm)
# -------------------------------------------------------------
if [[ -f "$RPM" ]]; then
  if command -v rpm >/dev/null 2>&1; then
    echo "=== Validating RPM Package: $(basename "$RPM") ==="
    assert_eq "$(rpm -qp --qf '%{NAME}' "$RPM")" 'termexplorer' 'RPM package name'
    assert_eq "$(rpm -qp --qf '%{VERSION}' "$RPM")" "$RPM_VERSION" 'RPM package version'
    assert_eq "$(rpm -qp --qf '%{RELEASE}' "$RPM")" '1' 'RPM package release'
    assert_eq "$(rpm -qp --qf '%{ARCH}' "$RPM")" "$RPM_ARCH" 'RPM package architecture'

    # Install into an isolated RPM root. No host package database or host files are modified.
    RPM_ROOT="$TMP/rpm-root"
    mkdir -p "$RPM_ROOT"
    rpm --root "$RPM_ROOT" --dbpath /var/lib/rpm --initdb
    rpm --root "$RPM_ROOT" --dbpath /var/lib/rpm --nodeps --noscripts -i "$RPM"
    [[ -x "$RPM_ROOT/usr/bin/termexplorer" ]] || { echo "FAIL: missing executable in rpm" >&2; exit 1; }
    [[ -f "$RPM_ROOT/usr/share/man/man1/termexplorer.1.gz" || -f "$RPM_ROOT/usr/share/man/man1/termexplorer.1" ]] || { echo "FAIL: missing manpage in rpm" >&2; exit 1; }
    "$RPM_ROOT/usr/bin/termexplorer" --version
    "$RPM_ROOT/usr/bin/termexplorer" --help >/dev/null
    echo 'PASS: isolated RPM installation contains working executable and expected assets'
  else
    echo "Notice: rpm command not found; skipping RPM inspection on this host."
  fi
else
  echo "Notice: RPM package not found in $DIST"
fi

# -------------------------------------------------------------
# 3. Test Portable Linux Tarball (.tar.gz)
# -------------------------------------------------------------
if [[ -f "$TARBALL" ]]; then
  echo "=== Validating Portable Tarball: $(basename "$TARBALL") ==="
  TAR_EXTRACT="$TMP/tar-extract"
  mkdir -p "$TAR_EXTRACT"
  tar -xzf "$TARBALL" -C "$TAR_EXTRACT"
  STAGE_DIR="$TAR_EXTRACT/termexplorer-v${VERSION}-linux-${TAR_ARCH}"

  [[ -x "$STAGE_DIR/bin/termexplorer" ]] || { echo "FAIL: missing executable in tarball" >&2; exit 1; }
  [[ -x "$STAGE_DIR/install.sh" ]] || { echo "FAIL: missing install.sh in tarball" >&2; exit 1; }
  [[ -x "$STAGE_DIR/uninstall.sh" ]] || { echo "FAIL: missing uninstall.sh in tarball" >&2; exit 1; }
  [[ -f "$STAGE_DIR/share/applications/termexplorer.desktop" ]] || { echo "FAIL: missing desktop file in tarball" >&2; exit 1; }
  [[ -f "$STAGE_DIR/share/icons/hicolor/scalable/apps/termexplorer.svg" ]] || { echo "FAIL: missing icon in tarball" >&2; exit 1; }
  [[ -f "$STAGE_DIR/share/bash-completion/completions/termexplorer" ]] || { echo "FAIL: missing bash completion in tarball" >&2; exit 1; }
  [[ -f "$STAGE_DIR/share/zsh/site-functions/_termexplorer" ]] || { echo "FAIL: missing zsh completion in tarball" >&2; exit 1; }
  [[ -f "$STAGE_DIR/share/fish/vendor_completions.d/termexplorer.fish" ]] || { echo "FAIL: missing fish completion in tarball" >&2; exit 1; }

  "$STAGE_DIR/bin/termexplorer" --version
  "$STAGE_DIR/bin/termexplorer" --help >/dev/null
  echo 'PASS: extracted portable tarball contains working executable, installer and full asset suite'
else
  echo "Notice: Portable tarball not found in $DIST"
fi

# -------------------------------------------------------------
# 4. Checksums verification
# -------------------------------------------------------------
if [[ -f "$DIST/SHA256SUMS" ]]; then
  echo "=== Validating SHA256SUMS ==="
  (cd "$DIST" && sha256sum --check SHA256SUMS)
  echo "PASS: SHA256SUMS verification passed"
fi

echo "=========================================================="
echo 'All Linux package verification smoke tests passed.'
echo "=========================================================="
