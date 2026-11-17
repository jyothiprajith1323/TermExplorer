#!/usr/bin/env bash
set -euo pipefail

# Build installable Debian, RPM and Portable Tarball packages from an already-built release binary.
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

DEB_VERSION=${VERSION/-/\~}
RPM_VERSION=${VERSION/-/\~}

MACHINE=$(uname -m)
case "$MACHINE" in
  x86_64) DEB_ARCH=amd64; RPM_ARCH=x86_64; TAR_ARCH=x86_64 ;;
  aarch64|arm64) DEB_ARCH=arm64; RPM_ARCH=aarch64; TAR_ARCH=aarch64 ;;
  *) echo "Unsupported package architecture: $MACHINE" >&2; exit 1 ;;
esac

mkdir -p "$OUTPUT"
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

# Prepare compressed manpage
MANPAGE_GZ="$WORK/termexplorer.1.gz"
if [[ -f "$ROOT/packaging/termexplorer.1" ]]; then
  gzip -9 -n -c "$ROOT/packaging/termexplorer.1" > "$MANPAGE_GZ"
fi

# Prepare Debian changelog
CHANGELOG_GZ="$WORK/changelog.Debian.gz"
cat > "$WORK/changelog.Debian" <<EOF
termexplorer (${DEB_VERSION}) stable; urgency=medium

  * Release version ${VERSION}.
  * Complete terminal developer workstation with file exploration, editor, and preview.
  * Shell completions for Bash, Zsh, and Fish.

 -- TermExplorer Contributors <101039137+jyothiprajith1323@users.noreply.github.com>  $(LC_ALL=C date -R)
EOF
gzip -9 -n -c "$WORK/changelog.Debian" > "$CHANGELOG_GZ"

# -------------------------------------------------------------
# 1. Debian Package (.deb)
# -------------------------------------------------------------
if command -v dpkg-deb >/dev/null 2>&1; then
  echo "Building Debian package (.deb)..."
  DEB_ROOT="$WORK/deb-root"
  mkdir -p "$DEB_ROOT/usr/bin"
  mkdir -p "$DEB_ROOT/usr/share/man/man1"
  mkdir -p "$DEB_ROOT/usr/share/applications"
  mkdir -p "$DEB_ROOT/usr/share/icons/hicolor/scalable/apps"
  mkdir -p "$DEB_ROOT/usr/share/bash-completion/completions"
  mkdir -p "$DEB_ROOT/usr/share/zsh/vendor-completions"
  mkdir -p "$DEB_ROOT/usr/share/fish/vendor_completions.d"
  mkdir -p "$DEB_ROOT/usr/share/doc/termexplorer"

  # Executable
  install -Dm755 "$BINARY" "$DEB_ROOT/usr/bin/termexplorer"

  # Manpage
  if [[ -f "$MANPAGE_GZ" ]]; then
    install -Dm644 "$MANPAGE_GZ" "$DEB_ROOT/usr/share/man/man1/termexplorer.1.gz"
  fi

  # Desktop entry & Icon
  if [[ -f "$ROOT/packaging/termexplorer.desktop" ]]; then
    install -Dm644 "$ROOT/packaging/termexplorer.desktop" "$DEB_ROOT/usr/share/applications/termexplorer.desktop"
  fi
  if [[ -f "$ROOT/packaging/termexplorer.svg" ]]; then
    install -Dm644 "$ROOT/packaging/termexplorer.svg" "$DEB_ROOT/usr/share/icons/hicolor/scalable/apps/termexplorer.svg"
  fi

  # Shell completions
  if [[ -f "$ROOT/packaging/completions/termexplorer.bash" ]]; then
    install -Dm644 "$ROOT/packaging/completions/termexplorer.bash" "$DEB_ROOT/usr/share/bash-completion/completions/termexplorer"
  fi
  if [[ -f "$ROOT/packaging/completions/_termexplorer" ]]; then
    install -Dm644 "$ROOT/packaging/completions/_termexplorer" "$DEB_ROOT/usr/share/zsh/vendor-completions/_termexplorer"
  fi
  if [[ -f "$ROOT/packaging/completions/termexplorer.fish" ]]; then
    install -Dm644 "$ROOT/packaging/completions/termexplorer.fish" "$DEB_ROOT/usr/share/fish/vendor_completions.d/termexplorer.fish"
  fi

  # Documentation & License
  install -Dm644 "$ROOT/LICENSE" "$DEB_ROOT/usr/share/doc/termexplorer/copyright"
  [[ ! -f "$ROOT/README.md" ]] || install -Dm644 "$ROOT/README.md" "$DEB_ROOT/usr/share/doc/termexplorer/README.md"
  [[ ! -f "$ROOT/INSTALLATION.md" ]] || install -Dm644 "$ROOT/INSTALLATION.md" "$DEB_ROOT/usr/share/doc/termexplorer/INSTALLATION.md"
  install -Dm644 "$CHANGELOG_GZ" "$DEB_ROOT/usr/share/doc/termexplorer/changelog.Debian.gz"

  # DEBIAN Control File
  mkdir -p "$DEB_ROOT/DEBIAN"
  cat > "$DEB_ROOT/DEBIAN/control" <<CONTROL
Package: termexplorer
Version: $DEB_VERSION
Section: utils
Priority: optional
Architecture: $DEB_ARCH
Depends: libc6 (>= 2.35), libgcc-s1
Suggests: git, ffmpeg, poppler-utils, trash-cli
Maintainer: TermExplorer Contributors <101039137+jyothiprajith1323@users.noreply.github.com>
Description: Rust terminal developer workstation
 Inspection-first terminal developer workstation combining file exploration,
 contextual previews, editing and developer workflows.
Homepage: https://github.com/jyothiprajith1323/TermExplorer
CONTROL
  chmod 0755 "$DEB_ROOT/DEBIAN"
  chmod 0644 "$DEB_ROOT/DEBIAN/control"

  # Generate md5sums for packaging integrity
  (
    cd "$DEB_ROOT"
    find usr -type f -exec md5sum {} + > DEBIAN/md5sums
    chmod 0644 DEBIAN/md5sums
  )

  dpkg-deb --root-owner-group --build "$DEB_ROOT" "$OUTPUT/termexplorer_${DEB_VERSION}_${DEB_ARCH}.deb" >/dev/null
  echo "Debian package created: $OUTPUT/termexplorer_${DEB_VERSION}_${DEB_ARCH}.deb"
else
  echo "dpkg-deb not found; skipping Debian package creation."
fi

# -------------------------------------------------------------
# 2. RPM Package (.rpm)
# -------------------------------------------------------------
if command -v rpmbuild >/dev/null 2>&1; then
  echo "Building RPM package (.rpm)..."
  RPM_CHANGELOG_DATE=$(LC_ALL=C date "+%a %b %d %Y")
  RPM_TOP="$WORK/rpmbuild"
  mkdir -p "$RPM_TOP/BUILD" "$RPM_TOP/BUILDROOT" "$RPM_TOP/RPMS" "$RPM_TOP/SOURCES" "$RPM_TOP/SPECS" "$RPM_TOP/SRPMS"

  cp "$BINARY" "$RPM_TOP/SOURCES/termexplorer"
  cp "$ROOT/LICENSE" "$RPM_TOP/SOURCES/LICENSE"
  [[ ! -f "$ROOT/README.md" ]] || cp "$ROOT/README.md" "$RPM_TOP/SOURCES/README.md"
  [[ ! -f "$ROOT/INSTALLATION.md" ]] || cp "$ROOT/INSTALLATION.md" "$RPM_TOP/SOURCES/INSTALLATION.md"
  if [[ -f "$MANPAGE_GZ" ]]; then cp "$MANPAGE_GZ" "$RPM_TOP/SOURCES/termexplorer.1.gz"; fi
  if [[ -f "$ROOT/packaging/termexplorer.desktop" ]]; then cp "$ROOT/packaging/termexplorer.desktop" "$RPM_TOP/SOURCES/termexplorer.desktop"; fi
  if [[ -f "$ROOT/packaging/termexplorer.svg" ]]; then cp "$ROOT/packaging/termexplorer.svg" "$RPM_TOP/SOURCES/termexplorer.svg"; fi
  if [[ -f "$ROOT/packaging/completions/termexplorer.bash" ]]; then cp "$ROOT/packaging/completions/termexplorer.bash" "$RPM_TOP/SOURCES/termexplorer.bash"; fi
  if [[ -f "$ROOT/packaging/completions/_termexplorer" ]]; then cp "$ROOT/packaging/completions/_termexplorer" "$RPM_TOP/SOURCES/_termexplorer"; fi
  if [[ -f "$ROOT/packaging/completions/termexplorer.fish" ]]; then cp "$ROOT/packaging/completions/termexplorer.fish" "$RPM_TOP/SOURCES/termexplorer.fish"; fi

  cat > "$RPM_TOP/SPECS/termexplorer.spec" <<SPEC
Name:           termexplorer
Version:        $RPM_VERSION
Release:        1
Summary:        Rust terminal developer workstation
License:        MIT
URL:            https://github.com/jyothiprajith1323/TermExplorer
BuildArch:      $RPM_ARCH
Suggests:       git, ffmpeg, poppler-utils, trash-cli

%description
Inspection-first terminal developer workstation combining file exploration,
contextual previews, editing and developer workflows.

%prep

%build

%install
rm -rf %{buildroot}
install -Dpm0755 %{_sourcedir}/termexplorer %{buildroot}%{_bindir}/termexplorer
install -Dpm0644 %{_sourcedir}/LICENSE %{buildroot}%{_licensedir}/%{name}/LICENSE
if [ -f %{_sourcedir}/README.md ]; then install -Dpm0644 %{_sourcedir}/README.md %{buildroot}%{_docdir}/%{name}/README.md; fi
if [ -f %{_sourcedir}/INSTALLATION.md ]; then install -Dpm0644 %{_sourcedir}/INSTALLATION.md %{buildroot}%{_docdir}/%{name}/INSTALLATION.md; fi
if [ -f %{_sourcedir}/termexplorer.1.gz ]; then install -Dpm0644 %{_sourcedir}/termexplorer.1.gz %{buildroot}%{_mandir}/man1/termexplorer.1.gz; fi
if [ -f %{_sourcedir}/termexplorer.desktop ]; then install -Dpm0644 %{_sourcedir}/termexplorer.desktop %{buildroot}%{_datadir}/applications/termexplorer.desktop; fi
if [ -f %{_sourcedir}/termexplorer.svg ]; then install -Dpm0644 %{_sourcedir}/termexplorer.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/termexplorer.svg; fi
if [ -f %{_sourcedir}/termexplorer.bash ]; then install -Dpm0644 %{_sourcedir}/termexplorer.bash %{buildroot}%{_datadir}/bash-completion/completions/termexplorer; fi
if [ -f %{_sourcedir}/_termexplorer ]; then install -Dpm0644 %{_sourcedir}/_termexplorer %{buildroot}%{_datadir}/zsh/site-functions/_termexplorer; fi
if [ -f %{_sourcedir}/termexplorer.fish ]; then install -Dpm0644 %{_sourcedir}/termexplorer.fish %{buildroot}%{_datadir}/fish/vendor_completions.d/termexplorer.fish; fi

%files
%{_bindir}/termexplorer
%license %{_licensedir}/%{name}/LICENSE
%doc %{_docdir}/%{name}/
%{_mandir}/man1/termexplorer.1*
%{_datadir}/applications/termexplorer.desktop
%{_datadir}/icons/hicolor/scalable/apps/termexplorer.svg
%{_datadir}/bash-completion/completions/termexplorer
%{_datadir}/zsh/site-functions/_termexplorer
%{_datadir}/fish/vendor_completions.d/termexplorer.fish

%changelog
* $RPM_CHANGELOG_DATE TermExplorer Contributors - $RPM_VERSION-1
- Build release package with full desktop and shell assets.
SPEC
  rpmbuild --define "_topdir $RPM_TOP" --define "_rpmdir $RPM_TOP/RPMS" -bb "$RPM_TOP/SPECS/termexplorer.spec" >/dev/null
  RPM_FILE=$(find "$RPM_TOP/RPMS" -type f -name '*.rpm' -print -quit)
  [[ -n "$RPM_FILE" ]] || { echo "RPM build did not produce an artifact." >&2; exit 1; }
  cp "$RPM_FILE" "$OUTPUT/"
  echo "RPM package created: $OUTPUT/$(basename "$RPM_FILE")"
else
  echo "rpmbuild not found; skipping RPM package creation."
fi

# -------------------------------------------------------------
# 3. Portable Linux Tarball (.tar.gz)
# -------------------------------------------------------------
echo "Building Portable Linux Tarball..."
TAR_NAME="termexplorer-v${VERSION}-linux-${TAR_ARCH}"
TAR_STAGE="$WORK/tar-root/$TAR_NAME"
mkdir -p "$TAR_STAGE/bin"
mkdir -p "$TAR_STAGE/share/man/man1"
mkdir -p "$TAR_STAGE/share/applications"
mkdir -p "$TAR_STAGE/share/icons/hicolor/scalable/apps"
mkdir -p "$TAR_STAGE/share/bash-completion/completions"
mkdir -p "$TAR_STAGE/share/zsh/site-functions"
mkdir -p "$TAR_STAGE/share/fish/vendor_completions.d"
mkdir -p "$TAR_STAGE/share/doc/termexplorer"

# Binary
install -Dm755 "$BINARY" "$TAR_STAGE/bin/termexplorer"

# Assets
if [[ -f "$ROOT/packaging/termexplorer.1" ]]; then
  install -Dm644 "$ROOT/packaging/termexplorer.1" "$TAR_STAGE/share/man/man1/termexplorer.1"
fi
if [[ -f "$MANPAGE_GZ" ]]; then
  install -Dm644 "$MANPAGE_GZ" "$TAR_STAGE/share/man/man1/termexplorer.1.gz"
fi
if [[ -f "$ROOT/packaging/termexplorer.desktop" ]]; then
  install -Dm644 "$ROOT/packaging/termexplorer.desktop" "$TAR_STAGE/share/applications/termexplorer.desktop"
fi
if [[ -f "$ROOT/packaging/termexplorer.svg" ]]; then
  install -Dm644 "$ROOT/packaging/termexplorer.svg" "$TAR_STAGE/share/icons/hicolor/scalable/apps/termexplorer.svg"
fi
if [[ -f "$ROOT/packaging/completions/termexplorer.bash" ]]; then
  install -Dm644 "$ROOT/packaging/completions/termexplorer.bash" "$TAR_STAGE/share/bash-completion/completions/termexplorer"
fi
if [[ -f "$ROOT/packaging/completions/_termexplorer" ]]; then
  install -Dm644 "$ROOT/packaging/completions/_termexplorer" "$TAR_STAGE/share/zsh/site-functions/_termexplorer"
fi
if [[ -f "$ROOT/packaging/completions/termexplorer.fish" ]]; then
  install -Dm644 "$ROOT/packaging/completions/termexplorer.fish" "$TAR_STAGE/share/fish/vendor_completions.d/termexplorer.fish"
fi

# Docs and licenses
install -Dm644 "$ROOT/LICENSE" "$TAR_STAGE/share/doc/termexplorer/LICENSE"
[[ ! -f "$ROOT/README.md" ]] || install -Dm644 "$ROOT/README.md" "$TAR_STAGE/share/doc/termexplorer/README.md"
[[ ! -f "$ROOT/INSTALLATION.md" ]] || install -Dm644 "$ROOT/INSTALLATION.md" "$TAR_STAGE/share/doc/termexplorer/INSTALLATION.md"

# Scripts
if [[ -f "$ROOT/packaging/install.sh" ]]; then
  install -Dm755 "$ROOT/packaging/install.sh" "$TAR_STAGE/install.sh"
fi
if [[ -f "$ROOT/packaging/uninstall.sh" ]]; then
  install -Dm755 "$ROOT/packaging/uninstall.sh" "$TAR_STAGE/uninstall.sh"
fi

# Assemble reproducible tarball
SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-$(git log -1 --pretty=%ct 2>/dev/null || stat -c %Y "$ROOT/Cargo.toml")}"
tar -C "$WORK/tar-root" \
  --owner=0 --group=0 --numeric-owner \
  --mtime="@${SOURCE_DATE_EPOCH}" \
  --sort=name \
  -czf "$OUTPUT/${TAR_NAME}.tar.gz" \
  "$TAR_NAME"
echo "Portable tarball created: $OUTPUT/${TAR_NAME}.tar.gz"

# -------------------------------------------------------------
# 4. Checksums & Manifest
# -------------------------------------------------------------
(
  cd "$OUTPUT"
  sha256sum -- *.deb *.rpm *.tar.gz 2>/dev/null > PACKAGES_SHA256SUMS || true
  cp -f PACKAGES_SHA256SUMS SHA256SUMS
)

{
  printf 'TermExplorer Version: %s\n' "$VERSION"
  printf 'Target Architecture: %s\n' "$TAR_ARCH"
  printf 'Build Date: %s\n' "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  printf 'Package Artifacts:\n'
  find "$OUTPUT" -maxdepth 1 -type f \( -name '*.deb' -o -name '*.rpm' -o -name '*.tar.gz' \) -printf '  %f (%s bytes)\n' | sort
} > "$OUTPUT/MANIFEST.txt"

echo "=========================================================="
echo " Packaging complete in $OUTPUT:"
ls -lh "$OUTPUT"
echo "=========================================================="
