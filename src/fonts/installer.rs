//! Font installation script generator for Ubuntu / Linux user environment.

use std::path::PathBuf;

pub fn user_fonts_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("fonts")
    } else {
        PathBuf::from(".local/share/fonts")
    }
}

pub fn install_script_content() -> &'static str {
    r#"#!/usr/bin/env bash
# TermExplorer JetBrains Mono Nerd Font User-Space Installer
set -euo pipefail

FONT_DIR="${HOME}/.local/share/fonts/NerdFonts"
mkdir -p "${FONT_DIR}"

echo "==> Downloading JetBrains Mono Nerd Font..."
TMP_DIR=$(mktemp -d)
cd "${TMP_DIR}"

curl -fLo "JetBrainsMono.zip" \
  "https://github.com/ryanoasis/nerd-fonts/releases/latest/download/JetBrainsMono.tar.xz" \
  || curl -fLo "JetBrainsMono.zip" \
  "https://github.com/ryanoasis/nerd-fonts/releases/download/v3.2.1/JetBrainsMono.zip"

if command -v unzip >/dev/null 2>&1; then
    unzip -q -o "JetBrainsMono.zip" -d "${FONT_DIR}" 2>/dev/null || tar -xf "JetBrainsMono.zip" -C "${FONT_DIR}"
else
    tar -xf "JetBrainsMono.zip" -C "${FONT_DIR}"
fi

echo "==> Refreshing font cache..."
fc-cache -f "${FONT_DIR}"

rm -rf "${TMP_DIR}"
echo "==> JetBrains Mono Nerd Font installed successfully in ${FONT_DIR}!"
echo "==> You can now select 'JetBrainsMono Nerd Font Mono' in your terminal emulator preferences."
"#
}
