#!/usr/bin/env bash
# TermExplorer v1.1.0 Installation Script
# https://github.com/jyothiprajith1323/TermExplorer
#
# Usage:
#   curl -sSL https://raw.githubusercontent.com/jyothiprajith1323/TermExplorer/master/install.sh | bash

set -euo pipefail

# ANSI Color Codes
BOLD='\033[1m'
BLUE='\033[38;2;137;180;250m'
GREEN='\033[38;2;166;227;161m'
YELLOW='\033[38;2;249;226;175m'
RED='\033[38;2;243;139;168m'
RESET='\033[0m'

echo -e "${BLUE}${BOLD}"
echo "  ⚡ TermExplorer — The Developer Workstation"
echo "  Installing TermExplorer v1.1.0-RC1..."
echo -e "${RESET}"

# 1. Platform Verification
OS="$(uname -s)"
ARCH="$(uname -m)"

if [ "$OS" != "Linux" ]; then
    echo -e "${RED}Error: TermExplorer currently targets Linux systems. Found OS: ${OS}${RESET}"
    exit 1
fi

if [ "$ARCH" != "x86_64" ] && [ "$ARCH" != "aarch64" ]; then
    echo -e "${YELLOW}Warning: Architecture ${ARCH} is not in primary test matrix (x86_64 / aarch64).${RESET}"
fi

# 2. Determine Installation Target Directory
INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "$INSTALL_DIR"

# 3. Check for Rust / Cargo Toolchain
if command -v cargo >/dev/null 2>&1; then
    echo -e "${BLUE}==>${RESET} Building and installing TermExplorer via Cargo..."
    cargo install --git https://github.com/jyothiprajith1323/TermExplorer.git --bin termexplorer --root "${HOME}/.local"
else
    echo -e "${YELLOW}Notice: Cargo was not found on your system.${RESET}"
    echo "To compile TermExplorer, install the official Rust toolchain via: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo "Or clone the repository manually: git clone https://github.com/jyothiprajith1323/TermExplorer.git"
    exit 1
fi

# 4. PATH Check
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${YELLOW}==> Note:${RESET} ${INSTALL_DIR} is not in your current PATH."
    echo "Add the following line to your shell profile (~/.bashrc or ~/.zshrc):"
    echo -e "  ${BOLD}export PATH=\"\$HOME/.local/bin:\$PATH\"${RESET}"
fi

# 5. Success
echo -e "${GREEN}${BOLD}✓ TermExplorer v1.1.0 installed successfully!${RESET}"
echo -e "Run ${BOLD}termexplorer${RESET} in any terminal to launch the workstation."
