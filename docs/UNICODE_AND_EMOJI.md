# TermExplorer — Unicode, Emoji & Typography Architecture

## Overview

Terminal emulators operate on a grid of character cells where display columns do not map 1:1 to byte offsets or UTF-8 code points. A naive string slice at arbitrary byte indices panics on multi-byte UTF-8 boundaries, while treating double-width emojis as single-column characters causes misaligned cursors and visual artifacts.

TermExplorer solves these problems across the workstation using `unicode-width` and `unicode-segmentation`.

---

## Cell Width Rules

| Character Category | Example | Byte Length | UTF-8 Code Points | Visual Cell Width |
|---|---|---|---|---|
| ASCII | `A`, `1`, `;` | 1 | 1 | 1 |
| Latin Extended | `é`, `ñ`, `ü` | 2 | 1 | 1 |
| Emoji Single | `🦀`, `🚀`, `📦` | 4 | 1 | 2 |
| Emoji ZWJ Sequence | `👩‍💻`, `👨‍👩‍👧‍👦` | 8–18 | 3–7 | 2 |
| CJK Full-Width | `漢`, `字`, `日` | 3 | 1 | 2 |
| Nerd Font Glyphs | `󰉋`, ``, `` | 3–4 | 1 | 1 or 2 (terminal dependent) |

---

## Implementation Details

### 1. Safe Slicing (`slice_by_display_columns`)
- Iterates over extended grapheme clusters (`grapheme_indices(true)`).
- Never splits code points or combines grapheme clusters midway.
- Calculates exact cumulative cell width.
- Used throughout editor line clipping, preview horizontal scrolling, and explorer path truncation.

### 2. Cursor Positioning
- The editor buffer tracks cursor column by character/grapheme index (`buf.cursor.col`).
- When rendering, `char_index_to_column` converts the character index to the exact visual terminal cell column.
- Eliminates ghost characters and cursor drifts when editing lines containing emojis or East Asian characters.

### 3. Graceful Nerd Font Fallback
TermExplorer implements a multi-tier fallback architecture:
1. **Catppuccin Pack**: Full Nerd Font glyphs with Catppuccin Mocha palette color styling.
2. **Nerd Font Pack**: Standard monochrome Nerd Font glyphs.
3. **Unicode / Emoji Pack**: Universal emojis and symbols (`📁`, `🦀`, `🐍`, `⚙️`) supported by standard system fonts.
4. **ASCII Pack**: Clean bracketed text tags (`[DIR]`, `[RS]`, `[PY]`, `[FILE]`) rendering reliably on any terminal.
5. **No Block Characters in Markdown**: Hardcoded heading blocks `█`, `▓▓`, `▒▒▒` have been replaced with standard clean Markdown headings (`#`, `##`) with subtle underline rules.
