//! Profiles: pre-configured workspace arrangements and layouts.

use crate::config::Config;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileKind {
    Default,
    Minimal,
    Developer,
    TerminalPowerUser,
    EditorFocus,
    Monitoring,
    Custom(String),
}

impl ProfileKind {
    pub const ALL: &'static [ProfileKind] = &[
        ProfileKind::Default,
        ProfileKind::Developer,
        ProfileKind::Minimal,
        ProfileKind::TerminalPowerUser,
        ProfileKind::EditorFocus,
        ProfileKind::Monitoring,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Developer => "Developer (Full Suite)",
            Self::Minimal => "Minimal (Fast & Clean)",
            Self::TerminalPowerUser => "Terminal Power User",
            Self::EditorFocus => "Editor Focus",
            Self::Monitoring => "System Monitoring Dashboard",
            Self::Custom(_) => "Custom Profile",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Default => "Balanced 3-column layout with sidebar, explorer, and preview",
            Self::Developer => "Sidebar, explorer, universal preview, and bottom terminal active",
            Self::Minimal => "Distraction-free file browsing with maximum horizontal space",
            Self::TerminalPowerUser => "Large bottom terminal panel with quick file selection",
            Self::EditorFocus => "Maximized editor and preview panels for code writing",
            Self::Monitoring => "System resource dashboard with CPU/RAM telemetry active",
            Self::Custom(_) => "User-customized profile",
        }
    }

    /// Apply profile presets onto an existing config.
    pub fn apply_to_config(&self, cfg: &mut Config) {
        match self {
            Self::Default => {
                cfg.show_sidebar = true;
                cfg.show_preview = true;
                cfg.show_terminal = false;
                cfg.show_monitor = false;
                cfg.sidebar_width = 20;
                cfg.preview_width = 35;
            }
            Self::Developer => {
                cfg.show_sidebar = true;
                cfg.show_preview = true;
                cfg.show_terminal = true;
                cfg.show_monitor = false;
                cfg.sidebar_width = 18;
                cfg.preview_width = 35;
                cfg.terminal_height = 14;
            }
            Self::Minimal => {
                cfg.show_sidebar = false;
                cfg.show_preview = true;
                cfg.show_terminal = false;
                cfg.show_monitor = false;
                cfg.preview_width = 35;
            }
            Self::TerminalPowerUser => {
                cfg.show_sidebar = false;
                cfg.show_preview = false;
                cfg.show_terminal = true;
                cfg.show_monitor = false;
                cfg.terminal_height = 24;
            }
            Self::EditorFocus => {
                cfg.show_sidebar = false;
                cfg.show_preview = true;
                cfg.show_terminal = false;
                cfg.show_monitor = false;
                cfg.preview_width = 50;
            }
            Self::Monitoring => {
                cfg.show_sidebar = true;
                cfg.show_preview = false;
                cfg.show_terminal = false;
                cfg.show_monitor = true;
                cfg.monitor_height = 18;
            }
            Self::Custom(_) => {}
        }
    }
}
