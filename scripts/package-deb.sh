#!/usr/bin/env bash
# TermExplorer Debian Package (.deb) Packaging Script (Compatibility Wrapper)
# Delegates to consolidated scripts/package_linux.sh.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

if [ ! -f "${REPO_ROOT}/target/release/termexplorer" ]; then
    echo "Release binary not found. Running build-release.sh first..."
    "${SCRIPT_DIR}/build-release.sh"
fi

VERSION="$(grep -m1 '^version = ' "${REPO_ROOT}/Cargo.toml" | cut -d '"' -f 2)"
"${SCRIPT_DIR}/package_linux.sh" "${REPO_ROOT}/target/release/termexplorer" "${VERSION}" "${REPO_ROOT}/dist"
