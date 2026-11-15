# 🐧 TermExplorer — WSL2 (Windows Subsystem for Linux) Setup & Integration Guide

This guide details configuring, optimizing, and running **TermExplorer** inside WSL2 (Windows Subsystem for Linux 2) on Windows 10/11.

---

## 📋 Executive Overview & Platform Boundary

> [!IMPORTANT]
> **WSL2 Compatibility vs Native Windows:**
> Running TermExplorer inside WSL2 is a **Linux x86_64 or AArch64 execution environment** hosted by the Linux kernel on Hyper-V. It is **not** a native Win32/NT port. 
> All POSIX terminal interfaces, PTY allocation (`/dev/ptmx`), signals (`SIGINT`, `SIGTERM`, `SIGWINCH`), and Linux filesystem conventions (XDG base directories) function identically to bare-metal Linux.

---

## 🛠️ Recommended Terminal Setup: Windows Terminal

For the best experience, use **Windows Terminal** (v1.18+ or Windows Terminal Preview):

### 1. TrueColor (24-bit RGB) Support
TermExplorer uses Catppuccin Mocha as its default theme and benefits from 24-bit RGB TrueColor. In your WSL2 bash/zsh shell:
```bash
export COLORTERM=truecolor
```
Add this line to your `~/.bashrc` or `~/.zshrc`. You can verify color support anytime by running:
```bash
termexplorer --health
```

### 2. Nerd Font Configuration
To display file icons, Git branch glyphs, and UI markers with high fidelity, install a Nerd Font in Windows and configure Windows Terminal to use it.

1. Download and install **JetBrainsMono Nerd Font** (or **FiraCode Nerd Font**) on Windows.
2. In Windows Terminal: `Settings (Ctrl+,) -> Ubuntu / WSL profile -> Appearance -> Font face`.
3. Select `JetBrainsMono Nerd Font` (or `JetBrainsMono NF`).
4. In your WSL2 environment, set:
   ```bash
   export NERD_FONT=1
   ```
   *Note:* If `NERD_FONT` is not set or font glyphs are unsupported, TermExplorer automatically degrades gracefully to standard Unicode symbols and ASCII representations.

---

## 🗂️ Filesystem Navigation & Performance Best Practices

WSL2 provides access to both the internal Linux virtual disk (`ext4`) and the host Windows NTFS drives (`/mnt/c/`, `/mnt/d/`).

### Recommendation: Keep Active Projects in the Linux Filesystem
- **Linux Filesystem (`/home/<user>/...`):** Full native `ext4` performance. Inotify file watching (`notify` crate), Git operations, and sub-millisecond tree traversals operate at peak speed (~17 ms per 1,000 files).
- **Windows Drives (`/mnt/c/Users/...`):** Cross-OS file operations via the `9P` / `drvfs` protocol introduce network-like latency and may not propagate inotify filesystem events reliably.
- TermExplorer seamlessly inspects files across `/mnt/c/`, but for high-throughput development, storing projects under `~/` or `/home/<user>/` is strongly advised.

---

## 💻 Clipboard Integration in WSL2

TermExplorer supports clipboard integration for file path copying and editor yank operations.
Inside WSL2, Windows clipboard tools can be integrated seamlessly:

```bash
# If using Windows Terminal and xclip/wl-clipboard:
sudo apt-get install -y wl-clipboard xclip

# Alternatively, alias to Windows clip.exe if headless:
if grep -qi microsoft /proc/version; then
    alias pbcopy="clip.exe"
fi
```

---

## 🖵 PTY Interactive Shell & Subprocess Lifecycle

The embedded terminal drawer in TermExplorer uses Unix Pseudo-Terminals (`portable-pty` targeting `/dev/ptmx`).
In WSL2:
- PTY allocation operates cleanly under the WSL2 Linux kernel.
- The default shell spawned is determined by your WSL2 user `$SHELL` environment variable (typically `/bin/bash` or `/usr/bin/zsh`).
- Terminal resize events (`SIGWINCH`) are fully handled when resizing Windows Terminal.
- To open the embedded terminal drawer at any time, press **`** (backtick) or **Ctrl-`**.

---

## 🧪 WSL2 Verification & Diagnostics

To verify your WSL2 setup before running TermExplorer:

```bash
# 1. Run diagnostic health check
termexplorer --health

# 2. Verify terminal dimensions and color support
echo "TERM=$TERM, COLORTERM=$COLORTERM"

# 3. Test clean exit and terminal restoration
termexplorer --version
```
