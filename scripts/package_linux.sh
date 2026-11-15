#!/usr/bin/env bash
set -euo pipefail

# Build installable Debian and RPM packages from an already-built release binary.
# Usage: scripts/package_linux.sh <binary> <version> <output-directory>

if [[ $# -ne 3 ]]; then
  echo "Usage: $0 <binary> <version> <output-directory>" >&2
  exit 2
fi

BINARY=$(realpath "$1")
VERSION=$2
OUTPUT=$(realpath -m "$3")
ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)

[[ -f "$BINARY" && -x "$BINARY" ]] || { echo "Release binary is missing or not executable: $BINARY" >&2; exit 1; }
[[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-][A-Za-z0-9.]+)?$ ]] || { echo "Invalid package version: $VERSION" >&2; exit 1; }
command -v dpkg-deb >/dev/null || { echo "dpkg-deb is required to build the Debian package." >&2; exit 1; }
command -v rpmbuild >/dev/null || { echo "rpmbuild is required to build the RPM package." >&2; exit 1; }

DEB_VERSION=${VERSION/-/\~}

MACHINE=$(uname -m)
case "$MACHINE" in
  x86_64) DEB_ARCH=amd64; RPM_ARCH=x86_64 ;;
  aarch64|arm64) DEB_ARCH=arm64; RPM_ARCH=aarch64 ;;
  *) echo "Unsupported package architecture: $MACHINE" >&2; exit 1 ;;
esac

mkdir -p "$OUTPUT"
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

# Debian package layout and metadata.
DEB_ROOT="$WORK/deb-root"
install -Dm755 "$BINARY" "$DEB_ROOT/usr/bin/termexplorer"
install -Dm644 "$ROOT/LICENSE" "$DEB_ROOT/usr/share/doc/termexplorer/copyright"
[[ ! -f "$ROOT/README.md" ]] || install -Dm644 "$ROOT/README.md" "$DEB_ROOT/usr/share/doc/termexplorer/README.md"
[[ ! -f "$ROOT/INSTALLATION.md" ]] || install -Dm644 "$ROOT/INSTALLATION.md" "$DEB_ROOT/usr/share/doc/termexplorer/INSTALLATION.md"
[[ ! -f "$ROOT/packaging/termexplorer.1" ]] || install -Dm644 "$ROOT/packaging/termexplorer.1" "$DEB_ROOT/usr/share/man/man1/termexplorer.1"
mkdir -p "$DEB_ROOT/DEBIAN"
cat > "$DEB_ROOT/DEBIAN/control" <<CONTROL
Package: termexplorer
Version: $DEB_VERSION
Section: utils
Priority: optional
Architecture: $DEB_ARCH
Depends: libc6, libgcc-s1
Suggests: git, ffmpeg, poppler-utils, trash-cli
Maintainer: TermExplorer Contributors <101039137+jyothiprajith1323@users.noreply.github.com>
Description: Rust terminal developer workstation
 Inspection-first terminal developer workstation combining file exploration,
 contextual previews, editing and developer workflows.
Homepage: https://github.com/jyothiprajith1323/TermExplorer
CONTROL
chmod 0755 "$DEB_ROOT/DEBIAN"
chmod 0644 "$DEB_ROOT/DEBIAN/control"
dpkg-deb --root-owner-group --build "$DEB_ROOT" "$OUTPUT/termexplorer_${DEB_VERSION}_${DEB_ARCH}.deb" >/dev/null

# RPM package layout. Tilde orders pre-release builds before the final version.
RPM_VERSION=${VERSION/-/\~}
RPM_CHANGELOG_DATE=$(LC_ALL=C date "+%a %b %d %Y")
RPM_TOP="$WORK/rpmbuild"
mkdir -p "$RPM_TOP/BUILD" "$RPM_TOP/BUILDROOT" "$RPM_TOP/RPMS" "$RPM_TOP/SOURCES" "$RPM_TOP/SPECS" "$RPM_TOP/SRPMS"
cp "$BINARY" "$RPM_TOP/SOURCES/termexplorer"
cp "$ROOT/LICENSE" "$RPM_TOP/SOURCES/LICENSE"
if [[ -f "$ROOT/packaging/termexplorer.1" ]]; then cp "$ROOT/packaging/termexplorer.1" "$RPM_TOP/SOURCES/termexplorer.1"; fi
cat > "$RPM_TOP/SPECS/termexplorer.spec" <<SPEC
Name:           termexplorer
Version:        $RPM_VERSION
Release:        1
Summary:        Rust terminal developer workstation
License:        MIT
URL:            https://github.com/jyothiprajith1323/TermExplorer
BuildArch:      $RPM_ARCH
# rpmbuild's ELF dependency generator adds SONAME-based Requires so
# each RPM distribution can resolve its own provider package names.
Suggests:       git
Suggests:       ffmpeg
Suggests:       poppler-utils

%description
Inspection-first terminal developer workstation combining file exploration,
contextual previews, editing and developer workflows.

%prep

%build

%install
rm -rf %{buildroot}
install -Dpm0755 %{_sourcedir}/termexplorer %{buildroot}%{_bindir}/termexplorer
install -Dpm0644 %{_sourcedir}/LICENSE %{buildroot}%{_licensedir}/%{name}/LICENSE
if [ -f %{_sourcedir}/termexplorer.1 ]; then install -Dpm0644 %{_sourcedir}/termexplorer.1 %{buildroot}%{_mandir}/man1/termexplorer.1; fi

%files
%{_bindir}/termexplorer
%{_mandir}/man1/termexplorer.1
%license %{_licensedir}/%{name}/LICENSE

%changelog
* $RPM_CHANGELOG_DATE TermExplorer Contributors - $RPM_VERSION-1
- Build release package.
SPEC
rpmbuild --define "_topdir $RPM_TOP" --define "_rpmdir $RPM_TOP/RPMS" -bb "$RPM_TOP/SPECS/termexplorer.spec" >/dev/null
RPM_FILE=$(find "$RPM_TOP/RPMS" -type f -name '*.rpm' -print -quit)
[[ -n "$RPM_FILE" ]] || { echo "RPM build did not produce an artifact." >&2; exit 1; }
cp "$RPM_FILE" "$OUTPUT/"

(
  cd "$OUTPUT"
  sha256sum "termexplorer_${DEB_VERSION}_${DEB_ARCH}.deb" "$(basename "$RPM_FILE")" > PACKAGES_SHA256SUMS
)

echo "Created packages in $OUTPUT"
ls -lh "$OUTPUT"/termexplorer_"$DEB_VERSION"_"$DEB_ARCH".deb "$OUTPUT"/*.rpm "$OUTPUT"/PACKAGES_SHA256SUMS
