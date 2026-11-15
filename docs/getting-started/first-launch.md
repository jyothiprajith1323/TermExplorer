# First Launch & Onboarding Experience

TermExplorer features an inspection-first design that respects developer focus. It distinguishes between fresh installations, upgrades, existing configurations, and corrupted states.

---

## 1. First-Run State Detection

When TermExplorer executes, `FirstRunManager` evaluates two files:
1. User configuration: `~/.config/termexplorer/config.toml`
2. Application state record: `~/.local/share/termexplorer/first_run.json`

| Condition | Detected State | Workstation Action |
|---|---|---|
| Neither config nor state file exists | `FreshFirstLaunch` | Displays the Welcome Surface overlay |
| Configuration and matching state exist | `ExistingUser` | Directly enters workstation with 0 prompt or delay |
| Configuration exists with older state version | `Upgrade` | Migrates schema silently; creates notification |
| Configuration file exists but contains invalid TOML | `CorruptedConfig` | Archives corrupted file and loads fresh defaults |

---

## 2. The Welcome Surface

On a fresh installation, TermExplorer launches with the Welcome Surface centered over the inactive workstation layout:

```text
┌──────────────────────── Welcome to TermExplorer ────────────────────────┐
│                                                                        │
│   TermExplorer v1.1.0                                                  │
│   Inspection-first terminal developer workstation                      │
│                                                                        │
│   [1] Browse Current Directory (Default workstation layout)            │
│   [2] Guided Setup Wizard (Configure theme, keys, terminal)            │
│   [3] View Keyboard Shortcuts Cheat-Sheet                              │
│   [4] Open Settings Studio (Full preferences editor)                   │
│   [5] Skip Onboarding & Never Show Again                               │
│                                                                        │
│   Press [1-5] or [Enter] to choose, [Esc] to dismiss                   │
└────────────────────────────────────────────────────────────────────────┘
```

### The 5 Operational Pathways

1. **`[1]` or `Enter`: Browse Current Directory**
   - Marks onboarding complete and persists `first_run_completed = true`.
   - Closes overlay immediately and places focus into the File Explorer.
   - Workstation boots into standard default layout: 25% Explorer / 75% Preview.

2. **`[2]`: Guided Setup Wizard**
   - Starts a 5-step interactive customization flow.

3. **`[3]`: View Keyboard Shortcuts Cheat-Sheet**
   - Opens the global keyboard shortcuts cheat-sheet overlay (`?` / `F1`).

4. **`[4]`: Open Settings Studio**
   - Directly launches the interactive Settings Studio (`F10` / `Ctrl+,`) across all 14 categories.

5. **`[5]` or `Esc`: Skip Onboarding**
   - Immediately closes overlay and saves state so it is never shown again.

---

## 3. The Guided Setup Wizard Steps

If option `[2]` is selected, the wizard guides you through 5 concise configuration steps:

1. **Welcome**: Introduces the workstation layout and core philosophies.
2. **Appearance & Themes**:
   - Choose default theme: Catppuccin Mocha (default), Nord, Dracula, Gruvbox, One Dark, Solarized Dark, or Minimal Light.
   - Toggle Nerd Font icon glyphs vs standard ASCII indicators.
3. **Keyboard Interaction**:
   - Choose navigation preset: Default Ergonomic, Vim Navigation (`hjkl`), or Minimalist.
4. **Terminal Environment**:
   - Review detected terminal emulator, color capabilities, TrueColor support, and `/dev/ptmx` PTY availability.
   - Select default shell binary (`/bin/zsh`, `/bin/bash`, `/usr/bin/fish`, or `/bin/sh`).
   - Enable or disable interactive login shell flag (`-l`).
5. **Completion**:
   - Displays a summary of selected preferences.
   - Saves settings atomically to `~/.config/termexplorer/config.toml`.
   - Enters the workstation ready for development.

---

## 4. Replaying Onboarding

If you wish to revisit the Welcome Surface or re-run the Guided Setup Wizard at any point:
1. Open the Command Palette with `Ctrl+Shift+P` (or `Ctrl+P`).
2. Type `onboarding` or `setup`.
3. Press `Enter` on **Start Guided Setup Wizard**.
