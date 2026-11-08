# 📦 TermExplorer — Installation Guide

This guide covers all supported installation methods, system prerequisites, and optional media decoders for **TermExplorer**.

---

## 🖥️ System Requirements

- **Operating System:** Linux (Ubuntu 20.04+, Debian 11+, Fedora 38+, Arch Linux, openSUSE, Alpine).
- **Architecture:** x86_64, aarch64 / ARM64.
- **Terminal Emulator:** Any modern terminal emulator supporting 24-bit Truecolor and UTF-8 (e.g., Kitty, Alacritty, WezTerm, GNOME Terminal, Foot).
- **Fonts (Recommended):** [JetBrains Mono Nerd Font](https://www.nerdfonts.com/font-downloads) for file icons, symbols, and ligatures.

---

## 📥 Method 1: Pre-compiled GitHub Release Binary (Fastest)

Download the validated standalone binary from GitHub Releases:

```bash
# 1. Download the release archive and SHA-256 checksums
curl -LO https://github.com/jyothiprajith1323/TermExplorer/releases/download/v1.1.0-rc2/termexplorer-v1.1.0-rc2-x86_64-unknown-linux-gnu.tar.gz
curl -LO https://github.com/jyothiprajith1323/TermExplorer/releases/download/v1.1.0-rc2/SHA256SUMS

# 2. Verify archive integrity
sha256sum --ignore-missing -c SHA256SUMS

# 3. Extract and install into ~/.local/bin
tar -xzf termexplorer-v1.1.0-rc2-x86_64-unknown-linux-gnu.tar.gz
mkdir -p ~/.local/bin
install -m 755 termexplorer-v1.1.0-rc2-x86_64-unknown-linux-gnu/termexplorer ~/.local/bin/termexplorer
```

*Ensure `~/.local/bin` is in your `$PATH`:*
```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

---

## ⚡ Method 2: Automated Shell Installer

Run our POSIX-compliant automated installation script. It verifies your Rust toolchain, fetches the latest release, compiles with optimizations, and installs `termexplorer` into `~/.local/bin/`:

```bash
curl -fsSL https://raw.githubusercontent.com/jyothiprajith1323/TermExplorer/master/install.sh | bash
```

To install directly to `/usr/local/bin` system-wide:
```bash
curl -fsSL https://raw.githubusercontent.com/jyothiprajith1323/TermExplorer/master/install.sh | sudo bash
```

*Ensure `~/.local/bin` is in your `$PATH`:*
```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

---

## 📦 Method 2: Arch Linux (AUR / PKGBUILD)

For Arch Linux and derivatives (Manjaro, EndeavourOS):

### Using an AUR Helper
```bash
yay -S termexplorer-bin
# or
paru -S termexplorer-bin
```

### Manual PKGBUILD Compilation
```bash
git clone https://github.com/jyothiprajith1323/TermExplorer.git
cd TermExplorer/packaging/aur
makepkg -si
```

---

## 🦀 Method 3: Install via Cargo

If you have a working Rust toolchain (`cargo >= 1.80`):

```bash
cargo install --git https://github.com/jyothiprajith1323/TermExplorer.git --locked
```

This compiles and places the `termexplorer` binary in `~/.cargo/bin/`.

---

## 🛠️ Method 4: Build from Source

### 1. Install Build Dependencies

#### Ubuntu / Debian / Pop!_OS
```bash
sudo apt update
sudo apt install -y build-essential pkg-config libasound2-dev libxcb1-dev git clang lld
```

#### Fedora / RHEL
```bash
sudo dnf groupinstall "Development Tools"
sudo dnf install -y alsa-lib-devel libxcb-devel git clang lld
```

#### Arch Linux
```bash
sudo pacman -S --needed base-devel alsa-lib libxcb git clang lld
```

### 2. Clone and Compile

```bash
git clone https://github.com/jyothiprajith1323/TermExplorer.git
cd TermExplorer

# Build optimized release binary
cargo build --release

# Copy to local binary path
install -Dm755 target/release/termexplorer ~/.local/bin/termexplorer
```

---

## 🎬 Optional Multimedia Preview Utilities

TermExplorer features pure-Rust fallbacks for syntax highlighting, hex viewing, CSV/TSV, JSON, and Office XML documents. To unlock full rich multimedia previews (images, audio ID3 tags, video thumbnails, and PDF rendering):

```bash
# Ubuntu / Debian
sudo apt install -y ffmpeg poppler-utils gio trash-cli

# Fedora
sudo apt install -y ffmpeg poppler-utils trash-cli

# Arch Linux
sudo pacman -S ffmpeg poppler trash-cli
```

| Utility | Capability Unlocked |
|---|---|
| `ffmpeg` & `ffprobe` | Video metadata inspection & first-frame visual thumbnail extraction |
| `poppler-utils` (`pdftotext`, `pdfinfo`) | High-speed PDF page stream extraction & metadata indexing |
| `trash-cli` / `gio trash` | Safe FreeDesktop `.trashinfo` compliant trashing and recovery |

---

## 🔍 Verifying the Installation

```bash
# Check version
termexplorer --version

# Run built-in self-test & launch
termexplorer
```
