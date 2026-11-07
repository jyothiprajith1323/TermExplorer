//! Terminal graphics capabilities and image-to-halfblock rendering.

use ratatui::style::Color;
use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalGraphicsProtocol {
    Kitty,
    Sixel,
    ITerm2,
    HalfBlockTrueColor,
    AsciiFallback,
}

pub struct GraphicsCapability;

impl GraphicsCapability {
    /// Detect supported graphics protocol based on terminal environment variables.
    pub fn detect() -> TerminalGraphicsProtocol {
        // Kitty detection
        if env::var("KITTY_WINDOW_ID").is_ok() || env::var("KITTY_PID").is_ok() {
            return TerminalGraphicsProtocol::Kitty;
        }

        // WezTerm, Ghostty, Foot often support Sixel or Kitty
        if let Ok(term_program) = env::var("TERM_PROGRAM") {
            let tp = term_program.to_ascii_lowercase();
            if tp.contains("wezterm") || tp.contains("ghostty") {
                return TerminalGraphicsProtocol::Kitty;
            }
            if tp.contains("iterm") {
                return TerminalGraphicsProtocol::ITerm2;
            }
        }

        // Colorterm truecolor / 24-bit support
        if let Ok(colorterm) = env::var("COLORTERM") {
            if colorterm == "truecolor" || colorterm == "24bit" {
                return TerminalGraphicsProtocol::HalfBlockTrueColor;
            }
        }

        TerminalGraphicsProtocol::HalfBlockTrueColor
    }

    /// Render an `image::DynamicImage` into a 2D grid of Ratatui styled spans using half-blocks `▀`.
    /// Each row of text displays two vertical pixel rows.
    pub fn render_halfblocks(
        img: &image::DynamicImage,
        target_width: u32,
        target_height: u32,
    ) -> Vec<Vec<(Color, Color, char)>> {
        let max_w = target_width.clamp(10, 160);
        let max_h = (target_height * 2).clamp(10, 160); // 2 pixels per char row

        let resized = img.resize(max_w, max_h, image::imageops::FilterType::Triangle);
        let rgba = resized.to_rgba8();
        let (w, h) = rgba.dimensions();

        let mut lines = Vec::new();
        let mut y = 0;
        while y < h {
            let mut line = Vec::with_capacity(w as usize);
            for x in 0..w {
                let top_pixel = rgba.get_pixel(x, y);
                let bottom_pixel = if y + 1 < h {
                    rgba.get_pixel(x, y + 1)
                } else {
                    top_pixel
                };

                let fg = Color::Rgb(top_pixel[0], top_pixel[1], top_pixel[2]);
                let bg = Color::Rgb(bottom_pixel[0], bottom_pixel[1], bottom_pixel[2]);

                line.push((fg, bg, '▀'));
            }
            lines.push(line);
            y += 2;
        }

        lines
    }
}
