//! Central Command Palette registry and fuzzy finder.

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandCategory {
    Navigation,
    Explorer,
    Editor,
    Terminal,
    Git,
    Theme,
    Layout,
    Workspace,
    Actions,
}

impl CommandCategory {
    pub fn badge(&self) -> &'static str {
        match self {
            Self::Navigation => "[Nav]",
            Self::Explorer => "[Explorer]",
            Self::Editor => "[Editor]",
            Self::Terminal => "[Terminal]",
            Self::Git => "[Git]",
            Self::Theme => "[Theme]",
            Self::Layout => "[Layout]",
            Self::Workspace => "[Workspace]",
            Self::Actions => "[Action]",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandItem {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub category: CommandCategory,
    pub shortcut: Option<&'static str>,
}

pub struct CommandRegistry;

impl CommandRegistry {
    /// Return the list of all registered developer commands.
    pub fn all_commands() -> Vec<CommandItem> {
        vec![
            // Explorer & Navigation
            CommandItem {
                id: "toggle_dual_pane",
                title: "Toggle Dual-Pane Explorer",
                description: "Split explorer into two independent side-by-side panes",
                category: CommandCategory::Layout,
                shortcut: Some("F7"),
            },
            CommandItem {
                id: "open_settings",
                title: "Open Settings Studio",
                description: "Interactive full-screen settings and customization center",
                category: CommandCategory::Navigation,
                shortcut: Some("Ctrl+, / F10"),
            },
            CommandItem {
                id: "open_help",
                title: "Open Help & Keybindings",
                description: "Show comprehensive shortcuts overlay",
                category: CommandCategory::Navigation,
                shortcut: Some("F1"),
            },
            CommandItem {
                id: "toggle_hidden",
                title: "Toggle Hidden Files",
                description: "Show or hide dotfiles in active directory",
                category: CommandCategory::Explorer,
                shortcut: Some("."),
            },
            CommandItem {
                id: "cycle_sort",
                title: "Cycle Sorting Order",
                description: "Switch between Name, Size, Modified Date, and Extension sorting",
                category: CommandCategory::Explorer,
                shortcut: Some("s"),
            },
            CommandItem {
                id: "new_file",
                title: "Create New File",
                description: "Create an empty file in current directory",
                category: CommandCategory::Explorer,
                shortcut: Some("a"),
            },
            CommandItem {
                id: "new_dir",
                title: "Create New Directory",
                description: "Create a new folder in current directory",
                category: CommandCategory::Explorer,
                shortcut: Some("A"),
            },
            CommandItem {
                id: "refresh_view",
                title: "Refresh Explorer",
                description: "Reload filesystem entries and update inotify watcher",
                category: CommandCategory::Explorer,
                shortcut: Some("F5"),
            },
            // Terminal & Monitor
            CommandItem {
                id: "toggle_terminal",
                title: "Toggle Embedded Terminal",
                description: "Open or close bottom real pseudo-terminal (PTY) shell",
                category: CommandCategory::Terminal,
                shortcut: Some("Ctrl+T"),
            },
            CommandItem {
                id: "terminal_increase_height",
                title: "Increase Terminal Height",
                description: "Expand embedded terminal panel (+2 rows)",
                category: CommandCategory::Terminal,
                shortcut: Some("} / Ctrl+Up"),
            },
            CommandItem {
                id: "terminal_decrease_height",
                title: "Decrease Terminal Height",
                description: "Shrink embedded terminal panel (-2 rows)",
                category: CommandCategory::Terminal,
                shortcut: Some("{ / Ctrl+Down"),
            },
            CommandItem {
                id: "terminal_maximize",
                title: "Maximize Embedded Terminal",
                description: "Full-height developer terminal (~80% height)",
                category: CommandCategory::Terminal,
                shortcut: Some("Alt+F6 / Ctrl+Shift+T"),
            },
            CommandItem {
                id: "terminal_minimize",
                title: "Minimize Embedded Terminal",
                description: "Reset terminal to compact minimal height (~6 rows)",
                category: CommandCategory::Terminal,
                shortcut: None,
            },
            CommandItem {
                id: "terminal_restore",
                title: "Restore Embedded Terminal",
                description: "Restore terminal panel to default compact height (8 rows)",
                category: CommandCategory::Terminal,
                shortcut: None,
            },
            CommandItem {
                id: "focus_terminal",
                title: "Focus Embedded Terminal",
                description: "Switch keyboard focus to embedded terminal shell",
                category: CommandCategory::Terminal,
                shortcut: None,
            },
            CommandItem {
                id: "open_plugins",
                title: "Open Plugin Center & Marketplace",
                description: "Browse, install, and manage extensions",
                category: CommandCategory::Actions,
                shortcut: Some("Ctrl+Shift+X"),
            },
            CommandItem {
                id: "toggle_monitor",
                title: "Toggle System Resource Monitor",
                description: "Show CPU, RAM, disk, and process telemetry dashboard",
                category: CommandCategory::Terminal,
                shortcut: Some("F6"),
            },
            CommandItem {
                id: "toggle_sidebar",
                title: "Toggle Sidebar",
                description: "Show or hide left navigation sidebar",
                category: CommandCategory::Layout,
                shortcut: Some("Ctrl+B"),
            },
            CommandItem {
                id: "toggle_preview",
                title: "Toggle Preview Panel",
                description: "Show or hide right universal multimedia preview panel",
                category: CommandCategory::Layout,
                shortcut: Some("F3"),
            },
            // Themes
            CommandItem {
                id: "theme_catppuccin_mocha",
                title: "Switch Theme: Catppuccin Mocha",
                description: "Flagship soothing pastel dark aesthetic",
                category: CommandCategory::Theme,
                shortcut: None,
            },
            CommandItem {
                id: "theme_catppuccin_macchiato",
                title: "Switch Theme: Catppuccin Macchiato",
                description: "Medium-contrast pastel dark palette",
                category: CommandCategory::Theme,
                shortcut: None,
            },
            CommandItem {
                id: "theme_catppuccin_frappe",
                title: "Switch Theme: Catppuccin Frappé",
                description: "Low-contrast soothing pastel muted dark palette",
                category: CommandCategory::Theme,
                shortcut: None,
            },
            CommandItem {
                id: "theme_catppuccin_latte",
                title: "Switch Theme: Catppuccin Latte",
                description: "Crisp daylight soothing pastel light palette",
                category: CommandCategory::Theme,
                shortcut: None,
            },
            CommandItem {
                id: "icons_catppuccin",
                title: "Switch Icons: Catppuccin Nerd Font",
                description: "Nerd Font v3 glyphs with canonical Catppuccin color associations",
                category: CommandCategory::Theme,
                shortcut: None,
            },
            CommandItem {
                id: "icons_nerd_font",
                title: "Switch Icons: Monochrome Nerd Font",
                description: "Nerd Font v3 glyphs with clean monochrome palette",
                category: CommandCategory::Theme,
                shortcut: None,
            },
            CommandItem {
                id: "icons_unicode",
                title: "Switch Icons: Unicode Emojis",
                description: "Standard Unicode symbols and emoji fallback",
                category: CommandCategory::Theme,
                shortcut: None,
            },
            CommandItem {
                id: "icons_ascii",
                title: "Switch Icons: ASCII Text Tags",
                description: "Pure ASCII bracketed tags ([DIR], [FILE], [RS], etc.)",
                category: CommandCategory::Theme,
                shortcut: None,
            },
            CommandItem {
                id: "icons_none",
                title: "Switch Icons: Disable Icons",
                description: "Hide all icon prefixes for minimal file display",
                category: CommandCategory::Theme,
                shortcut: None,
            },
            CommandItem {
                id: "fonts_setup_guide",
                title: "Typography: JetBrains Mono Setup Guide",
                description: "Inspect font detection status and view terminal emulator configuration instructions",
                category: CommandCategory::Workspace,
                shortcut: None,
            },
            CommandItem {
                id: "plugins_reload",
                title: "Plugins: Discover & Reload Plugins",
                description: "Rescan ~/.local/share/termexplorer/plugins/ and reload sandboxed wasm modules",
                category: CommandCategory::Actions,
                shortcut: None,
            },
            CommandItem {
                id: "theme_dracula",
                title: "Switch Theme: Dracula",
                description: "Classic dark theme with vibrant purples & pinks",
                category: CommandCategory::Theme,
                shortcut: None,
            },
            CommandItem {
                id: "theme_nord",
                title: "Switch Theme: Nord",
                description: "Arctic north-bluish clean design",
                category: CommandCategory::Theme,
                shortcut: None,
            },
            CommandItem {
                id: "theme_gruvbox",
                title: "Switch Theme: Gruvbox",
                description: "Retro warm amber groove palette",
                category: CommandCategory::Theme,
                shortcut: None,
            },
            CommandItem {
                id: "theme_tokyo_night",
                title: "Switch Theme: Tokyo Night",
                description: "Dark palette inspired by Tokyo neon lights",
                category: CommandCategory::Theme,
                shortcut: None,
            },
            CommandItem {
                id: "theme_one_dark",
                title: "Switch Theme: One Dark",
                description: "Atom/VSCode balanced programmer theme",
                category: CommandCategory::Theme,
                shortcut: None,
            },
            CommandItem {
                id: "theme_solarized_dark",
                title: "Switch Theme: Solarized Dark",
                description: "Ethan Schoonover's calibrated palette",
                category: CommandCategory::Theme,
                shortcut: None,
            },
            CommandItem {
                id: "theme_solarized_light",
                title: "Switch Theme: Solarized Light",
                description: "Ethan Schoonover's daylight palette",
                category: CommandCategory::Theme,
                shortcut: None,
            },
            CommandItem {
                id: "theme_monokai",
                title: "Switch Theme: Monokai",
                description: "High contrast dark theme with bright accents",
                category: CommandCategory::Theme,
                shortcut: None,
            },
            CommandItem {
                id: "theme_default_dark",
                title: "Switch Theme: Default Dark",
                description: "Transparent-friendly standard dark theme",
                category: CommandCategory::Theme,
                shortcut: None,
            },
            CommandItem {
                id: "theme_default_light",
                title: "Switch Theme: Default Light",
                description: "Clean daylight theme",
                category: CommandCategory::Theme,
                shortcut: None,
            },
            // Profiles
            CommandItem {
                id: "profile_developer",
                title: "Apply Profile: Developer",
                description: "Layout with sidebar, explorer, universal preview, and terminal",
                category: CommandCategory::Layout,
                shortcut: None,
            },
            CommandItem {
                id: "profile_minimal",
                title: "Apply Profile: Minimal",
                description: "Distraction-free file manager with maximum horizontal width",
                category: CommandCategory::Layout,
                shortcut: None,
            },
            CommandItem {
                id: "profile_terminal",
                title: "Apply Profile: Terminal Power User",
                description: "Large bottom shell session with quick explorer",
                category: CommandCategory::Layout,
                shortcut: None,
            },
            CommandItem {
                id: "profile_monitor",
                title: "Apply Profile: System Monitoring",
                description: "Resource monitor dashboard focused view",
                category: CommandCategory::Layout,
                shortcut: None,
            },
            // Git
            CommandItem {
                id: "git_panel",
                title: "Open Git Dashboard",
                description: "Inspect branches, staged files, diffs, and recent commits",
                category: CommandCategory::Git,
                shortcut: Some("g"),
            },
            CommandItem {
                id: "git_status",
                title: "Git Status Check",
                description: "Inspect repository dirty status in active directory",
                category: CommandCategory::Git,
                shortcut: None,
            },
            // Custom Developer Actions
            CommandItem {
                id: "action_vscode",
                title: "Open in VS Code (code .)",
                description: "Launch Visual Studio Code in current directory",
                category: CommandCategory::Actions,
                shortcut: None,
            },
            CommandItem {
                id: "action_cargo_test",
                title: "Run: cargo test",
                description: "Run test suite in current Rust project",
                category: CommandCategory::Actions,
                shortcut: None,
            },
            CommandItem {
                id: "action_cargo_build",
                title: "Run: cargo build",
                description: "Compile current Rust project",
                category: CommandCategory::Actions,
                shortcut: None,
            },
            CommandItem {
                id: "action_open_terminal",
                title: "Open External Terminal (x-terminal-emulator)",
                description: "Spawn a new independent terminal window",
                category: CommandCategory::Actions,
                shortcut: None,
            },
        ]
    }

    /// Perform fuzzy search across all commands.
    pub fn search(query: &str) -> Vec<CommandItem> {
        let all = Self::all_commands();
        if query.trim().is_empty() {
            return all;
        }

        let matcher = SkimMatcherV2::default();
        let mut matches: Vec<(i64, CommandItem)> = all
            .into_iter()
            .filter_map(|cmd| {
                let target = format!("{} {} {}", cmd.title, cmd.description, cmd.category.badge());
                let score = matcher.fuzzy_match(&target, query)?;
                Some((score, cmd))
            })
            .collect();

        matches.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
        matches.into_iter().map(|(_, cmd)| cmd).collect()
    }

    /// Perform contextual ranking across commands and dynamic pinned/recent locations.
    pub fn search_with_context(
        query: &str,
        pinned: &[crate::config::PinnedFolder],
        recent: &[std::path::PathBuf],
        focused: crate::state::FocusedPanel,
    ) -> Vec<CommandItem> {
        let suggestions =
            crate::completion::palette::rank_palette_commands(query, pinned, recent, focused);
        suggestions
            .into_iter()
            .map(|s| s.to_command_item())
            .collect()
    }
}
