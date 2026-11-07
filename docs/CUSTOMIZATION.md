# TermExplorer V1.1 Personalization & Customization Guide

TermExplorer is designed to be personalized across themes, icons, typography, plugins, workspaces, and layouts.

---

## 1. Quick Personalization Map

```
~/.config/termexplorer/
├── config.toml           # Main user configuration & panel settings
├── icons.toml            # Custom icon mappings & pack choice
├── themes/               # Custom TOML color themes
│   └── *.toml
└── backups/              # Automated configuration snapshots
~/.local/share/termexplorer/
└── plugins/              # WebAssembly sandboxed plugins
    └── <plugin-id>/
        ├── plugin.toml
        └── plugin.wasm
```

---

## 2. Interactive Settings Studio (`F10`)

The easiest way to customize TermExplorer is via the built-in **Settings Studio**:
- Press `F10` from anywhere in TermExplorer.
- Use `Up` / `Down` (`k` / `j`) to navigate sections.
- Press `Tab` to switch between section index and setting items.
- Press `Enter`, `Space`, or `Left` / `Right` to change options and toggle switches.
- Press `Esc` or `F10` to automatically save changes with backup creation.

---

## 3. Command Palette Quick Actions (`Ctrl+Shift+P` / `Space`)

Search and trigger instant actions:
- `Switch Theme: Catppuccin Mocha` / `Macchiato` / `Frappé` / `Latte`
- `Switch Icons: Catppuccin Nerd Font` / `Unicode Emojis` / `ASCII`
- `Typography: JetBrains Mono Setup Guide`
- `Plugins: Discover & Reload Plugins`
- `Apply Profile: Developer` / `Minimal` / `Terminal Power User` / `System Monitoring`

---

## 4. Configuration Schema (`config.toml`)

```toml
# Panel Dimensions
sidebar_width = 20
preview_width = 35
terminal_height = 12
monitor_height = 14

# Visibility
show_sidebar = true
show_preview = true
show_terminal = false
show_monitor = false
show_hidden = false

# Icon Pack: "catppuccin", "nerdfont", "unicode", "ascii", "none"
icon_pack = "catppuccin"

[theme]
name = "Catppuccin Mocha"
accent = "#89b4fa"
transparent_background = true

[editor]
tab_width = 4
auto_pair = true
line_numbers = true
syntax_highlight = true
word_wrap = true
show_completion = true
```
