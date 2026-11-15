#!/usr/bin/env bash
# Generate a static YUM/DNF RPM repository structure for TermExplorer RPM packages.
# Usage: scripts/generate_rpm_repo.sh <rpm-file-or-dir> <repo-output-dir>

set -euo pipefail

if [[ $# -lt 2 ]]; then
  echo "Usage: $0 <rpm-file-or-dir> <repo-output-dir>" >&2
  exit 2
fi

INPUT_PATH=$(realpath "$1")
REPO_DIR=$(realpath -m "$2")
ARCH=$(uname -m)

mkdir -p "$REPO_DIR/$ARCH"

# Collect RPM packages
if [[ -d "$INPUT_PATH" ]]; then
  RPM_FILES=( $(find "$INPUT_PATH" -maxdepth 1 -name '*.rpm') )
elif [[ -f "$INPUT_PATH" ]]; then
  RPM_FILES=("$INPUT_PATH")
else
  echo "Error: $INPUT_PATH does not exist." >&2
  exit 1
fi

if [[ ${#RPM_FILES[@]} -eq 0 ]]; then
  echo "Notice: No .rpm packages found in $INPUT_PATH" >&2
  exit 0
fi

for rpm_pkg in "${RPM_FILES[@]}"; do
  cp -u "$rpm_pkg" "$REPO_DIR/$ARCH/" || cp -f "$rpm_pkg" "$REPO_DIR/$ARCH/"
done

echo "Generating RPM repository metadata for $ARCH..."
if command -v createrepo_c >/dev/null 2>&1; then
  createrepo_c --update "$REPO_DIR/$ARCH"
elif command -v createrepo >/dev/null 2>&1; then
  createrepo --update "$REPO_DIR/$ARCH"
else
  echo "Notice: 'createrepo_c' or 'createrepo' not installed on this system."
  echo "RPM packages staged in $REPO_DIR/$ARCH/."
  echo "To generate repodata XML metadata, run 'createrepo_c $REPO_DIR/$ARCH'."
fi

# GPG Signing of repodata if key is available
if [[ -n "${GPG_KEY_ID:-}" ]] && command -v gpg >/dev/null 2>&1 && [[ -f "$REPO_DIR/$ARCH/repodata/repomd.xml" ]]; then
  echo "Signing repomd.xml with GPG key: $GPG_KEY_ID"
  gpg --batch --yes --armor --detach-sign -u "$GPG_KEY_ID" -o "$REPO_DIR/$ARCH/repodata/repomd.xml.asc" "$REPO_DIR/$ARCH/repodata/repomd.xml"
fi

# Provide sample .repo configuration for local testing / static staging
cat > "$REPO_DIR/termexplorer.repo" <<EOF
[termexplorer]
name=TermExplorer Development / Static Repository (Unsigned)
baseurl=https://<host>/rpm/$ARCH/
enabled=1
gpgcheck=0
# Note: Unsigned repository feed for local/staging use. Production deployment requires GPG key signing.
EOF

echo "RPM repository layout ready at $REPO_DIR"
