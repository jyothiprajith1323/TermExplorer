//! Terminal graphics capabilities and image protocol detection.

use ratatui::style::Color;
use std::env;
use std::sync::OnceLock;

/// Graphics protocols supported by modern terminal emulators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalGraphicsProtocol {
    /// Kitty Graphics Protocol (APC `_G...;payload\x1b\\`)
    Kitty,
    /// Sixel bitmap graphics (DCS `P...#...-...ST`)
    Sixel,
    /// iTerm2 / WezTerm Inline Images (OSC `1337;File=...`)
    ITerm2,
    /// Unicode Half-Block Truecolor cells (`▀` foreground/background pair)
    HalfBlocks,
    /// Metadata and description only (minimal terminal fallback)
    TextOnly,
}

static DETECTED_PROTOCOL: OnceLock<TerminalGraphicsProtocol> = OnceLock::new();

/// Detect the best available terminal graphics protocol from environment and queries.
pub fn detect_graphics_protocol() -> TerminalGraphicsProtocol {
    *DETECTED_PROTOCOL.get_or_init(|| {
        let term = env::var("TERM").unwrap_or_default().to_lowercase();
        let term_program = env::var("TERM_PROGRAM").unwrap_or_default().to_lowercase();
        let kitty_window = env::var("KITTY_WINDOW_ID");
        let ghostty_resources = env::var("GHOSTTY_RESOURCES_DIR");

        // 1. Kitty Graphics Protocol (Native Kitty, WezTerm, Ghostty)
        if kitty_window.is_ok() || ghostty_resources.is_ok() || term.contains("kitty") {
            return TerminalGraphicsProtocol::Kitty;
        }

        // 2. iTerm2 Protocol
        if term_program.contains("iterm") || term_program.contains("wezterm") {
            return TerminalGraphicsProtocol::ITerm2;
        }

        // 3. Sixel Protocol (Foot, mlterm, xterm with sixel)
        if term.contains("sixel") || term.contains("foot") || term.contains("mlterm") {
            return TerminalGraphicsProtocol::Sixel;
        }

        // 4. Default: High-fidelity Half-Block TrueColor rendering
        TerminalGraphicsProtocol::HalfBlocks
    })
}

/// Convert an in-memory DynamicImage into a 2D grid of Ratatui truecolor halfblocks (`▀`).
/// Each character cell represents 2 vertical pixels:
/// - Top pixel: Foreground color of `▀`
/// - Bottom pixel: Background color of `▀`
pub fn render_halfblocks_from_image(
    img: &image::DynamicImage,
    target_width: u32,
    target_height: u32,
) -> Vec<Vec<(Color, Color, char)>> {
    let target_w = target_width.max(1);
    let target_h = target_height.max(1);

    // Double vertical resolution because each half-block cell holds 2 vertical pixels
    let pixel_w = target_w;
    let pixel_h = target_h * 2;

    let resized = img.resize_exact(pixel_w, pixel_h, image::imageops::FilterType::Triangle);
    let rgb = resized.to_rgba8();

    let mut grid = Vec::with_capacity(target_h as usize);

    for row in 0..target_h {
        let mut row_cells = Vec::with_capacity(target_w as usize);
        let y_top = row * 2;
        let y_bottom = row * 2 + 1;

        for x in 0..target_w {
            let top_p = rgb.get_pixel(x, y_top);
            let bottom_p = if y_bottom < pixel_h {
                rgb.get_pixel(x, y_bottom)
            } else {
                top_p
            };

            let fg = Color::Rgb(top_p[0], top_p[1], top_p[2]);
            let bg = Color::Rgb(bottom_p[0], bottom_p[1], bottom_p[2]);

            // Handle transparency if alpha is low
            let (final_fg, final_bg, ch) = if top_p[3] < 32 && bottom_p[3] < 32 {
                (Color::Reset, Color::Reset, ' ')
            } else if top_p[3] < 32 {
                (
                    Color::Rgb(bottom_p[0], bottom_p[1], bottom_p[2]),
                    Color::Reset,
                    '▄',
                )
            } else if bottom_p[3] < 32 {
                (Color::Rgb(top_p[0], top_p[1], top_p[2]), Color::Reset, '▀')
            } else {
                (fg, bg, '▀')
            };

            row_cells.push((final_fg, final_bg, ch));
        }
        grid.push(row_cells);
    }

    grid
}
