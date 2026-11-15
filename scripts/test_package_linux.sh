#!/usr/bin/env bash
set -euo pipefail

# Validate the real Debian and RPM package artifacts produced by package_linux.sh.
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
  x86_64) DEB_ARCH=amd64; RPM_ARCH=x86_64 ;;
  aarch64|arm64) DEB_ARCH=arm64; RPM_ARCH=aarch64 ;;
  *) echo "Unsupported test architecture: $MACHINE" >&2; exit 1 ;;
esac

DEB="$DIST/termexplorer_${DEB_VERSION}_${DEB_ARCH}.deb"
RPM="$DIST/termexplorer-${RPM_VERSION}-1.${RPM_ARCH}.rpm"
[[ -f "$DEB" ]] || { echo "Missing Debian package: $DEB" >&2; exit 1; }
[[ -f "$RPM" ]] || { echo "Missing RPM package: $RPM" >&2; exit 1; }
command -v dpkg-deb >/dev/null || { echo 'dpkg-deb is required.' >&2; exit 1; }
command -v rpm >/dev/null || { echo 'rpm is required.' >&2; exit 1; }

assert_eq() {
  local actual=$1 expected=$2 label=$3
  if [[ "$actual" != "$expected" ]]; then
    printf 'FAIL: %s: expected <%s>, got <%s>\n' "$label" "$expected" "$actual" >&2
    exit 1
  fi
  printf 'PASS: %s = %s\n' "$label" "$actual"
}

assert_eq "$(dpkg-deb -f "$DEB" Package)" 'termexplorer' 'Debian package name'
assert_eq "$(dpkg-deb -f "$DEB" Version)" "$DEB_VERSION" 'Debian package version'
assert_eq "$(dpkg-deb -f "$DEB" Architecture)" "$DEB_ARCH" 'Debian package architecture'
assert_eq "$(rpm -qp --qf '%{NAME}' "$RPM")" 'termexplorer' 'RPM package name'
assert_eq "$(rpm -qp --qf '%{VERSION}' "$RPM")" "$RPM_VERSION" 'RPM package version'
assert_eq "$(rpm -qp --qf '%{RELEASE}' "$RPM")" '1' 'RPM package release'
assert_eq "$(rpm -qp --qf '%{ARCH}' "$RPM")" "$RPM_ARCH" 'RPM package architecture'

# Ensure release candidates sort before their final version in Debian tooling.
if [[ "$VERSION" == *-* ]]; then
  stable_version=${VERSION%%-*}
  dpkg --compare-versions "$DEB_VERSION" lt "$stable_version"
  echo "PASS: Debian pre-release ordering ($DEB_VERSION < $stable_version)"
fi

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
DEB_ROOT="$TMP/deb-root"
mkdir -p "$DEB_ROOT"
dpkg-deb -x "$DEB" "$DEB_ROOT"
[[ -x "$DEB_ROOT/usr/bin/termexplorer" ]]
[[ -f "$DEB_ROOT/usr/share/man/man1/termexplorer.1" ]]
[[ -f "$DEB_ROOT/usr/share/doc/termexplorer/copyright" ]]
"$DEB_ROOT/usr/bin/termexplorer" --version
"$DEB_ROOT/usr/bin/termexplorer" --help >/dev/null
echo 'PASS: extracted Debian package contains a working executable and expected documentation'

# Install into an isolated RPM root. No host package database or host files are modified.
RPM_ROOT="$TMP/rpm-root"
mkdir -p "$RPM_ROOT"
rpm --root "$RPM_ROOT" --dbpath /var/lib/rpm --initdb
rpm --root "$RPM_ROOT" --dbpath /var/lib/rpm --nodeps --noscripts -i "$RPM"
[[ -x "$RPM_ROOT/usr/bin/termexplorer" ]]
[[ -f "$RPM_ROOT/usr/share/man/man1/termexplorer.1" ]]
"$RPM_ROOT/usr/bin/termexplorer" --version
"$RPM_ROOT/usr/bin/termexplorer" --help >/dev/null
echo 'PASS: isolated RPM installation contains a working executable and expected man page'

echo 'All Linux package smoke tests passed.'
