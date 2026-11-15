# 🚀 TermExplorer — First-Run Experience & Onboarding Guide

This document describes the first-run onboarding architecture, interactive welcome surface, guided setup wizard, and state persistence in **TermExplorer**.

---

## 🧭 1. First-Run Philosophy

TermExplorer is built on an **inspection-first developer philosophy**:
1. **Sensible Defaults:** Launching TermExplorer immediately opens a functional workstation configured with the approved Catppuccin Mocha theme, 25% Explorer / 75% Inspector split, and embedded PTY drawer.
2. **Minimal Interruptions:** Existing users with prior configurations are never interrupted by onboarding wizards when upgrading.
3. **Discoverable & Non-Blocking:** Users can explore the directory immediately (`[Enter]` or `[1]`), walk through the short 2-minute guided setup (`[2]`), view shortcuts (`[3]`), or open settings (`[4]`).
4. **Offline & Autonomous:** Zero external network calls, accounts, telemetry, or cloud dependencies.

---

## 🔍 2. First-Launch Detection Matrix

First-run state is tracked deterministically using XDG base directory locations:
- **Configuration File:** `$XDG_CONFIG_HOME/termexplorer/config.toml` (default: `~/.config/termexplorer/config.toml`).
- **State File:** `$XDG_DATA_HOME/termexplorer/first_run.json` (default: `~/.local/share/termexplorer/first_run.json`).

| System Condition | Detected State | Initial Application Action |
|:---|:---|:---|
| **Neither config nor state exists** | `FreshFirstLaunch` | Launches interactive Welcome Surface (`OnboardingStep::Welcome`) |
| **Valid `config.toml` exists with `first_run_completed = true`** | `ExistingConfigValid` | Launches directly into file exploration (`AppMode::Browse`) |
| **Legacy unversioned (v1) `config.toml` exists** | `ExistingConfigLegacy` | Migrates to Schema v2 in background, marks completed, enters `AppMode::Browse` |
| **Corrupted/malformed `config.toml`** | `ExistingConfigCorrupted` | Preserves corrupted copy as `config.toml.corrupted.<timestamp>`, recovers from backup |
| **Application version upgrade detected** | `ApplicationUpgrade` | Updates state version, increments launch count, preserves user settings |

---

## 🖥️ 3. Interactive Welcome Surface

When launched for the first time, TermExplorer displays an interactive welcome dialog offering 5 immediate actions:

```text
┌──────────────────────── 󰒓 TERMEXPLORER ─ Welcome to TermExplorer ────────────────────────┐
│                                                                                           │
│ Welcome to TermExplorer — Inspection-First Terminal Developer Workstation                 │
│                                                                                           │
│ TermExplorer combines a 25% hierarchical explorer, 75% universal previewer,               │
│ embedded PTY shell drawer, Git branch workbench, and secure WebAssembly extensions.       │
│                                                                                           │
│ Choose an action to begin (use ↑/↓ or number keys 1-5):                                   │
│                                                                                           │
│ ▶ 1. Explore Current Directory     Launch immediately into the active project workspace   │
│   2. Guided Setup Wizard           Customize theme, layout, and audit terminal (2 min)   │
│   3. View Keyboard Shortcuts       Browse complete modal keyboard shortcut reference (F1) │
│   4. Open Settings Studio          Full-screen settings center with 14 categories (F10)   │
│   5. Skip Onboarding               Accept sensible defaults and enter TermExplorer        │
│                                                                                           │
├───────────────────────────────────────────────────────────────────────────────────────────┤
│  [1-5] Choose Action    [Enter] Select    [Esc] Skip Setup                                │
└───────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 🛠️ 4. Guided Setup Sequence

If the user chooses **Guided Setup Wizard** (`[2]`), a 4-step sequence walks through:

1. **Step 1: Color Theme & Appearance:**
   - Choose from 14 canonical color themes (Catppuccin Mocha is default).
   - **Live Preview:** As you scroll through themes with `↑`/`↓`, the active UI palette changes immediately so you can evaluate contrast in real-time.
2. **Step 2: Keyboard Interaction & Keymaps:**
   - Overview of primary modal bindings:
     - Navigation: `j`, `k`, `h`, `l`, `Enter`, `gg`, `G`
     - Terminal Drawer: `` ` `` (Backtick) or `Ctrl-`` (15% height drawer)
     - Command Palette: `Ctrl+Shift+P`
     - Settings Studio: `F10`
     - Clean Exit: `q` or `Ctrl+c`
3. **Step 3: Terminal & Linux Environment:**
   - Real-time audit of host capabilities:
     - Terminal dimensions and `$TERM` / `$COLORTERM` (TrueColor verification)
     - PTY pseudo-terminal availability (`/dev/ptmx`)
     - Default shell path (`$SHELL` / `/bin/bash` / `/usr/bin/zsh`)
     - Detected CLI tools: Git, FFmpeg, Poppler, SQLite, Rust-Analyzer
4. **Step 4: Setup Complete:**
   - Confirmation that preferences are persisted to `~/.config/termexplorer/config.toml`.
   - Pressing `[Enter]` or `[Space]` launches the workstation.

---

## 🔄 5. Replaying Onboarding

Users can reopen the onboarding wizard at any time:
1. **Via Command Palette:** Press `Ctrl+Shift+P`, type `Onboarding`, and select **Run Onboarding Setup Wizard**.
2. **Via Settings Studio:** Press `F10` -> `General` -> select **Replay Onboarding Setup Wizard**.
3. **Programmatically:** Call `termexplorer::config::FirstRunManager::reset()`.
