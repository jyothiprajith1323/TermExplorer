# Development Environment Setup Guide

This guide details how to configure a workstation for developing, building, testing, and debugging **TermExplorer**.

---

## 1. Prerequisites & Toolchain

### Rust Toolchain
TermExplorer targets modern stable Rust (1.80+):
```bash
# Install rustup and stable toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install rustfmt and clippy
rustup component add rustfmt clippy
```

### System Build Dependencies (Debian / Ubuntu)
```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    git \
    libasound2-dev \
    libudev-dev
```

### Optional Preview & Test Utilities
```bash
sudo apt-get install -y \
    ffmpeg \
    poppler-utils \
    sqlite3 \
    trash-cli
```

---

## 2. Cloning and First Build

```bash
# Clone the repository
git clone https://github.com/jyothiprajith1323/TermExplorer.git
cd TermExplorer

# Build in debug mode
cargo build

# Run unit and integration tests
cargo test --all-targets

# Run the local build
cargo run -- .
```

---

## 3. WSL2 (Windows Subsystem for Linux) Setup

If developing inside WSL2 on Windows 10/11:
1. Ensure your WSL distribution is running WSL 2: `wsl -l -v`.
2. Use **Windows Terminal** as your host terminal emulator to ensure full 24-bit TrueColor and UTF-8 support.
3. Install the recommended font: [JetBrainsMono Nerd Font](https://www.nerdfonts.com/font-downloads) in Windows, and set it as the font face in Windows Terminal settings (`"font": { "face": "JetBrainsMono Nerd Font" }`).
4. Develop inside the Linux filesystem (e.g. `~/Projects/TermExplorer`), not under `/mnt/c/`, for optimal I/O performance.
