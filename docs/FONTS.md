# TermExplorer V1.1 Typography & JetBrains Mono Support

TermExplorer is optimized for **JetBrains Mono** and **JetBrainsMono Nerd Font Mono**, providing code readability and symbol alignment.

---

## 1. Terminal Emulator Font Architecture

> [!NOTE]
> Terminal emulators (like Kitty, Alacritty, GNOME Terminal, and WezTerm) control host typography independently from CLI/TUI applications.
> A terminal program cannot silently change your terminal emulator font without user configuration.
> TermExplorer inspects font availability on your system and provides configuration snippets for your specific terminal.

---

## 2. Host Font Inspection

TermExplorer uses `fontconfig` (`fc-list` and `fc-match`) to detect available fonts on your machine:

- **JetBrainsMono Nerd Font Mono:** Fully supported with Nerd Font v3 icons.
- **JetBrains Mono (Standard):** Monospace font with Unicode/ASCII fallback icons.
- **System Fallback:** Automatically switches to `Unicode` or `ASCII` mode if Nerd Fonts are absent.

You can inspect your current font status in:
- **Settings Studio (`F10`)** -> `2. Appearance & Themes` -> `Host Font Detection`.
- **Command Palette (`Ctrl+Shift+P`)** -> `Typography: JetBrains Mono Setup Guide`.

---

## 3. Terminal Emulator Configuration Guides

### 1. Kitty
Add to `~/.config/kitty/kitty.conf`:
```conf
font_family      JetBrainsMono Nerd Font Mono
bold_font        auto
italic_font      auto
font_size        12.0
```
Reload shortcut: `Ctrl+Shift+F5`.

---

### 2. Alacritty
Add to `~/.config/alacritty/alacritty.toml`:
```toml
[font]
size = 12.0

[font.normal]
family = "JetBrainsMono Nerd Font Mono"
style = "Regular"
```
Alacritty reloads the configuration live upon saving.

---

### 3. WezTerm
Add to `~/.wezterm.lua`:
```lua
local wezterm = require 'wezterm'
local config = {}

config.font = wezterm.font('JetBrainsMono Nerd Font Mono')
config.font_size = 12.0

return config
```
WezTerm applies font updates automatically.

---

### 4. GNOME Terminal
1. Open GNOME Terminal.
2. Navigate to `Preferences` -> `Profiles` -> Select Profile.
3. Open the `Text` tab.
4. Check **Custom font**.
5. Select **JetBrainsMono Nerd Font Mono Regular 11**.

---

## 4. User-Space Local Font Installer

If JetBrains Mono Nerd Font is not installed on your system, install it locally into `~/.local/share/fonts/` without needing root/sudo permissions:

```bash
mkdir -p ~/.local/share/fonts/NerdFonts
cd $(mktemp -d)

curl -fLo "JetBrainsMono.tar.xz" \
  "https://github.com/ryanoasis/nerd-fonts/releases/latest/download/JetBrainsMono.tar.xz"

tar -xf "JetBrainsMono.tar.xz" -C ~/.local/share/fonts/NerdFonts
fc-cache -fv ~/.local/share/fonts/NerdFonts

echo "Installed successfully! Restart your terminal emulator."
```
