//! Comprehensive Unicode, Emoji, Grapheme Cluster & Display Width Test Suite.

use termexplorer::editor::buffer::{Buffer, Cursor};
use termexplorer::fonts::unicode::{
    char_index_to_column, column_to_char_index, fallback_nerd_font_glyph, is_nerd_font_glyph,
    slice_by_display_columns, split_at_char, str_display_width, truncate_to_display_width,
};
use termexplorer::icons::glyphs::{FILE_PYTHON, FILE_RUST, FOLDER_CLOSED};
use termexplorer::icons::pack::IconPack;

#[test]
fn test_display_widths_ascii_and_multibyte() {
    assert_eq!(str_display_width("hello world"), 11);
    assert_eq!(str_display_width(""), 0);

    // Single-cell Latin extended
    assert_eq!(str_display_width("café"), 4);
    assert_eq!(str_display_width("naïve"), 5);

    // Double-cell emojis
    assert_eq!(str_display_width("🦀"), 2);
    assert_eq!(str_display_width("🚀"), 2);
    assert_eq!(str_display_width("📦"), 2);

    // Mixed text and emojis
    assert_eq!(str_display_width("Rust 🦀 Fast 🚀"), 15);

    // CJK full-width characters
    assert_eq!(str_display_width("漢字"), 4);
    assert_eq!(str_display_width("日本語"), 6);
    assert_eq!(str_display_width("한글"), 4);
}

#[test]
fn test_safe_column_slicing_no_utf8_panics() {
    let text = "Rust 🦀 is blazingly fast 🚀!";

    // Slicing within ASCII
    let (s1, w1) = slice_by_display_columns(text, 0, 4);
    assert_eq!(s1, "Rust");
    assert_eq!(w1, 4);

    // Slicing across emoji boundary (col 5 to 7 is "🦀")
    let (s2, w2) = slice_by_display_columns(text, 5, 2);
    assert_eq!(s2, "🦀");
    assert_eq!(w2, 2);

    // Slicing midway into double-width emoji boundary does not panic
    let (s3, _) = slice_by_display_columns(text, 6, 4);
    assert!(!s3.is_empty());

    // Slicing to end
    let (s4, _) = slice_by_display_columns(text, 0, 100);
    assert_eq!(s4, text);
}

#[test]
fn test_char_and_column_conversions() {
    let s = "A🦀B🚀C";
    // 'A' (char 0, col 0)
    // '🦀' (char 1, col 1, width 2)
    // 'B' (char 2, col 3, width 1)
    // '🚀' (char 3, col 4, width 2)
    // 'C' (char 4, col 6, width 1)

    assert_eq!(char_index_to_column(s, 0), 0);
    assert_eq!(char_index_to_column(s, 1), 1);
    assert_eq!(char_index_to_column(s, 2), 3);
    assert_eq!(char_index_to_column(s, 3), 4);
    assert_eq!(char_index_to_column(s, 4), 6);

    assert_eq!(column_to_char_index(s, 0), 0);
    assert_eq!(column_to_char_index(s, 1), 1);
    assert_eq!(column_to_char_index(s, 2), 1); // inside '🦀'
    assert_eq!(column_to_char_index(s, 3), 2); // 'B'
}

#[test]
fn test_safe_split_at_char() {
    let s = "Hello 🦀 World";
    let (left, right) = split_at_char(s, 6);
    assert_eq!(left, "Hello ");
    assert_eq!(right, "🦀 World");
}

#[test]
fn test_truncate_to_display_width() {
    let s = "Rust 🦀 Language";
    let truncated = truncate_to_display_width(s, 8);
    assert!(truncated.ends_with('…'));
    assert!(str_display_width(&truncated) <= 8);
}

#[test]
fn test_buffer_emoji_editing_and_backspace() {
    let mut buf = Buffer::new();
    assert_eq!(buf.cursor, Cursor { row: 0, col: 0 });

    // Insert text and emoji
    buf.insert_char('R');
    buf.insert_char('u');
    buf.insert_char('s');
    buf.insert_char('t');
    buf.insert_char(' ');
    buf.insert_char('🦀');

    assert_eq!(buf.lines[0], "Rust 🦀");
    assert_eq!(buf.cursor.col, 6);

    // Backspace should delete the single emoji character cleanly
    buf.backspace();
    assert_eq!(buf.lines[0], "Rust ");
    assert_eq!(buf.cursor.col, 5);

    // Undo should restore emoji
    buf.undo();
    assert_eq!(buf.lines[0], "Rust 🦀");
    assert_eq!(buf.cursor.col, 6);

    // Redo should remove it again
    buf.redo();
    assert_eq!(buf.lines[0], "Rust ");
    assert_eq!(buf.cursor.col, 5);
}

#[test]
fn test_nerd_font_glyph_detection_and_fallback() {
    // Nerd font private use area glyphs
    let rust_glyph = '\u{E7A8}';
    assert!(is_nerd_font_glyph(rust_glyph));
    assert_eq!(fallback_nerd_font_glyph(rust_glyph), "🦀");

    // Standard ASCII and Unicode are not Nerd Font PUA
    assert!(!is_nerd_font_glyph('A'));
    assert!(!is_nerd_font_glyph('🦀'));

    // Check built-in fallback sets
    assert_eq!(FOLDER_CLOSED.nerd, "󰉋");
    assert_eq!(FOLDER_CLOSED.unicode, "📁");
    assert_eq!(FOLDER_CLOSED.ascii, "[DIR]");

    assert_eq!(FILE_RUST.unicode, "🦀");
    assert_eq!(FILE_RUST.ascii, "[RS]");

    assert_eq!(FILE_PYTHON.unicode, "🐍");
    assert_eq!(FILE_PYTHON.ascii, "[PY]");
}

#[test]
fn test_icon_pack_modes() {
    assert_eq!(IconPack::parse("catppuccin"), IconPack::Catppuccin);
    assert_eq!(IconPack::parse("nerdfont"), IconPack::NerdFont);
    assert_eq!(IconPack::parse("unicode"), IconPack::Unicode);
    assert_eq!(IconPack::parse("ascii"), IconPack::Ascii);
    assert_eq!(IconPack::parse("none"), IconPack::None);
}
