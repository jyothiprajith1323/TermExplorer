# 📦 TermExplorer — Comprehensive Installation Guide

This guide covers all supported installation methods, system prerequisites, package verification, and configuration paths for **TermExplorer**.

---

## 🖥️ System Requirements & Supported Platforms

- **Operating System:** Linux (Ubuntu 22.04+, Debian 12+, Fedora 38+, Arch Linux, openSUSE, Alpine) or WSL2 (Windows Subsystem for Linux 2).
- **Architecture:** `x86_64` (Primary), `aarch64` / `arm64` (Secondary).
- **C Library Baseline:** GNU C Library (`glibc >= 2.34`).
- **Terminal Emulator:** Any modern terminal emulator supporting 24-bit TrueColor and UTF-8 (e.g., Alacritty, Kitty, WezTerm, Foot, Windows Terminal).
- **Recommended Font:** [JetBrains Mono Nerd Font](https://www.nerdfonts.com/font-downloads) (or any Nerd Font) for file icons and Git branch markers.

---

## 📥 Method 1: Portable Linux Archive (`.tar.gz`) — Recommended for Non-Root Users

The portable archive includes the pre-compiled binary, manual page, desktop launcher, scalable icon, shell completions, and installer script.

### 1. Download & Verify Checksums
```bash
# Download the release archive and checksums
curl -LO https://github.com/jyothiprajith1323/TermExplorer/releases/download/v1.1.0/termexplorer-v1.1.0-linux-x86_64.tar.gz
curl -LO https://github.com/jyothiprajith1323/TermExplorer/releases/download/v1.1.0/SHA256SUMS

# Verify cryptographic hash
sha256sum --ignore-missing -c SHA256SUMS
```

### 2. Extract Archive
```bash
tar -xzf termexplorer-v1.1.0-linux-x86_64.tar.gz
cd termexplorer-v1.1.0-linux-x86_64
```

### 3. Run Installer
- **For Current User Only (`~/.local` — No Root Required):**
  ```bash
  ./install.sh --user
  ```
- **System-Wide (`/usr/local` — Requires Sudo):**
  ```bash
  sudo ./install.sh --system
  ```
- **Custom Prefix:**
  ```bash
  ./install.sh --prefix /opt/termexplorer
  ```

### 4. PATH Configuration (User-Level Install)
If `~/.local/bin` is not yet in your `$PATH`, append it:
```bash
# For Bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# For Zsh
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

---

## 📦 Method 2: Debian / Ubuntu Package (`.deb`)

For Debian, Ubuntu, Linux Mint, Pop!_OS, and other Debian-based distributions:

### 1. Download the Debian Package
```bash
curl -LO https://github.com/jyothiprajith1323/TermExplorer/releases/download/v1.1.0/termexplorer_1.1.0_amd64.deb
curl -LO https://github.com/jyothiprajith1323/TermExplorer/releases/download/v1.1.0/SHA256SUMS

# Verify integrity
sha256sum --ignore-missing -c SHA256SUMS
```

### 2. Install Package
```bash
sudo apt-get update
sudo apt-get install -y ./termexplorer_1.1.0_amd64.deb
```
*(Or use `sudo dpkg -i termexplorer_1.1.0_amd64.deb && sudo apt-get install -f`)*

### 3. Verify Debian Installation
```bash
termexplorer --version
termexplorer --health
```

---

## 📦 Method 3: Fedora / RHEL / openSUSE Package (`.rpm`)

For Fedora, Red Hat Enterprise Linux, Rocky Linux, AlmaLinux, and openSUSE:

### 1. Download the RPM Package
```bash
curl -LO https://github.com/jyothiprajith1323/TermExplorer/releases/download/v1.1.0/termexplorer-1.1.0-1.x86_64.rpm
curl -LO https://github.com/jyothiprajith1323/TermExplorer/releases/download/v1.1.0/SHA256SUMS

# Verify integrity
sha256sum --ignore-missing -c SHA256SUMS
```

### 2. Install Package
- **Fedora / RHEL / CentOS Stream:**
  ```bash
  sudo dnf install -y ./termexplorer-1.1.0-1.x86_64.rpm
  ```
- **openSUSE / SUSE Linux Enterprise:**
  ```bash
  sudo zypper install -y ./termexplorer-1.1.0-1.x86_64.rpm
  ```

### 3. Verify RPM Installation
```bash
termexplorer --version
termexplorer --health
```

---

## 🐧 Method 4: Arch Linux (AUR / PKGBUILD)

For Arch Linux, EndeavourOS, and Manjaro:

### Using an AUR Helper (e.g. `paru` or `yay`)
```bash
paru -S termexplorer-git
# or
yay -S termexplorer-git
```

### Manual Compilation from PKGBUILD
```bash
git clone https://github.com/jyothiprajith1323/TermExplorer.git
cd TermExplorer/packaging/aur
makepkg -si
```

---

## 🦀 Method 5: Install via Cargo

If you have a working Rust toolchain (`cargo >= 1.80`):
```bash
cargo install --git https://github.com/jyothiprajith1323/TermExplorer.git --locked
```
This places `termexplorer` into `~/.cargo/bin/`.

---

## 🛠️ Method 6: Build from Source & Package

To build and package TermExplorer directly from source:

```bash
# 1. Clone repository
git clone https://github.com/jyothiprajith1323/TermExplorer.git
cd TermExplorer

# 2. Build and package all release artifacts
./scripts/package-all.sh

# 3. Artifacts will be located in dist/
ls -la dist/
```

---

## 🔐 Package Security & Repository Feed Posture

TermExplorer maintains strict security boundaries for software distribution:

1. **Official Distribution Channel (Production):**
   - Official releases are distributed exclusively as cryptographically hashed artifacts (`.deb`, `.rpm`, `.tar.gz`) attached to GitHub Releases.
   - Every release includes a signed/verified `SHA256SUMS` manifest. Users should always verify SHA-256 digests prior to installation.

2. **Static & Development Repository Generators:**
   - The repository includes helper scripts (`scripts/generate_apt_repo.sh` and `scripts/generate_rpm_repo.sh`) to assemble static APT pools and RPM repodata.
   - These scripts produce **unsigned development repositories** (`gpgcheck=0` / `[trusted=yes]`) designed strictly for internal testing, staging, and local offline mirrors.
   - They are **not** production-ready remote package feeds.

3. **Future Signed Repository Hosting:**
   - Production hosted package feeds (`apt.termexplorer.org` / `rpm.termexplorer.org`) require official, maintainer-controlled private signing keys managed through hardware security modules (HSM) or dedicated GitHub Actions secrets environments.
   - Private keys and mock signing credentials are never checked into source control.

## 🪟 Method 6: WSL2 (Windows Subsystem for Linux 2)

TermExplorer runs natively inside WSL2 distributions (e.g., Ubuntu on WSL2).
1. Open Windows Terminal with your WSL2 Ubuntu profile.
2. Follow **Method 1 (Portable Archive)** or **Method 2 (Debian Package)** above.
3. Configure Windows Terminal to use a Nerd Font (e.g. JetBrainsMono Nerd Font).
4. See [WSL2 Setup Guide](file:///home/prajith/TermExplorer/docs/WSL2_SETUP.md) for advanced clipboard, TrueColor, and font tips.

---

## 🔄 Upgrading TermExplorer

Upgrading TermExplorer is seamless and non-destructive:
- **Portable Archive:** Simply download the new archive and run `./install.sh`. It updates the binary and system files while strictly preserving your user configuration.
- **Debian Package:** Install the newer package with `sudo apt-get install -y ./termexplorer_<NEW_VERSION>_amd64.deb`.
- **User Config Safety:** Your preferences in `~/.config/termexplorer/config.toml` and plugins in `~/.local/share/termexplorer/plugins/` are **never overwritten or altered** during upgrades.

---

## 🗑️ Uninstallation

TermExplorer provides a clean uninstallation procedure that preserves user data by default.

### If Installed via Portable Archive:
```bash
# For user-level installation:
~/.local/share/termexplorer/uninstall.sh

# For system-wide installation:
sudo /usr/local/share/termexplorer/uninstall.sh
```

### If Installed via Debian Package:
```bash
sudo apt-get remove termexplorer
```

### Configuration & Data Retention Guarantee
By default, uninstallation removes application binaries, icons, manual pages, and completions, but **preserves all user configuration files** (`~/.config/termexplorer`). To completely purge user configuration and caches as well, use `--purge-user-data`:
```bash
~/.local/share/termexplorer/uninstall.sh --purge-user-data
```

---

## 🩺 System Diagnostics & Health Check

After installation, run the built-in diagnostic suite to confirm terminal and system capabilities:
```bash
termexplorer --health
```

Example diagnostic report:
```text
TermExplorer System Diagnostics & Health Check
================================================
Version:           1.1.0
Terminal Size:     120 cols × 40 rows
Terminal Support:  $TERM=xterm-256color, $COLORTERM=truecolor, TrueColor=true
Nerd Font:         Detected ($NERD_FONT=1)
PTY Subsystem:     Available (/dev/ptmx, portable-pty)
FreeDesktop Trash: Available (~/.local/share/Trash)
Config Directory:  /home/user/.config/termexplorer
Health Status:     All required workstation subsystems OK.
```
