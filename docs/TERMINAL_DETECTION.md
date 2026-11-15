# TermExplorer Terminal Environment Detection & Hardware Capability Architecture

## 1. Overview & Architectural Philosophy

TermExplorer functions as an inspection-first terminal developer workstation. To provide a predictable, reliable experience across diverse host environments—ranging from native Linux virtual consoles (TTY) to modern GPU-accelerated terminal emulators (Ghostty, Alacritty, Kitty, WezTerm) and containerized or SSH sessions—TermExplorer incorporates an environment detection engine (`src/shell/detection.rs`).

A core design principle of TermExplorer is that **environment variables are advisory hints, never ground truth**. Host environments frequently export misleading values (e.g., `TERM=xterm-256color` in a terminal capable of 24-bit TrueColor, or stale `TERM_PROGRAM` inside an SSH sub-shell). TermExplorer pairs environment variable inspection with concrete hardware, ioctl, and filesystem validation before enabling advanced rendering or PTY execution.

---

## 2. Capabilities Detected

`TerminalCapabilities` provides a typed representation of host terminal capabilities:

| Capability Field | Type | Detection Mechanism | Fallback / Safe Default |
|---|---|---|---|
| `truecolor_supported` | `bool` | Checks `COLORTERM` for `"truecolor"` or `"24bit"`; checks `TERM_PROGRAM` for modern terminals (`"iTerm.app"`, `"ghostty"`, `"wezterm"`, `"alacritty"`, `"kitty"`) | `false` (falls back to 256 or 16 ANSI colors) |
| `unicode_supported` | `bool` | Inspects `LANG`, `LC_ALL`, `LC_CTYPE` for `"UTF-8"` or `"utf8"` | `false` (ASCII fallbacks for borders & icons) |
| `nerd_fonts_supported` | `bool` | Inspects configuration flag `[general].nerd_fonts` and terminal profiles | `false` (Unicode or ASCII icon fallback) |
| `pty_supported` | `bool` | Verifies existence and read/write accessibility of `/dev/ptmx` | `false` (disables terminal drawer cleanly) |
| `dimensions` | `(u16, u16)` | `crossterm::terminal::size()` invoking `ioctl(TIOCGWINSZ)` | `(80, 24)` standard fallback |
| `term_name` | `String` | Inspects `TERM` variable | `"unknown"` |
| `term_program` | `Option<String>` | Inspects `TERM_PROGRAM` variable | `None` |

---

## 3. Shell Discovery & Execution Strategy

TermExplorer inspects the user's host shell to configure the embedded terminal drawer:

### Shell Detection Order
1. **User Configuration**: Checks if the user explicitly set a preferred shell in `config.toml` (`[terminal].default_shell`).
2. **Environment `$SHELL`**: If valid and executable via `std::path::Path::new(&shell).is_file()`, uses this binary.
3. **Known System Paths**: Iterates through standard POSIX shells:
   - `/bin/zsh`, `/usr/bin/zsh`
   - `/bin/bash`, `/usr/bin/bash`
   - `/usr/bin/fish`, `/bin/fish`
   - `/bin/sh`, `/usr/bin/sh`
4. **POSIX Fallback**: If no standard shells are found, falls back safely to `/bin/sh`.

### Login vs Non-Login Shells
Default configuration launches interactive login shells (`-l`) ensuring environment configurations (such as `$PATH`, `$EDITOR`, cargo binaries in `~/.cargo/bin`, and node binaries) are accurately sourced within TermExplorer's embedded PTY drawer.

---

## 4. Host CLI Tool & Integrations Discovery

TermExplorer automatically checks for the availability of external productivity and preview CLI tools in the user's `$PATH`:

| Tool | Purpose in TermExplorer | Detection Target | Fallback Behavior |
|---|---|---|---|
| **Git** | Native Git workbench, branch status, dirty file indicators | `git --version` / `which git` | Git status bar and workbench disabled gracefully |
| **FFmpeg** | Video / audio frame metadata extraction | `ffmpeg -version` / `which ffmpeg` | Raw file metadata displayed without media frames |
| **Poppler** | High-fidelity PDF text rendering | `pdftotext -v` / `which pdftotext` | Built-in fallback text extractor |
| **SQLite** | SQLite structured table preview & schema explorer | `sqlite3 --version` / `which sqlite3` | Built-in SQLite parser |
| **Rust-Analyzer** | Language Server Protocol (LSP) diagnostics & completion | `rust-analyzer --version` | Built-in word-completion engine |

---

## 5. Diagnostic Reporting & Recovery

Users can inspect detected terminal capabilities at any time through the **Terminal Environment** screen in the First-Run Wizard or via **Settings Studio -> Terminal -> Capabilities Diagnostics**:

```text
┌───────────────── Terminal Capabilities Diagnostics ─────────────────┐
│ TERM: xterm-256color                                                │
│ COLORTERM: truecolor                                                │
│ Detected Program: ghostty                                           │
│ Window Dimensions: 120 x 36                                         │
│ TrueColor Support: [Active] 24-bit RGB DirectColor                  │
│ Unicode Support:   [Active] UTF-8 Locale (en_US.UTF-8)              │
│ PTY Availability:  [Active] /dev/ptmx Accessible                    │
│ Default Shell:     /bin/zsh (version 5.9)                           │
│ Detected Tools:    git, ffmpeg, pdftotext, sqlite3, rust-analyzer   │
└─────────────────────────────────────────────────────────────────────┘
```

If terminal capabilities report degraded functionality (e.g., missing TrueColor or broken Unicode width calculation), TermExplorer automatically adjusts rendering parameters to prevent screen tearing and garbled glyphs.
