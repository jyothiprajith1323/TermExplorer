# TermExplorer Installation Guide

This guide covers installing **TermExplorer v1.1.0** across supported Linux distributions, WSL2 environments, and package formats.

---

## 1. Supported Platform Matrix

| Platform | Architecture | Tier | Notes |
|---|---|:---:|---|
| **Ubuntu / Debian** | `x86_64` (amd64) | Tier 1 | Native `.deb` package with dependencies |
| **Fedora / RHEL / AlmaLinux** | `x86_64` | Tier 1 | Portable standalone tarball |
| **Arch Linux / Manjaro** | `x86_64` | Tier 1 | Standalone tarball or source compilation |
| **Generic Linux (glibc 2.31+)** | `x86_64` | Tier 1 | Portable binary release |
| **WSL2 (Ubuntu on Windows)** | `x86_64` | Tier 1 | Full support with Windows Terminal / TrueColor |
| **Linux ARM64 / aarch64** | `aarch64` | Tier 2 | Build from source with Cargo |

---

## 2. Installation Methods

### Method A: Debian / Ubuntu Package (`.deb`)

For Ubuntu 20.04+, Debian 11+, Linux Mint, or Pop!_OS:

```bash
# 1. Download release .deb package
curl -LO https://github.com/jyothiprajith1323/TermExplorer/releases/download/v1.1.0/termexplorer_1.1.0_amd64.deb

# 2. Install package and dependencies via apt
sudo apt-get update
sudo apt-get install -y ./termexplorer_1.1.0_amd64.deb

# 3. Verify installation
termexplorer --version
termexplorer --health
```

The Debian package automatically places:
- Executable at `/usr/bin/termexplorer`
- Desktop entry at `/usr/share/applications/termexplorer.desktop`
- Application icon at `/usr/share/icons/hicolor/scalable/apps/termexplorer.svg`
- Manual page at `/usr/share/man/man1/termexplorer.1.gz`
- Shell completions in Bash, Zsh, and Fish completion directories

---

### Method B: Portable Standalone Tarball (`.tar.gz`)

For generic Linux distributions:

```bash
# 1. Download tarball and checksum file
curl -LO https://github.com/jyothiprajith1323/TermExplorer/releases/download/v1.1.0/termexplorer-v1.1.0-linux-x86_64.tar.gz
curl -LO https://github.com/jyothiprajith1323/TermExplorer/releases/download/v1.1.0/SHA256SUMS

# 2. Verify SHA-256 integrity
sha256sum --check --ignore-missing SHA256SUMS

# 3. Extract archive
tar -xzf termexplorer-v1.1.0-linux-x86_64.tar.gz
cd termexplorer-v1.1.0-linux-x86_64

# 4. Run automated user-space or system-wide installer
./install.sh           # Installs to ~/.local/bin and ~/.local/share
# Or for system-wide:
sudo ./install.sh --system  # Installs to /usr/local/bin
```

---

### Method C: Cargo (From Source)

To build from source using the Rust toolchain:

```bash
# Prerequisites: Rust 1.80+ and basic C compiler
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
sudo apt-get install -y build-essential pkg-config libgit2-dev

# Clone and compile
git clone https://github.com/jyothiprajith1323/TermExplorer.git
cd TermExplorer
cargo build --release --locked

# Install binary into user path
install -Dm755 target/release/termexplorer ~/.local/bin/termexplorer
```

---

## 3. Recommended Optional Dependencies

TermExplorer provides pure-Rust fallbacks for all operations. For enhanced multimedia previews, the following system utilities are detected automatically if present in `$PATH`:

| Utility | Distribution Package (Debian/Ubuntu) | Enhancement |
|---|---|---|
| `git` | `git` | Native repository branch status, diff viewing, and commits |
| `ffmpeg` | `ffmpeg` | Video frame extraction and audio waveform thumbnails |
| `pdftotext` | `poppler-utils` | High-fidelity text stream extraction from PDF documents |
| `sqlite3` | `sqlite3` | Database schema querying and sample row inspection |
| `trash-cli` | `trash-cli` | FreeDesktop.org Trash compliance (`gio trash` also supported) |
| `rust-analyzer` | `rust-analyzer` | Language Server Protocol diagnostics and auto-completion |

Install all optional helpers in a single command:
```bash
sudo apt-get install -y git ffmpeg poppler-utils sqlite3 trash-cli
```

---

## 4. Verification and Health Diagnostics

Run TermExplorer's built-in hardware diagnostics:

```bash
termexplorer --health
```

Sample output:
```text
TermExplorer System Diagnostics & Health Check
================================================
Version:           1.1.0
Terminal Size:     120 cols × 36 rows
Terminal Support:  $TERM=xterm-256color, $COLORTERM=truecolor, TrueColor=true
Default Shell:     /bin/zsh
PTY Available:     /dev/ptmx accessible
```

---

## 5. Uninstallation

### Debian Package:
```bash
sudo apt-get remove termexplorer
```

### Tarball / Local Install:
```bash
~/.local/share/termexplorer/uninstall.sh
# Or if installed system-wide:
sudo /usr/local/share/termexplorer/uninstall.sh
```
User configuration in `~/.config/termexplorer` is preserved unless manually deleted.
