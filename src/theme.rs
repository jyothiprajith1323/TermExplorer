//! Theme — colours, borders, and 12 built-in designer color schemes.

use ratatui::style::{Color, Modifier, Style};

/// A complete colour palette.
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: &'static str,
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
    /// Return theme matching name, with transparent background toggle.
    pub fn from_name(name: &str, transparent: bool) -> Self {
        let mut t = match name.to_ascii_lowercase().as_str() {
            "catppuccin mocha" | "mocha" => Self::catppuccin_mocha(),
            "catppuccin macchiato" | "macchiato" => Self::catppuccin_macchiato(),
            "dracula" => Self::dracula(),
            "nord" => Self::nord(),
            "gruvbox" => Self::gruvbox(),
            "tokyo night" | "tokyonight" => Self::tokyo_night(),
            "one dark" | "onedark" => Self::one_dark(),
            "solarized dark" => Self::solarized_dark(),
            "solarized light" => Self::solarized_light(),
            "monokai" => Self::monokai(),
            "default light" | "light" => Self::default_light(),
            _ => Self::default_dark(),
        };

        if transparent {
            t.bg = None;
            t.header_bg = None;
        }
        t
    }

    /// List all 12 built-in themes with descriptions.
    pub fn list_themes() -> &'static [(&'static str, &'static str)] {
        &[
            (
                "Default Dark",
                "Modern sleek dark theme with One Dark accents",
            ),
            ("Default Light", "Clean readable daylight theme"),
            (
                "Catppuccin Mocha",
                "Soothing pastel dark theme (Mocha palette)",
            ),
            (
                "Catppuccin Macchiato",
                "Medium-contrast pastel dark palette",
            ),
            (
                "Dracula",
                "Famous vampire dark theme with vibrant purples & pinks",
            ),
            ("Nord", "Arctic, north-bluish clean and elegant palette"),
            ("Gruvbox", "Retro groove warm brown and amber palette"),
            (
                "Tokyo Night",
                "Celebrated dark aesthetic inspired by Tokyo neon lights",
            ),
            ("One Dark", "Iconic Atom/VSCode balanced programmer theme"),
            (
                "Solarized Dark",
                "Ethan Schoonover's precision-calibrated dark palette",
            ),
            (
                "Solarized Light",
                "Ethan Schoonover's low-contrast daylight palette",
            ),
            (
                "Monokai",
                "Classic high-contrast dark theme with bright accents",
            ),
        ]
    }

    pub fn dark() -> Self {
        Self::default_dark()
    }

    pub fn default_dark() -> Self {
        Self {
            name: "Default Dark",
            bg: None,
            fg: Color::Rgb(210, 218, 230),
            accent: Color::Rgb(0, 215, 255),
            selection_bg: Color::Rgb(36, 48, 68),
            selection_fg: Color::Rgb(255, 255, 255),
            border: Color::Rgb(50, 56, 68),
            border_focused: Color::Rgb(0, 215, 255),
            status_fg: Color::Rgb(140, 150, 170),
            error: Color::Rgb(255, 107, 107),
            warning: Color::Rgb(255, 190, 70),
            success: Color::Rgb(80, 220, 140),
            dir_color: Color::Rgb(90, 180, 255),
            file_color: Color::Rgb(215, 222, 235),
            symlink_color: Color::Rgb(80, 210, 200),
            hidden_color: Color::Rgb(95, 105, 125),
            size_color: Color::Rgb(140, 150, 170),
            date_color: Color::Rgb(120, 130, 150),
            header_fg: Color::Rgb(240, 245, 255),
            header_bg: None,
        }
    }

    pub fn default_light() -> Self {
        Self {
            name: "Default Light",
            bg: Some(Color::Rgb(250, 250, 250)),
            fg: Color::Rgb(56, 58, 66),
            accent: Color::Rgb(64, 120, 242),
            selection_bg: Color::Rgb(229, 235, 245),
            selection_fg: Color::Rgb(30, 30, 30),
            border: Color::Rgb(200, 205, 215),
            border_focused: Color::Rgb(64, 120, 242),
            status_fg: Color::Rgb(100, 105, 115),
            error: Color::Rgb(228, 86, 73),
            warning: Color::Rgb(193, 132, 1),
            success: Color::Rgb(80, 161, 79),
            dir_color: Color::Rgb(64, 120, 242),
            file_color: Color::Rgb(56, 58, 66),
            symlink_color: Color::Rgb(1, 132, 188),
            hidden_color: Color::Rgb(160, 161, 167),
            size_color: Color::Rgb(100, 105, 115),
            date_color: Color::Rgb(100, 105, 115),
            header_fg: Color::Rgb(20, 20, 20),
            header_bg: Some(Color::Rgb(235, 238, 242)),
        }
    }

    pub fn catppuccin_mocha() -> Self {
        Self {
            name: "Catppuccin Mocha",
            bg: Some(Color::Rgb(30, 30, 46)),
            fg: Color::Rgb(205, 214, 244),
            accent: Color::Rgb(137, 180, 250),
            selection_bg: Color::Rgb(88, 91, 112),
            selection_fg: Color::Rgb(245, 224, 220),
            border: Color::Rgb(69, 71, 90),
            border_focused: Color::Rgb(203, 166, 247),
            status_fg: Color::Rgb(166, 173, 200),
            error: Color::Rgb(243, 139, 168),
            warning: Color::Rgb(249, 226, 175),
            success: Color::Rgb(166, 227, 161),
            dir_color: Color::Rgb(137, 180, 250),
            file_color: Color::Rgb(205, 214, 244),
            symlink_color: Color::Rgb(148, 226, 213),
            hidden_color: Color::Rgb(108, 112, 134),
            size_color: Color::Rgb(166, 173, 200),
            date_color: Color::Rgb(166, 173, 200),
            header_fg: Color::Rgb(245, 224, 220),
            header_bg: None,
        }
    }

    pub fn catppuccin_macchiato() -> Self {
        Self {
            name: "Catppuccin Macchiato",
            bg: Some(Color::Rgb(36, 39, 58)),
            fg: Color::Rgb(202, 211, 245),
            accent: Color::Rgb(138, 173, 244),
            selection_bg: Color::Rgb(91, 96, 120),
            selection_fg: Color::Rgb(244, 219, 214),
            border: Color::Rgb(73, 77, 100),
            border_focused: Color::Rgb(198, 160, 246),
            status_fg: Color::Rgb(165, 173, 203),
            error: Color::Rgb(237, 135, 150),
            warning: Color::Rgb(238, 212, 159),
            success: Color::Rgb(166, 218, 149),
            dir_color: Color::Rgb(138, 173, 244),
            file_color: Color::Rgb(202, 211, 245),
            symlink_color: Color::Rgb(139, 213, 202),
            hidden_color: Color::Rgb(110, 115, 141),
            size_color: Color::Rgb(165, 173, 203),
            date_color: Color::Rgb(165, 173, 203),
            header_fg: Color::Rgb(244, 219, 214),
            header_bg: None,
        }
    }

    pub fn dracula() -> Self {
        Self {
            name: "Dracula",
            bg: Some(Color::Rgb(40, 42, 54)),
            fg: Color::Rgb(248, 248, 242),
            accent: Color::Rgb(189, 147, 249),
            selection_bg: Color::Rgb(68, 71, 90),
            selection_fg: Color::Rgb(255, 121, 198),
            border: Color::Rgb(68, 71, 90),
            border_focused: Color::Rgb(189, 147, 249),
            status_fg: Color::Rgb(98, 114, 164),
            error: Color::Rgb(255, 85, 85),
            warning: Color::Rgb(241, 250, 140),
            success: Color::Rgb(80, 250, 123),
            dir_color: Color::Rgb(139, 233, 253),
            file_color: Color::Rgb(248, 248, 242),
            symlink_color: Color::Rgb(255, 184, 108),
            hidden_color: Color::Rgb(98, 114, 164),
            size_color: Color::Rgb(98, 114, 164),
            date_color: Color::Rgb(98, 114, 164),
            header_fg: Color::Rgb(255, 121, 198),
            header_bg: None,
        }
    }

    pub fn nord() -> Self {
        Self {
            name: "Nord",
            bg: Some(Color::Rgb(46, 52, 64)),
            fg: Color::Rgb(236, 239, 244),
            accent: Color::Rgb(136, 192, 208),
            selection_bg: Color::Rgb(67, 76, 94),
            selection_fg: Color::Rgb(236, 239, 244),
            border: Color::Rgb(76, 86, 106),
            border_focused: Color::Rgb(129, 161, 193),
            status_fg: Color::Rgb(216, 222, 233),
            error: Color::Rgb(191, 97, 106),
            warning: Color::Rgb(235, 203, 139),
            success: Color::Rgb(163, 190, 140),
            dir_color: Color::Rgb(136, 192, 208),
            file_color: Color::Rgb(236, 239, 244),
            symlink_color: Color::Rgb(143, 188, 187),
            hidden_color: Color::Rgb(76, 86, 106),
            size_color: Color::Rgb(216, 222, 233),
            date_color: Color::Rgb(216, 222, 233),
            header_fg: Color::Rgb(236, 239, 244),
            header_bg: None,
        }
    }

    pub fn gruvbox() -> Self {
        Self {
            name: "Gruvbox",
            bg: Some(Color::Rgb(40, 40, 40)),
            fg: Color::Rgb(235, 219, 178),
            accent: Color::Rgb(254, 128, 25),
            selection_bg: Color::Rgb(80, 73, 69),
            selection_fg: Color::Rgb(251, 241, 199),
            border: Color::Rgb(60, 56, 54),
            border_focused: Color::Rgb(215, 153, 33),
            status_fg: Color::Rgb(168, 153, 132),
            error: Color::Rgb(204, 36, 29),
            warning: Color::Rgb(215, 153, 33),
            success: Color::Rgb(152, 151, 26),
            dir_color: Color::Rgb(131, 165, 152),
            file_color: Color::Rgb(235, 219, 178),
            symlink_color: Color::Rgb(104, 157, 106),
            hidden_color: Color::Rgb(146, 131, 116),
            size_color: Color::Rgb(168, 153, 132),
            date_color: Color::Rgb(168, 153, 132),
            header_fg: Color::Rgb(251, 241, 199),
            header_bg: None,
        }
    }

    pub fn tokyo_night() -> Self {
        Self {
            name: "Tokyo Night",
            bg: Some(Color::Rgb(26, 27, 38)),
            fg: Color::Rgb(169, 177, 214),
            accent: Color::Rgb(122, 162, 247),
            selection_bg: Color::Rgb(40, 52, 80),
            selection_fg: Color::Rgb(192, 202, 245),
            border: Color::Rgb(41, 46, 66),
            border_focused: Color::Rgb(187, 154, 247),
            status_fg: Color::Rgb(86, 95, 137),
            error: Color::Rgb(247, 118, 142),
            warning: Color::Rgb(224, 175, 104),
            success: Color::Rgb(158, 206, 106),
            dir_color: Color::Rgb(122, 162, 247),
            file_color: Color::Rgb(169, 177, 214),
            symlink_color: Color::Rgb(115, 218, 202),
            hidden_color: Color::Rgb(65, 72, 104),
            size_color: Color::Rgb(86, 95, 137),
            date_color: Color::Rgb(86, 95, 137),
            header_fg: Color::Rgb(192, 202, 245),
            header_bg: None,
        }
    }

    pub fn one_dark() -> Self {
        Self::default_dark()
    }

    pub fn solarized_dark() -> Self {
        Self {
            name: "Solarized Dark",
            bg: Some(Color::Rgb(0, 43, 54)),
            fg: Color::Rgb(131, 148, 150),
            accent: Color::Rgb(38, 139, 210),
            selection_bg: Color::Rgb(7, 54, 66),
            selection_fg: Color::Rgb(147, 161, 161),
            border: Color::Rgb(7, 54, 66),
            border_focused: Color::Rgb(42, 161, 152),
            status_fg: Color::Rgb(88, 110, 117),
            error: Color::Rgb(220, 50, 47),
            warning: Color::Rgb(181, 137, 0),
            success: Color::Rgb(133, 153, 0),
            dir_color: Color::Rgb(38, 139, 210),
            file_color: Color::Rgb(131, 148, 150),
            symlink_color: Color::Rgb(42, 161, 152),
            hidden_color: Color::Rgb(88, 110, 117),
            size_color: Color::Rgb(88, 110, 117),
            date_color: Color::Rgb(88, 110, 117),
            header_fg: Color::Rgb(147, 161, 161),
            header_bg: None,
        }
    }

    pub fn solarized_light() -> Self {
        Self {
            name: "Solarized Light",
            bg: Some(Color::Rgb(253, 246, 227)),
            fg: Color::Rgb(101, 123, 131),
            accent: Color::Rgb(38, 139, 210),
            selection_bg: Color::Rgb(238, 232, 213),
            selection_fg: Color::Rgb(7, 54, 66),
            border: Color::Rgb(215, 207, 187),
            border_focused: Color::Rgb(38, 139, 210),
            status_fg: Color::Rgb(147, 161, 161),
            error: Color::Rgb(220, 50, 47),
            warning: Color::Rgb(181, 137, 0),
            success: Color::Rgb(133, 153, 0),
            dir_color: Color::Rgb(38, 139, 210),
            file_color: Color::Rgb(101, 123, 131),
            symlink_color: Color::Rgb(42, 161, 152),
            hidden_color: Color::Rgb(180, 190, 190),
            size_color: Color::Rgb(147, 161, 161),
            date_color: Color::Rgb(147, 161, 161),
            header_fg: Color::Rgb(7, 54, 66),
            header_bg: Some(Color::Rgb(238, 232, 213)),
        }
    }

    pub fn monokai() -> Self {
        Self {
            name: "Monokai",
            bg: Some(Color::Rgb(39, 40, 34)),
            fg: Color::Rgb(248, 248, 242),
            accent: Color::Rgb(102, 217, 239),
            selection_bg: Color::Rgb(73, 72, 62),
            selection_fg: Color::Rgb(249, 38, 114),
            border: Color::Rgb(60, 60, 50),
            border_focused: Color::Rgb(166, 226, 46),
            status_fg: Color::Rgb(117, 113, 94),
            error: Color::Rgb(249, 38, 114),
            warning: Color::Rgb(253, 151, 31),
            success: Color::Rgb(166, 226, 46),
            dir_color: Color::Rgb(102, 217, 239),
            file_color: Color::Rgb(248, 248, 242),
            symlink_color: Color::Rgb(174, 129, 255),
            hidden_color: Color::Rgb(117, 113, 94),
            size_color: Color::Rgb(117, 113, 94),
            date_color: Color::Rgb(117, 113, 94),
            header_fg: Color::Rgb(248, 248, 242),
            header_bg: None,
        }
    }

    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    pub fn border_focused_style(&self) -> Style {
        Style::default().fg(self.border_focused)
    }

    pub fn selection_style(&self) -> Style {
        Style::default()
            .fg(self.selection_fg)
            .bg(self.selection_bg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn normal_style(&self) -> Style {
        Style::default().fg(self.fg)
    }

    pub fn accent_style(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn status_style(&self) -> Style {
        Style::default().fg(self.status_fg)
    }

    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error).add_modifier(Modifier::BOLD)
    }

    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning)
    }

    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success)
    }

    pub fn header_style(&self) -> Style {
        let mut s = Style::default()
            .fg(self.header_fg)
            .add_modifier(Modifier::BOLD);
        if let Some(bg) = self.header_bg {
            s = s.bg(bg);
        }
        s
    }

    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.hidden_color)
    }

    pub fn pill_style(&self, fg: Color, bg: Color) -> Style {
        Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD)
    }

    pub fn key_hint_style(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }
}
