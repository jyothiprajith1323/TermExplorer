//! Terminal emulator configuration guides for JetBrains Mono & Nerd Fonts.

pub struct TerminalGuide {
    pub name: &'static str,
    pub config_path: &'static str,
    pub snippet: &'static str,
    pub instructions: &'static str,
}

pub fn get_terminal_guides() -> &'static [TerminalGuide] {
    &[
        TerminalGuide {
            name: "Kitty",
            config_path: "~/.config/kitty/kitty.conf",
            snippet: "font_family      JetBrainsMono Nerd Font Mono\nbold_font        auto\nitalic_font      auto\nfont_size        12.0",
            instructions: "Add snippet to ~/.config/kitty/kitty.conf and reload with Ctrl+Shift+F5.",
        },
        TerminalGuide {
            name: "Alacritty",
            config_path: "~/.config/alacritty/alacritty.toml",
            snippet: "[font]\nsize = 12.0\n\n[font.normal]\nfamily = \"JetBrainsMono Nerd Font Mono\"\nstyle = \"Regular\"",
            instructions: "Add snippet to ~/.config/alacritty/alacritty.toml. Alacritty reloads automatically.",
        },
        TerminalGuide {
            name: "WezTerm",
            config_path: "~/.wezterm.lua",
            snippet: "local wezterm = require 'wezterm'\nlocal config = {}\nconfig.font = wezterm.font('JetBrainsMono Nerd Font Mono')\nconfig.font_size = 12.0\nreturn config",
            instructions: "Save to ~/.wezterm.lua. WezTerm reloads immediately on save.",
        },
        TerminalGuide {
            name: "GNOME Terminal",
            config_path: "Preferences -> Profiles -> Text",
            snippet: "Custom font: JetBrainsMono Nerd Font Mono 11",
            instructions: "Open Preferences -> Profiles -> Text -> check 'Custom font' -> select JetBrainsMono Nerd Font Mono.",
        },
    ]
}

/// Explanation why TUI cannot force-change the host terminal font directly.
pub const FONT_EXPLANATION: &str = "\
Terminal emulators control font rendering independently from running CLI/TUI applications. \
To render Nerd Font icons and JetBrains Mono ligatures correctly, your terminal emulator \
must be configured with 'JetBrainsMono Nerd Font Mono'. Use the guides below or run the \
built-in local installer script to setup fonts in ~/.local/share/fonts/.";
