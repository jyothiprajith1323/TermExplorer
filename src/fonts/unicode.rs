//! Unicode and Grapheme Display Width Utilities.
//!
//! Provides grapheme-aware display cell calculations, safe slicing without UTF-8
//! boundary panics, and multi-tier icon fallbacks (Nerd Font -> Unicode symbol -> ASCII).

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Calculate visual display cell width of a string in terminal columns.
/// Accounts for wide characters (emojis, CJK) and zero-width joiners / variation selectors.
pub fn str_display_width(s: &str) -> usize {
    s.width()
}

/// Calculate visual display cell width of a single character.
pub fn char_display_width(c: char) -> usize {
    c.width().unwrap_or(0)
}

/// Slice a string safely by terminal display column width, without splitting multi-byte
/// UTF-8 characters or grapheme clusters.
/// Returns the sliced substring and the actual displayed cell width.
pub fn slice_by_display_columns(s: &str, start_col: usize, max_cols: usize) -> (&str, usize) {
    if max_cols == 0 || s.is_empty() {
        return ("", 0);
    }

    let mut current_col = 0;
    let mut start_byte = None;
    let mut end_byte = s.len();

    for (byte_idx, grapheme) in s.grapheme_indices(true) {
        let g_width = grapheme.width();

        // Find the start byte
        if start_byte.is_none() {
            if current_col >= start_col {
                start_byte = Some(byte_idx);
            } else if current_col + g_width > start_col {
                // Grapheme crosses the start_col boundary; start after it to avoid clipping
                start_byte = Some(byte_idx + grapheme.len());
            }
        }

        // Check if we reached the maximum column limit
        if start_byte.is_some() {
            let width_from_start = current_col.saturating_sub(start_col);
            if width_from_start + g_width > max_cols {
                end_byte = byte_idx;
                break;
            }
        }

        current_col += g_width;
    }

    let start = start_byte.unwrap_or(s.len()).min(s.len());
    let end = end_byte.min(s.len()).max(start);

    let slice = &s[start..end];
    (slice, slice.width())
}

/// Convert a character index in a string to its visual display column on a terminal.
pub fn char_index_to_column(s: &str, char_idx: usize) -> usize {
    s.chars()
        .take(char_idx)
        .map(|c| c.width().unwrap_or(0))
        .sum()
}

/// Convert a visual display column on a terminal to the closest character index in a string.
pub fn column_to_char_index(s: &str, target_col: usize) -> usize {
    let mut current_col = 0;
    for (idx, c) in s.chars().enumerate() {
        let w = c.width().unwrap_or(0);
        if current_col + w > target_col {
            return idx;
        }
        current_col += w;
    }
    s.chars().count()
}

/// Convert a visual column to a safe byte index in a string.
pub fn column_to_byte_index(s: &str, target_col: usize) -> usize {
    let mut current_col = 0;
    for (byte_idx, grapheme) in s.grapheme_indices(true) {
        let w = grapheme.width();
        if current_col + w > target_col {
            return byte_idx;
        }
        current_col += w;
    }
    s.len()
}

/// Safely split a string at a character index (not byte index), preventing UTF-8 panics.
pub fn split_at_char(s: &str, char_idx: usize) -> (&str, &str) {
    let byte_idx = s
        .char_indices()
        .nth(char_idx)
        .map(|(b, _)| b)
        .unwrap_or(s.len());
    s.split_at(byte_idx)
}

/// Truncate a string to a maximum terminal column width, appending an ellipsis if truncated.
pub fn truncate_to_display_width(s: &str, max_width: usize) -> String {
    if s.width() <= max_width {
        return s.to_string();
    }
    if max_width <= 1 {
        return "…".to_string();
    }

    let target_width = max_width.saturating_sub(1);
    let (prefix, _) = slice_by_display_columns(s, 0, target_width);
    format!("{prefix}…")
}

/// Check if a character is a Nerd Font private-use area glyph (E000–F8FF, F0000–FFFFD, 100000–10FFFD).
pub fn is_nerd_font_glyph(c: char) -> bool {
    matches!(
        c as u32,
        0xE000..=0xF8FF | 0xF0000..=0xFFFFD | 0x100000..=0x10FFFD
    )
}

/// Fallback translation for common Nerd Font glyphs to standard Unicode or ASCII characters
/// for environments lacking Nerd Fonts.
pub fn fallback_nerd_font_glyph(c: char) -> &'static str {
    match c {
        // Rust logo
        '\u{E7A8}' => "🦀",
        // Folder closed / open
        '\u{F413}' | '\u{E5FE}' => "📁",
        '\u{F412}' | '\u{E5FF}' => "📂",
        // File generic
        '\u{F15C}' | '\u{F016}' => "📄",
        // Git branch
        '\u{E725}' | '\u{F126}' => "🌿",
        // Settings / gear
        '\u{F013}' | '\u{E5FC}' => "⚙",
        // Search
        '\u{F002}' => "🔍",
        // Terminal
        '\u{F120}' => "",
        // Warning / Error
        '\u{F071}' => "⚠",
        '\u{F057}' | '\u{F00D}' => "✖",
        // Check
        '\u{F00C}' => "✔",
        _ => "•",
    }
}
