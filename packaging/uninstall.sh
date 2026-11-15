#!/usr/bin/env bash
# TermExplorer Uninstaller
# Safely removes files installed by TermExplorer installer while preserving user configuration.

set -euo pipefail

DRY_RUN=0
PURGE_USER_DATA=0
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
TermExplorer Uninstaller

Usage:
  ./uninstall.sh [OPTIONS]

Options:
  --prefix <PATH>       Prefix to remove from (default: ${DEFAULT_PREFIX})
  -n, --dry-run         Show what files would be removed without deleting them
  --purge-user-data     Remove user configuration (~/.config/termexplorer) and
                        cache (~/.cache/termexplorer) as well (prompts for confirmation)
  -h, --help            Show this help message and exit

Preservation Guarantee:
  By default, your configuration (~/.config/termexplorer), persistent data
  (~/.local/share/termexplorer), and logs are NEVER modified or deleted.

EOF
}

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
        -n|--dry-run)
            DRY_RUN=1
            shift
            ;;
        --purge-user-data)
            PURGE_USER_DATA=1
            shift
            ;;
        -h|--help)
            print_usage
            exit 0
            ;;
        *)
            echo "Error: Unknown argument: $1" >&2
            echo "Try './uninstall.sh --help' for options." >&2
            exit 1
            ;;
    esac
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# If script is located inside <PREFIX>/share/termexplorer/uninstall.sh, automatically infer prefix
if [ -z "${CUSTOM_PREFIX}" ] && [ -f "${SCRIPT_DIR}/install-manifest.txt" ]; then
    PREFIX="$(cd "${SCRIPT_DIR}/../.." && pwd)"
fi

MANIFEST="${PREFIX}/share/termexplorer/install-manifest.txt"

echo "=========================================================="
echo " TermExplorer Uninstaller"
echo "=========================================================="
echo " Prefix: ${PREFIX}"
echo " Mode:   $([ "$DRY_RUN" -eq 1 ] && echo "DRY-RUN (no files will be deleted)" || echo "UNINSTALL")"
echo "=========================================================="

remove_file() {
    local file="$1"
    if [ -f "$file" ] || [ -L "$file" ]; then
        if [ "$DRY_RUN" -eq 1 ]; then
            echo "[DRY-RUN] rm -f \"${file}\""
        else
            rm -f "${file}"
            echo "Removed: ${file}"
        fi
    fi
}

remove_dir_if_empty() {
    local dir="$1"
    if [ -d "$dir" ]; then
        if [ "$DRY_RUN" -eq 1 ]; then
            echo "[DRY-RUN] rmdir --ignore-fail-on-non-empty \"${dir}\""
        else
            rmdir --ignore-fail-on-non-empty "${dir}" 2>/dev/null || true
        fi
    fi
}

if [ -f "${MANIFEST}" ]; then
    echo "Found install manifest: ${MANIFEST}"
    echo "Removing registered files..."
    while IFS= read -r line || [ -n "$line" ]; do
        # Ignore comments and empty lines
        [[ "$line" =~ ^#.*$ ]] && continue
        [ -z "$line" ] && continue
        remove_file "$line"
    done < "${MANIFEST}"
else
    echo "No install manifest found. Removing known standard files from ${PREFIX}..."
    remove_file "${PREFIX}/bin/termexplorer"
    remove_file "${PREFIX}/share/man/man1/termexplorer.1"
    remove_file "${PREFIX}/share/man/man1/termexplorer.1.gz"
    remove_file "${PREFIX}/share/applications/termexplorer.desktop"
    remove_file "${PREFIX}/share/icons/hicolor/scalable/apps/termexplorer.svg"
    remove_file "${PREFIX}/share/bash-completion/completions/termexplorer"
    remove_file "${PREFIX}/share/zsh/site-functions/_termexplorer"
    remove_file "${PREFIX}/share/fish/vendor_completions.d/termexplorer.fish"
    remove_file "${PREFIX}/share/doc/termexplorer/README.md"
    remove_file "${PREFIX}/share/doc/termexplorer/LICENSE"
    remove_file "${PREFIX}/share/termexplorer/uninstall.sh"
    remove_file "${PREFIX}/share/termexplorer/install-manifest.txt"
fi

# Clean up empty directories
remove_dir_if_empty "${PREFIX}/share/doc/termexplorer"
remove_dir_if_empty "${PREFIX}/share/termexplorer"

# Handle optional user data purging
if [ "$PURGE_USER_DATA" -eq 1 ]; then
    echo ""
    echo "WARNING: --purge-user-data requested."
    USER_CONFIG="${XDG_CONFIG_HOME:-${HOME}/.config}/termexplorer"
    USER_DATA="${XDG_DATA_HOME:-${HOME}/.local/share}/termexplorer"
    USER_CACHE="${XDG_CACHE_HOME:-${HOME}/.cache}/termexplorer"

    echo "This will delete the following user directories:"
    echo "  - ${USER_CONFIG}"
    echo "  - ${USER_DATA}"
    echo "  - ${USER_CACHE}"

    if [ "$DRY_RUN" -eq 1 ]; then
        echo "[DRY-RUN] rm -rf \"${USER_CONFIG}\" \"${USER_DATA}\" \"${USER_CACHE}\""
    else
        read -r -p "Are you sure you want to completely erase user config and data? [y/N] " response
        case "$response" in
            [yY][eE][sS]|[yY])
                rm -rf "${USER_CONFIG}" "${USER_DATA}" "${USER_CACHE}"
                echo "Purged user configuration and cache directories."
                ;;
            *)
                echo "User data purge cancelled. Preserved all configuration files."
                ;;
        esac
    fi
else
    echo ""
    echo "User configuration and persistent data were preserved."
    echo "Config location: ${XDG_CONFIG_HOME:-${HOME}/.config}/termexplorer"
fi

echo ""
echo "=========================================================="
echo " Uninstallation Complete."
echo "=========================================================="
