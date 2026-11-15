//! Virtual Terminal Screen Emulator integrating `vt100` with Ratatui rendering.
//!
//! Provides incremental ANSI/VT stream parsing, 24-bit Truecolor & 256-color cell translation,
//! cursor tracking, alternate screen buffers, and bounded scrollback history.

use std::num::NonZeroU16;

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use vt100::Parser;

/// Bounded virtual terminal screen backed by `vt100::Parser`.
pub struct TerminalScreen {
    parser: Parser,
    rows: u16,
    cols: u16,
    scroll_offset: usize,
    scrollback_limit: usize,
}

impl TerminalScreen {
    /// Create a new terminal screen with initial dimensions and scrollback line capacity.
    pub fn new(rows: u16, cols: u16, scrollback_limit: usize) -> Self {
        let r = rows.max(1);
        let c = cols.max(1);
        let nz_r = NonZeroU16::new(r).unwrap_or(NonZeroU16::MIN);
        let nz_c = NonZeroU16::new(c).unwrap_or(NonZeroU16::MIN);
        let parser = Parser::new(nz_r, nz_c, scrollback_limit);
        Self {
            parser,
            rows: r,
            cols: c,
            scroll_offset: 0,
            scrollback_limit,
        }
    }

    /// Process raw byte chunk from the PTY master.
    pub fn process(&mut self, data: &[u8]) {
        self.parser.process(data);
    }

    /// Resize virtual screen dimensions.
    pub fn resize(&mut self, rows: u16, cols: u16) {
        let r = rows.max(1);
        let c = cols.max(1);
        if r != self.rows || c != self.cols {
            self.rows = r;
            self.cols = c;
            let nz_r = NonZeroU16::new(r).unwrap_or(NonZeroU16::MIN);
            let nz_c = NonZeroU16::new(c).unwrap_or(NonZeroU16::MIN);
            self.parser.set_size(nz_r, nz_c);
        }
    }

    /// Current terminal dimensions (rows, cols).
    pub fn size(&self) -> (u16, u16) {
        (self.rows, self.cols)
    }

    /// Current cursor position (row, col) relative to top-left (0-indexed).
    pub fn cursor_position(&self) -> (u16, u16) {
        self.parser.screen().cursor_position()
    }

    /// True if cursor visibility is disabled by the terminal application (`\x1b[?25l`).
    pub fn hide_cursor(&self) -> bool {
        self.parser.screen().hide_cursor()
    }

    /// True if application is running in the alternate screen buffer (e.g. htop, vim, less).
    pub fn is_alternate_screen(&self) -> bool {
        self.parser.screen().alternate_screen()
    }

    /// Maximum scrollback limit configured.
    pub fn scrollback_limit(&self) -> usize {
        self.scrollback_limit
    }

    /// Number of lines currently stored in scrollback history.
    pub fn scrollback_len(&self) -> usize {
        self.parser.screen().scrollback_len()
    }

    /// Current scroll offset from live bottom view (0 = live screen).
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Scroll up into historical buffer.
    pub fn scroll_up(&mut self, delta: usize) {
        let max_scroll = self.scrollback_len();
        self.scroll_offset = (self.scroll_offset + delta).min(max_scroll);
        self.parser.screen_mut().set_scrollback(self.scroll_offset);
    }

    /// Scroll down towards live screen.
    pub fn scroll_down(&mut self, delta: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(delta);
        self.parser.screen_mut().set_scrollback(self.scroll_offset);
    }

    /// Reset scroll offset to live screen.
    pub fn reset_scroll(&mut self) {
        if self.scroll_offset != 0 {
            self.scroll_offset = 0;
            self.parser.screen_mut().set_scrollback(0);
        }
    }

    /// Render a single terminal row as a Ratatui `Line` with full color & attribute styles.
    pub fn render_row(&self, row: u16, default_fg: Color, default_bg: Color) -> Line<'static> {
        let screen = self.parser.screen();
        let mut spans = Vec::new();
        let cols = self.cols;

        let mut current_text = String::new();
        let mut current_style = Style::default();

        for col in 0..cols {
            if let Some(cell) = screen.cell(row, col) {
                if cell.is_wide_continuation() {
                    continue;
                }

                let style = cell_style(cell, default_fg, default_bg);
                let content = if cell.has_contents() {
                    cell.contents()
                } else {
                    " "
                };

                if style == current_style {
                    current_text.push_str(content);
                } else {
                    if !current_text.is_empty() {
                        spans.push(Span::styled(
                            std::mem::take(&mut current_text),
                            current_style,
                        ));
                    }
                    current_style = style;
                    current_text.push_str(content);
                }
            } else {
                if !current_text.is_empty() {
                    spans.push(Span::styled(
                        std::mem::take(&mut current_text),
                        current_style,
                    ));
                }
                current_style = Style::default().fg(default_fg).bg(default_bg);
                current_text.push(' ');
            }
        }

        if !current_text.is_empty() {
            spans.push(Span::styled(current_text, current_style));
        }

        Line::from(spans)
    }

    /// Render all visible rows of the virtual terminal screen.
    pub fn render_all_rows(&self, default_fg: Color, default_bg: Color) -> Vec<Line<'static>> {
        (0..self.rows)
            .map(|r| self.render_row(r, default_fg, default_bg))
            .collect()
    }

    /// Search for matches in the current screen and scrollback buffer.
    pub fn search(&self, query: &str) -> Vec<String> {
        if query.trim().is_empty() {
            return Vec::new();
        }
        let q = query.to_lowercase();
        let mut results = Vec::new();
        let screen = self.parser.screen();

        let contents = screen.contents();
        for line in contents.lines() {
            if line.to_lowercase().contains(&q) {
                results.push(line.trim_end().to_string());
                if results.len() >= 50 {
                    break;
                }
            }
        }
        results
    }
}

/// Convert a `vt100::Color` into `ratatui::style::Color`.
pub fn vt100_color_to_ratatui(color: vt100::Color, default_color: Color) -> Color {
    match color {
        vt100::Color::Default => default_color,
        vt100::Color::Idx(n) => Color::Indexed(n),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}

/// Translate `vt100::Cell` attributes to `ratatui::style::Style`.
fn cell_style(cell: &vt100::Cell, default_fg: Color, default_bg: Color) -> Style {
    let raw_fg = vt100_color_to_ratatui(cell.fgcolor(), default_fg);
    let raw_bg = vt100_color_to_ratatui(cell.bgcolor(), default_bg);

    let (fg, bg) = if cell.inverse() {
        (raw_bg, raw_fg)
    } else {
        (raw_fg, raw_bg)
    };

    let mut style = Style::default().fg(fg).bg(bg);
    if cell.bold() {
        style = style.add_modifier(Modifier::BOLD);
    }
    if cell.dim() {
        style = style.add_modifier(Modifier::DIM);
    }
    if cell.italic() {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if cell.underline() {
        style = style.add_modifier(Modifier::UNDERLINED);
    }

    style
}
