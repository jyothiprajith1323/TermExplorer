# Environment Variables Reference

This reference documents the environment variables inspected and respected by TermExplorer.

---

## 1. Terminal & Hardware Detection Variables

| Variable | Values / Examples | Purpose in TermExplorer |
|---|---|---|
| `TERM` | `xterm-256color`, `alacritty`, `ghostty`, `xterm-kitty` | Advises terminal type and capabilities. |
| `COLORTERM` | `truecolor`, `24bit` | Advises whether 24-bit RGB DirectColor rendering is supported. |
| `TERM_PROGRAM`| `ghostty`, `WezTerm`, `iTerm.app`, `vscode` | Identifies specific host terminal emulators for feature detection. |
| `LANG` / `LC_ALL` | `en_US.UTF-8`, `C.UTF-8` | Inspects whether the active system locale supports UTF-8 Unicode characters. |
| `NERD_FONT` | `1`, `true`, `0`, `false` | Environment override for Nerd Font icon rendering. |

---

## 2. Shell & Execution Variables

| Variable | Default Fallback | Purpose in TermExplorer |
|---|---|---|
| `SHELL` | `/bin/sh` | The default user shell spawned in the embedded PTY terminal drawer. |
| `EDITOR` / `VISUAL` | `nano` / `vi` | Fallback editor used when selecting "Open in External Editor" from preview actions. |

---

## 3. Storage & XDG Base Directory Variables

| Variable | Default Fallback | Purpose in TermExplorer |
|---|---|---|
| `XDG_CONFIG_HOME` | `~/.config` | Directory where configuration is stored (`~/.config/termexplorer/config.toml`). |
| `XDG_DATA_HOME` | `~/.local/share` | Directory where logs, trust stores, and plugins are stored (`~/.local/share/termexplorer/`). |
| `XDG_CACHE_HOME` | `~/.cache` | Directory where persistent preview caches are stored (`~/.cache/termexplorer/previews/`). |
| `RUST_LOG` | `info` | Filter directive for tracing logger (e.g. `debug`, `trace`, `termexplorer::editor=trace`). |
