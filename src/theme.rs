//! Theme — colours, borders, and style helpers.

use ratatui::style::{Color, Modifier, Style};

/// A complete colour palette.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Theme {
    /// Terminal default BG (None = inherit transparency).
    pub bg: Option<Color>,
    pub fg: Color,
    pub accent: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub border: Color,
    pub border_focused: Color,
    pub status_fg: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub dir_color: Color,
    pub file_color: Color,
    pub symlink_color: Color,
    pub hidden_color: Color,
    pub size_color: Color,
    pub date_color: Color,
    pub header_fg: Color,
    pub header_bg: Option<Color>,
}

impl Theme {
    /// Professional dark theme with terminal-transparent background.
    pub fn dark() -> Self {
        Self {
            bg: None,                         // Transparent — inherit from terminal emulator.
            fg: Color::Rgb(171, 178, 191),    // #ABB2BF (One Dark)
            accent: Color::Rgb(97, 175, 239), // #61AFEF
            selection_bg: Color::Rgb(44, 62, 80),
            selection_fg: Color::Rgb(229, 229, 229),
            border: Color::Rgb(60, 60, 70),
            border_focused: Color::Rgb(97, 175, 239),
            status_fg: Color::Rgb(130, 140, 160),
            error: Color::Rgb(224, 108, 117),   // #E06C75
            warning: Color::Rgb(229, 192, 123), // #E5C07B
            success: Color::Rgb(152, 195, 121), // #98C379
            dir_color: Color::Rgb(97, 175, 239),
            file_color: Color::Rgb(171, 178, 191),
            symlink_color: Color::Rgb(86, 182, 194),
            hidden_color: Color::Rgb(100, 110, 130),
            size_color: Color::Rgb(130, 140, 160),
            date_color: Color::Rgb(130, 140, 160),
            header_fg: Color::Rgb(229, 229, 229),
            header_bg: None,
        }
    }

    /// Normal (unfocused) border style.
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    /// Focused border style.
    pub fn border_focused_style(&self) -> Style {
        Style::default().fg(self.border_focused)
    }

    /// Selected-row style.
    pub fn selection_style(&self) -> Style {
        Style::default()
            .fg(self.selection_fg)
            .bg(self.selection_bg)
            .add_modifier(Modifier::BOLD)
    }

    /// Normal text style.
    pub fn normal_style(&self) -> Style {
        Style::default().fg(self.fg)
    }

    /// Accent style.
    pub fn accent_style(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// Status bar style.
    pub fn status_style(&self) -> Style {
        Style::default().fg(self.status_fg)
    }

    /// Error style.
    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error).add_modifier(Modifier::BOLD)
    }

    /// Warning style.
    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning)
    }

    /// Success style.
    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success)
    }

    /// Header style.
    pub fn header_style(&self) -> Style {
        let mut s = Style::default()
            .fg(self.header_fg)
            .add_modifier(Modifier::BOLD);
        if let Some(bg) = self.header_bg {
            s = s.bg(bg);
        }
        s
    }
}
