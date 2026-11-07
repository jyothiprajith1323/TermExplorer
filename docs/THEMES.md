# TermExplorer V1.1 Theme Ecosystem

TermExplorer V1.1 introduces an end-to-end theme and color ecosystem featuring the official canonical **Catppuccin** palette across all four flavors: **Mocha**, **Macchiato**, **Frappé**, and **Latte**.

In addition to built-in themes, TermExplorer provides a custom TOML theme engine with design token mapping, transparent terminal background inheritance, and live preview.

---

## 1. Official Catppuccin Palette (26 Colors)

TermExplorer adopts the exact canonical colors specified by the [Catppuccin Style Guide](https://catppuccin.com/palette/):

| Color Name | Mocha (Default Dark) | Macchiato (Medium Dark) | Frappé (Muted Dark) | Latte (Crisp Light) |
|:---|:---:|:---:|:---:|:---:|
| **Rosewater** | `#f5e0dc` | `#f4dbd6` | `#f2d5cf` | `#dc8a78` |
| **Flamingo**  | `#f2cdcd` | `#f0c6c6` | `#eebebe` | `#dd7878` |
| **Pink**      | `#f5c2e7` | `#f5bde6` | `#f4b8e4` | `#ea76cb` |
| **Mauve**     | `#cba6f7` | `#c6a0f6` | `#ca9ee6` | `#8839ef` |
| **Red**       | `#f38ba8` | `#ed8796` | `#e78284` | `#d20f39` |
| **Maroon**    | `#eba0ac` | `#ee99a0` | `#ea999c` | `#e64553` |
| **Peach**     | `#fab387` | `#f5a97f` | `#ef9f76` | `#fe640b` |
| **Yellow**    | `#f9e2af` | `#eed49f` | `#e5c890` | `#df8e1d` |
| **Green**     | `#a6e3a1` | `#a6da95` | `#a6d189` | `#40a02b` |
| **Teal**      | `#94e2d5` | `#8bd5ca` | `#81c8be` | `#179299` |
| **Sky**       | `#89dceb` | `#91d7e3` | `#99d1db` | `#04a5e5` |
| **Sapphire**  | `#74c7ec` | `#7dc4e4` | `#85c1dc` | `#209fb5` |
| **Blue**      | `#89b4fa` | `#8aadf4` | `#8caaee` | `#1e66f5` |
| **Lavender**  | `#b4befe` | `#b7bdf8` | `#babbf1` | `#7287fd` |
| **Text**      | `#cdd6f4` | `#cad3f5` | `#c6d0f5` | `#4c4f69` |
| **Subtext 1** | `#bac2de` | `#b8c0e0` | `#b5bfe2` | `#5c5f77` |
| **Subtext 0** | `#a6adc8` | `#a5adcb` | `#a5adce` | `#6c6f85` |
| **Overlay 2** | `#9399b2` | `#939ab7` | `#949cbb` | `#7c7f93` |
| **Overlay 1** | `#7f849c` | `#8087a2` | `#838ba7` | `#8c8fa1` |
| **Overlay 0** | `#6c7086` | `#6e738d` | `#737994` | `#9ca0b0` |
| **Surface 2** | `#585b70` | `#5b6078` | `#626880` | `#acb0be` |
| **Surface 1** | `#45475a` | `#494d64` | `#51576d` | `#bcc0cc` |
| **Surface 0** | `#313244` | `#363a4f` | `#414559` | `#ccd0da` |
| **Base**      | `#1e1e2e` | `#24273a` | `#303446` | `#eff1f5` |
| **Mantle**    | `#181825` | `#1e2030` | `#292c3c` | `#e6e9ef` |
| **Crust**     | `#11111b` | `#181926` | `#232634` | `#dce0e8` |

---

## 2. Design Tokens Architecture

TermExplorer abstracts colors into functional UI design tokens via `ThemeTokens`:

```
┌────────────────────────────────────────────────────────┐
│                   Canonical Palette                    │
│   (base, mantle, surface0, text, blue, red, etc.)      │
└───────────────────────────┬────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────┐
│                   Semantic Tokens                      │
│   bg_primary      text_primary      border_focused     │
│   bg_surface      accent_primary    status_error       │
│   fs_directory    fs_symlink        syntax_keyword     │
└───────────────────────────┬────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────┐
│                   UI Component Layer                   │
│   File Explorer   Editor    Monitor   Command Palette  │
└────────────────────────────────────────────────────────┘
```

---

## 3. Custom Themes (TOML Format)

Custom themes are loaded automatically from:
```
~/.config/termexplorer/themes/*.toml
```

### TOML Schema Example:

```toml
[theme]
name = "Cyberpunk Neon"
author = "Runner"
description = "High-voltage terminal palette"
dark = true

[colors]
bg = "#0d0221"
fg = "#00f0ff"
accent = "#ff003c"
selection_bg = "#261447"
selection_fg = "#00f0ff"
border = "#261447"
border_focused = "#fcee0a"
status_fg = "#a5adcb"
error = "#ff003c"
warning = "#fcee0a"
success = "#00ff66"
dir_color = "#00f0ff"
file_color = "#e0e6ed"
symlink_color = "#74c7ec"
hidden_color = "#585b70"
header_fg = "#00f0ff"
header_bg = "#110b29"
```

---

## 4. Switching Themes

- **Settings Studio:** Press `F10`, navigate to `2. Appearance & Themes`, and select your theme.
- **Command Palette:** Press `Ctrl+Shift+P` (or `Space`), search `Switch Theme`, and choose between:
  - `Catppuccin Mocha` (Default dark)
  - `Catppuccin Macchiato`
  - `Catppuccin Frappé`
  - `Catppuccin Latte` (Light)
  - `Dracula`, `Nord`, `Gruvbox`, `Tokyo Night`, `One Dark`, etc.
- **Config File:** In `~/.config/termexplorer/config.toml`:
  ```toml
  [theme]
  name = "Catppuccin Mocha"
  transparent_background = true
  ```
