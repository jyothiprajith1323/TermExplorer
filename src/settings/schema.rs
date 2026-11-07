//! Settings Studio schema: 15 comprehensive sections with typed fields and descriptions.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    General,
    Appearance,
    Layout,
    Keybindings,
    FileExplorer,
    Preview,
    Editor,
    Terminal,
    SystemMonitor,
    Git,
    Workspaces,
    Integrations,
    CustomActions,
    Profiles,
    Advanced,
}

impl SettingsSection {
    pub const ALL: &'static [SettingsSection] = &[
        SettingsSection::General,
        SettingsSection::Appearance,
        SettingsSection::Layout,
        SettingsSection::Keybindings,
        SettingsSection::FileExplorer,
        SettingsSection::Preview,
        SettingsSection::Editor,
        SettingsSection::Terminal,
        SettingsSection::SystemMonitor,
        SettingsSection::Git,
        SettingsSection::Workspaces,
        SettingsSection::Integrations,
        SettingsSection::CustomActions,
        SettingsSection::Profiles,
        SettingsSection::Advanced,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            Self::General => "1. General",
            Self::Appearance => "2. Appearance & Themes",
            Self::Layout => "3. Layout & Panels",
            Self::Keybindings => "4. Keybinding Manager",
            Self::FileExplorer => "5. File Explorer",
            Self::Preview => "6. Universal Preview",
            Self::Editor => "7. Integrated Editor & LSP",
            Self::Terminal => "8. Embedded Terminal (PTY)",
            Self::SystemMonitor => "9. System Resource Monitor",
            Self::Git => "10. Git Integration",
            Self::Workspaces => "11. Workspaces & Projects",
            Self::Integrations => "12. External Integrations",
            Self::CustomActions => "13. Custom Developer Actions",
            Self::Profiles => "14. Configuration Profiles",
            Self::Advanced => "15. Advanced & Diagnostics",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::General => "󰒓",
            Self::Appearance => "󰔎",
            Self::Layout => "󰕰",
            Self::Keybindings => "󰌌",
            Self::FileExplorer => "󰉓",
            Self::Preview => "󰍉",
            Self::Editor => "󰅩",
            Self::Terminal => "󰞷",
            Self::SystemMonitor => "󰨇",
            Self::Git => "󰊢",
            Self::Workspaces => "󰲋",
            Self::Integrations => "󰘦",
            Self::CustomActions => "󰜎",
            Self::Profiles => "󰀯",
            Self::Advanced => "󰘳",
        }
    }
}

#[derive(Debug, Clone)]
pub enum SettingValue {
    Bool(bool),
    Choice {
        current: usize,
        options: Vec<String>,
    },
    Number {
        value: i64,
        min: i64,
        max: i64,
        step: i64,
    },
    Text(String),
    ReadOnly(String),
}

#[derive(Debug, Clone)]
pub struct SettingItem {
    pub key: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub value: SettingValue,
}
