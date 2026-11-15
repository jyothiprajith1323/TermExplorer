#!/usr/bin/env bash
# TermExplorer Installer
# Portable installation script for user-level (~/.local) or system-level (/usr/local) environments.

set -euo pipefail

VERSION="1.1.0"
DRY_RUN=0
CUSTOM_PREFIX=""

# Detect default prefix based on EUID
if [ "$(id -u)" -eq 0 ]; then
    DEFAULT_PREFIX="/usr/local"
else
    DEFAULT_PREFIX="${HOME}/.local"
fi

PREFIX="${DEFAULT_PREFIX}"

print_usage() {
    cat <<EOF
TermExplorer v${VERSION} Installer

Usage:
  ./install.sh [OPTIONS]

Options:
  --prefix <PATH>    Set custom installation prefix (default: ${DEFAULT_PREFIX})
  --user             Install for current user only (${HOME}/.local)
  --system           Install system-wide (/usr/local, requires root/sudo)
  -n, --dry-run      Print planned file operations without making changes
  -h, --help         Show this help message and exit

File Destinations:
  Binary:            <PREFIX>/bin/termexplorer
  Manual page:       <PREFIX>/share/man/man1/termexplorer.1
  Desktop entry:     <PREFIX>/share/applications/termexplorer.desktop
  Scalable icon:     <PREFIX>/share/icons/hicolor/scalable/apps/termexplorer.svg
  Bash completion:   <PREFIX>/share/bash-completion/completions/termexplorer
  Zsh completion:    <PREFIX>/share/zsh/site-functions/_termexplorer
  Fish completion:   <PREFIX>/share/fish/vendor_completions.d/termexplorer.fish
  Documentation:     <PREFIX>/share/doc/termexplorer/
  Install manifest:  <PREFIX>/share/termexplorer/install-manifest.txt

EOF
}

# Parse CLI arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --prefix)
            if [ -n "${2:-}" ]; then
                PREFIX="$2"
                CUSTOM_PREFIX="1"
                shift 2
            else
                echo "Error: --prefix requires a directory path." >&2
                exit 1
            fi
            ;;
        --user)
            PREFIX="${HOME}/.local"
            CUSTOM_PREFIX="1"
            shift
            ;;
        --system)
            PREFIX="/usr/local"
            CUSTOM_PREFIX="1"
            shift
            ;;
        -n|--dry-run)
            DRY_RUN=1
            shift
            ;;
        -h|--help)
            print_usage
            exit 0
            ;;
        *)
            echo "Error: Unknown argument: $1" >&2
            echo "Try './install.sh --help' for options." >&2
            exit 1
            ;;
    esac
done

# Resolve source root directory (either tarball root or repository root)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -f "${SCRIPT_DIR}/bin/termexplorer" ]; then
    # Running from extracted portable tarball
    SRC_BIN="${SCRIPT_DIR}/bin/termexplorer"
    SRC_MAN="${SCRIPT_DIR}/share/man/man1/termexplorer.1"
    SRC_DESKTOP="${SCRIPT_DIR}/share/applications/termexplorer.desktop"
    SRC_ICON="${SCRIPT_DIR}/share/icons/hicolor/scalable/apps/termexplorer.svg"
    SRC_BASH="${SCRIPT_DIR}/share/bash-completion/completions/termexplorer"
    SRC_ZSH="${SCRIPT_DIR}/share/zsh/site-functions/_termexplorer"
    SRC_FISH="${SCRIPT_DIR}/share/fish/vendor_completions.d/termexplorer.fish"
    SRC_README="${SCRIPT_DIR}/share/doc/termexplorer/README.md"
    SRC_LICENSE="${SCRIPT_DIR}/share/doc/termexplorer/LICENSE"
    SRC_UNINSTALL="${SCRIPT_DIR}/uninstall.sh"
elif [ -f "${SCRIPT_DIR}/../target/release/termexplorer" ]; then
    # Running from repository packaging/ directory after cargo build --release
    REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
    SRC_BIN="${REPO_ROOT}/target/release/termexplorer"
    SRC_MAN="${SCRIPT_DIR}/termexplorer.1"
    SRC_DESKTOP="${SCRIPT_DIR}/termexplorer.desktop"
    SRC_ICON="${SCRIPT_DIR}/termexplorer.svg"
    SRC_BASH="${SCRIPT_DIR}/completions/termexplorer.bash"
    SRC_ZSH="${SCRIPT_DIR}/completions/_termexplorer"
    SRC_FISH="${SCRIPT_DIR}/completions/termexplorer.fish"
    SRC_README="${REPO_ROOT}/README.md"
    SRC_LICENSE="${REPO_ROOT}/LICENSE"
    SRC_UNINSTALL="${SCRIPT_DIR}/uninstall.sh"
else
    echo "Error: Could not locate compiled 'termexplorer' binary." >&2
    echo "Please build the project first ('cargo build --release') or run install.sh from an official release archive." >&2
    exit 1
fi

if [ ! -f "${SRC_BIN}" ]; then
    echo "Error: Executable not found at ${SRC_BIN}" >&2
    exit 1
fi

echo "=========================================================="
echo " TermExplorer v${VERSION} Installer"
echo "=========================================================="
echo " Target Prefix: ${PREFIX}"
echo " Action:        $([ "$DRY_RUN" -eq 1 ] && echo "DRY-RUN (no changes made)" || echo "INSTALL")"
echo " Source:        ${SCRIPT_DIR}"
echo "=========================================================="

# Check permissions for target prefix
if [ "$DRY_RUN" -eq 0 ]; then
    if ! mkdir -p "${PREFIX}/bin" 2>/dev/null; then
        echo "Error: Cannot create directories in ${PREFIX}. Permission denied." >&2
        echo "If installing to a system directory (like /usr/local), please run with sudo:" >&2
        echo "  sudo ./install.sh --system" >&2
        echo "Or install to your user home directory instead:" >&2
        echo "  ./install.sh --user" >&2
        exit 1
    fi
fi

# Define destination paths
DEST_BIN="${PREFIX}/bin/termexplorer"
DEST_MAN="${PREFIX}/share/man/man1/termexplorer.1"
DEST_DESKTOP="${PREFIX}/share/applications/termexplorer.desktop"
DEST_ICON="${PREFIX}/share/icons/hicolor/scalable/apps/termexplorer.svg"
DEST_BASH="${PREFIX}/share/bash-completion/completions/termexplorer"
DEST_ZSH="${PREFIX}/share/zsh/site-functions/_termexplorer"
DEST_FISH="${PREFIX}/share/fish/vendor_completions.d/termexplorer.fish"
DEST_DOC_DIR="${PREFIX}/share/doc/termexplorer"
DEST_DATA_DIR="${PREFIX}/share/termexplorer"
DEST_MANIFEST="${DEST_DATA_DIR}/install-manifest.txt"
DEST_UNINSTALL="${DEST_DATA_DIR}/uninstall.sh"

copy_file() {
    local src="$1"
    local dst="$2"
    local mode="$3"

    if [ ! -f "$src" ]; then
        echo "Warning: Source file $src does not exist, skipping."
        return 0
    fi

    local parent_dir
    parent_dir="$(dirname "$dst")"

    if [ "$DRY_RUN" -eq 1 ]; then
        echo "[DRY-RUN] install -m ${mode} \"${src}\" -> \"${dst}\""
    else
        mkdir -p "${parent_dir}"
        install -m "${mode}" "${src}" "${dst}"
        echo "Installed: ${dst}"
    fi
}

echo ""
echo "Installing application artifacts..."

copy_file "${SRC_BIN}" "${DEST_BIN}" "0755"
copy_file "${SRC_MAN}" "${DEST_MAN}" "0644"
copy_file "${SRC_DESKTOP}" "${DEST_DESKTOP}" "0644"
copy_file "${SRC_ICON}" "${DEST_ICON}" "0644"
copy_file "${SRC_BASH}" "${DEST_BASH}" "0644"
copy_file "${SRC_ZSH}" "${DEST_ZSH}" "0644"
copy_file "${SRC_FISH}" "${DEST_FISH}" "0644"

if [ -f "${SRC_README}" ]; then
    copy_file "${SRC_README}" "${DEST_DOC_DIR}/README.md" "0644"
fi
if [ -f "${SRC_LICENSE}" ]; then
    copy_file "${SRC_LICENSE}" "${DEST_DOC_DIR}/LICENSE" "0644"
fi
if [ -f "${SRC_UNINSTALL}" ]; then
    copy_file "${SRC_UNINSTALL}" "${DEST_UNINSTALL}" "0755"
fi

# Write manifest for reliable uninstallation
if [ "$DRY_RUN" -eq 0 ]; then
    mkdir -p "${DEST_DATA_DIR}"
    cat > "${DEST_MANIFEST}" <<EOF
# TermExplorer v${VERSION} Installation Manifest
# Generated at: $(date -u +"%Y-%m-%dT%H:%M:%SZ")
# Prefix: ${PREFIX}
${DEST_BIN}
${DEST_MAN}
${DEST_DESKTOP}
${DEST_ICON}
${DEST_BASH}
${DEST_ZSH}
${DEST_FISH}
${DEST_DOC_DIR}/README.md
${DEST_DOC_DIR}/LICENSE
${DEST_UNINSTALL}
${DEST_MANIFEST}
EOF
    echo "Recorded installation manifest at: ${DEST_MANIFEST}"
fi

echo ""
echo "=========================================================="
echo " Installation Complete!"
echo "=========================================================="
echo "Executable: ${DEST_BIN}"

# Check PATH integration
if [[ ":$PATH:" != *":${PREFIX}/bin:"* ]]; then
    echo ""
    echo "Notice: '${PREFIX}/bin' is not in your current PATH."
    echo "To run 'termexplorer' from anywhere, add it to your shell configuration:"
    echo ""
    if [ -n "${BASH_VERSION:-}" ]; then
        echo "  echo 'export PATH=\"${PREFIX}/bin:\$PATH\"' >> ~/.bashrc"
        echo "  source ~/.bashrc"
    elif [ -n "${ZSH_VERSION:-}" ]; then
        echo "  echo 'export PATH=\"${PREFIX}/bin:\$PATH\"' >> ~/.zshrc"
        echo "  source ~/.zshrc"
    else
        echo "  export PATH=\"${PREFIX}/bin:\$PATH\""
    fi
    echo ""
fi

echo "To verify the installation, run:"
echo "  ${DEST_BIN} --version"
echo "  ${DEST_BIN} --health"
echo ""
echo "To uninstall TermExplorer in the future, run:"
echo "  ${DEST_UNINSTALL}"
echo "=========================================================="
