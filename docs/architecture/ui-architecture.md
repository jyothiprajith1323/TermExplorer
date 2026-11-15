# UI Architecture & Ratatui Rendering Engine

TermExplorer's user interface is built on **Ratatui** (v0.30) using an immediate-mode rendering architecture.

---

## 1. Immediate-Mode Layout Pipeline

Ratatui renders the complete user interface on each frame from the underlying application state:

```text
Full Terminal Rect: 120 cols × 40 rows
├── Header Area (1 row): Title, Active Directory, Global Status
├── Main Body Area (Constraint::Min(0)):
│   ├── Left Pane: File Explorer (25% default, Constraint::Percentage(25))
│   └── Right Pane: Contextual Preview / Editor (75% default, Constraint::Percentage(75))
├── Terminal Drawer (Optional, 15% height when visible, Constraint::Length(h))
└── Status Bar Area (1 row): Mode, Git Branch, Key Hints, Diagnostics
```

### Dynamic Layout Splitting
When the embedded terminal drawer is opened:
- The main body height is compressed by the drawer's allocated row count (15% of terminal height).
- The ratio between the File Explorer (25%) and Preview pane (75%) is maintained horizontally.
- Layout splits are evaluated mathematically via `ratatui::layout::Layout` using `Direction::Vertical` and `Direction::Horizontal`.

---

## 2. Double-Buffering & Terminal Diffing

Ratatui manages two terminal screen buffers:
- **Previous Buffer**: The state of every cell on the user's terminal.
- **Current Buffer**: The newly constructed frame rendered during `terminal.draw()`.

During the render commit phase:
1. Ratatui compares the current buffer with the previous buffer cell by cell.
2. Only cells whose character, foreground color, background color, or modifier attributes changed are emitted as ANSI escape sequences.
3. This minimizes I/O overhead over SSH or slow terminal emulators, yielding high redraw performance (<2 ms per frame).

---

## 3. Theme Engine & Color Palette

All UI components source colors from the active `Theme` struct (`src/theme.rs`):
- **Catppuccin Mocha** (Default):
  - Base Background: `#1e1e2e` (or `None` when `transparent_background = true`)
  - Active Border: `#89b4fa` (Sapphire / Blue)
  - Inactive Border: `#45475a` (Surface1)
  - Selection Highlight: `#585b70` (Surface2)
  - Text Primary: `#cdd6f4` (Text)
  - Text Subtitle / Dim: `#a6adc8` (Subtext0)
  - Success / Git Added: `#a6e3a1` (Green)
  - Error / Git Deleted: `#f38ba8` (Red)
  - Warning / Git Modified: `#f9e2af` (Yellow)

---

## 4. Diagnostics Overlay (`F12`)

Pressing `F12` toggles the Diagnostics Overlay, providing real-time telemetry:
- Frame render duration in microseconds ($\mu\text{s}$)
- Instantaneous and rolling average Frames Per Second (FPS)
- Active memory allocation estimate
- Terminal row and column dimensions
- Hit-test bounding boxes
