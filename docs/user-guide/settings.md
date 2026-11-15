# Settings Studio & Personalization

The **Settings Studio** provides full interactive control over TermExplorer's appearance, layout, editor preferences, terminal integration, and keybindings across 14 dedicated categories.

---

## 1. Opening Settings Studio

- Press `Ctrl+,` or `F10` anywhere in the application.
- Or open the Command Palette (`Ctrl+Shift+P`) and select **Open Settings Studio**.
- Press `Esc` or `q` to save changes atomically and return to the workspace.

---

## 2. Interface Layout & Navigation

Settings Studio is organized into two side-by-side columns:
- **Left Column**: The 14 Category Sections. Use `↑` / `↓` or `k` / `j` to browse categories.
- **Right Column**: Configurable items within the active category.
- **Tab Key**: Toggle focus between the category list and the setting items.

### Value Controls
- **Boolean settings**: Press `Space` or `Enter` to toggle `true` / `false`.
- **Numeric settings**: Press `←` / `→` or `h` / `l` to increment or decrement values within clamped safety ranges.
- **Option lists (Themes, Shells)**: Press `Enter` or `→` to cycle available choices.

---

## 3. The 14 Configuration Categories

1. **General**: Startup directory, session restoration, delete confirmation dialogs, Nerd Font icon support.
2. **Appearance**: Color theme selection (Catppuccin Mocha default, Nord, Dracula, Gruvbox, etc.), transparent background toggle, active border accent colors.
3. **Layout**: Default Explorer width ratio (25%), Preview width ratio (75%), and Terminal Drawer height percent (15%).
4. **File Explorer**: Hidden dotfile visibility, sort ordering (Name, Size, Modified, Type), icon pack selection.
5. **Preview / Inspector**: Maximum preview file size limits, hex dump row count, image pixel halfblock rendering.
6. **Editor**: Tab width (1–8 spaces), bracket auto-pairing, line numbering, syntax highlighting toggle, word wrapping, completion popup.
7. **Terminal / PTY**: Default shell path, interactive login shell (`-l`) flag, scrollback buffer limit (128–16384 KiB).
8. **Git Integration**: Git watch mode, auto-status refresh, diff context lines.
9. **Search**: Max search results ceiling, case-sensitivity mode, smart case, follow symlinks toggle.
10. **Developer Tools**: Project tech stack auto-detection, test runner defaults, telemetry polling interval.
11. **System Monitor**: Telemetry refresh interval (100–10000 ms), CPU/Memory chart sparklines.
12. **Keybindings**: Active keybinding preset (Default, Vim, Minimalist), conflict inspection report.
13. **Plugins & Sandbox**: WebAssembly permissions review, plugin enablement toggles, fuel limit display.
14. **Diagnostics**: Real-time frame render latency, FPS counter, hit-testing bounds overlay.

---

## 4. Live Visual Preview

Adjustments in Settings Studio take effect immediately in the running application:
- Switching the theme instantly swaps the 24-bit color palette across panels, borders, and syntax highlighting.
- Changing `preview_ratio` or `terminal_height_percent` updates workstation dimensions dynamically on the next frame.

---

## 5. Item Reset, Reset All, Export & Import

Settings Studio includes quick shortcuts for configuration management:
- **`r` (Reset Single Setting)**: Reverts only the currently highlighted setting to its default factory value.
- **`R` (Reset All Settings)**: Reverts all 14 categories back to standard TermExplorer factory defaults.
- **`e` (Export Configuration)**: Exports current active configuration to `~/.config/termexplorer/config_export.toml`.
- **`i` (Import Configuration)**: Imports and validates settings from `~/.config/termexplorer/config_export.toml`.
