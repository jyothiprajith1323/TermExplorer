# Workstation Overview & Layout Architecture

TermExplorer is built around an **inspection-first** developer workflow. Unlike traditional file managers that treat files as passive icons, TermExplorer treats code and files as active, inspectable artifacts.

---

## 1. Default Workstation Layout

The workstation layout is optimized for rapid inspection without excessive window toggling:

```text
┌── EXPLORER (25%) ──┬── PREVIEW / INSPECTOR (75%) [Preview|Info|Hex|Actions] ──────────────┐
│ SIZE   NAME        │ Type: Rust Source File (14.2 KB)                                     │
│ DIR    src/        │  1  use anyhow::Result;                                              │
│ DIR    tests/      │  2  use ratatui::widgets::Paragraph;                                 │
│ 2.4 KB Cargo.toml  │  3                                                                   │
│ 12 KB  README.md   │  4  #[derive(Debug, Clone)]                                          │
│ 14 KB >main.rs     │  5  pub struct AppEngine {                                           │
│  1 KB  build.rs    │  6      pub terminal: TerminalSession,                               │
│                    │  7  }                                                                │
├────────────────────┴──────────────────────────────────────────────────────────────────────┤
│ termexplorer:~/Projects/TermExplorer$ cargo test --all-targets                            │
│    Finished `test` profile in 0.45s — 129 passed; 0 failed                                │
└───────────────────────────────────────────────────────────────────────────────────────────┘
 NORMAL  ~/Projects/TermExplorer/src/main.rs | Git: master* | F1:Help F10:Settings ` :Term
```

### Layout Proportions & Invariants
- **File Explorer Panel**: Occupies **25%** of the workstation width by default (configurable between 15% and 50%).
- **Preview / Inspector Panel**: Occupies **75%** of the workstation width by default (configurable between 50% and 85%).
- **Terminal Drawer**: Collapsible bottom drawer occupying **15%** of workstation height when opened (configurable between 10% and 40%). Hidden by default.
- **Status Bar**: A single line at the bottom displaying active mode, path, Git branch indicator, and context-sensitive shortcut hints.

---

## 2. Focus Cycling Model

Keyboard input is directed to whichever panel currently holds focus:

| Panel | Indicator | Focus Key |
|---|---|---|
| **File Explorer** | Blue highlight border (Catppuccin Mocha `#89b4fa`) | `Tab` or `Esc` (from modal/preview) |
| **Preview / Inspector** | Highlighted border and active tab indicator | `Space` or `Tab` |
| **Terminal Drawer** | Active cursor inside terminal screen | `` ` `` (toggle) or `Tab` |

Cycle focus sequentially across panels with `Tab` (forward) or `Shift+Tab` (backward).

---

## 3. Responsive Resizing

The layout responds dynamically to terminal window resizing:
- Expand preview width (+5%): `Ctrl+Right` or `Alt+]`
- Shrink preview width (-5%): `Ctrl+Left` or `Alt+[`
- Maximize preview to 100% width: `F3` or `F11` (restores original 25%/75% split when toggled again)
- Expand terminal height (+2 rows): `Ctrl+Up` or `Alt+}`
- Shrink terminal height (-2 rows): `Ctrl+Down` or `Alt+{`

All layout adjustments respect hard boundaries (Explorer ratio never shrinks below 15% or exceeds 50%; Terminal height stays between 10% and 40%).
