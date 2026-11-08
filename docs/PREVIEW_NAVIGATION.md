# TermExplorer — Preview Navigation & Scrolling Architecture

## Overview

The Contextual Preview panel occupies 75% of the workstation width and displays rich multi-format content:
- Source code with syntax highlighting
- Markdown documents with table of contents
- Images (dimensions, format, ASCII preview)
- Audio / Video metadata
- PDF page counts and metadata
- Archives (ZIP/TAR/GZ file lists)
- Databases (SQLite tables, schemas)
- Hex / Binary inspector
- File action shortcuts

---

## Scrolling Architecture & Clamping

### Vertical Scrolling
- **Unbounded scroll bug resolved**: Previous versions incremented `scroll_offset` on mouse wheel without bounds checking, which scrolled the view past the end of the file into empty space.
- **Strict Clamping**: All scroll inputs clamp `scroll_offset` to `total_lines.saturating_sub(1)`:
  - Down Arrow / `j`: +1 line
  - Up Arrow / `k`: -1 line
  - PageDown: +20 lines (clamped to EOF)
  - PageUp: -20 lines (clamped to 0)
  - Home: First line (`offset = 0`)
  - End: Last line (`offset = total_lines - 1`)

### Mouse Wheel Hover Routing
- TermExplorer detects the mouse cursor position against panel layout boundaries (`self.layout.contains_preview(x, y)`).
- When the mouse wheel is rotated over the Preview panel, preview scrolling is triggered even if the Explorer panel currently has focus.

### Horizontal Scrolling
- Long code lines or wide tables can be scrolled horizontally using:
  - `Shift + Left` / `H`: Scroll left 4 columns
  - `Shift + Right` / `L`: Scroll right 4 columns
  - `Home`: Reset horizontal offset to 0

### Tabs Navigation
- `Tab` / `]` : Next preview tab (`Preview` → `Info` → `Metadata` → `Actions`)
- `BackTab` / `[` : Previous preview tab
- `h` / `Esc` : Exit preview focus back to file explorer
