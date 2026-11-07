//! Settings Studio schema: 14 dedicated sections with typed fields and descriptions.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    General,
    Appearance,
    Themes,
    Fonts,
    FileIcons,
    Layout,
    Keybindings,
    FileExplorer,
    Preview,
    Editor,
    Terminal,
    SystemMonitor,
    Plugins,
    Advanced,
}

impl SettingsSection {
    pub const ALL: &'static [SettingsSection] = &[
        SettingsSection::General,
        SettingsSection::Appearance,
        SettingsSection::Themes,
        SettingsSection::Fonts,
        SettingsSection::FileIcons,
        SettingsSection::Layout,
        SettingsSection::Keybindings,
        SettingsSection::FileExplorer,
        SettingsSection::Preview,
        SettingsSection::Editor,
        SettingsSection::Terminal,
        SettingsSection::SystemMonitor,
        SettingsSection::Plugins,
        SettingsSection::Advanced,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            Self::General => "1. General",
            Self::Appearance => "2. Appearance & UI",
            Self::Themes => "3. Themes (Catppuccin)",
            Self::Fonts => "4. Fonts & Typography",
            Self::FileIcons => "5. File & Folder Icons",
            Self::Layout => "6. Layout & Workstation",
            Self::Keybindings => "7. Keybinding Manager",
            Self::FileExplorer => "8. File Explorer",
            Self::Preview => "9. Universal Preview",
            Self::Editor => "10. Integrated Editor & LSP",
            Self::Terminal => "11. Embedded Terminal (PTY)",
            Self::SystemMonitor => "12. System Resource Monitor",
            Self::Plugins => "13. Plugin Center & Extensions",
            Self::Advanced => "14. Advanced & Diagnostics",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::General => "󰒓",
            Self::Appearance => "󰔎",
            Self::Themes => "󰏘",
            Self::Fonts => "󰛖",
            Self::FileIcons => "󰉓",
            Self::Layout => "󰕰",
            Self::Keybindings => "󰌌",
            Self::FileExplorer => "󰝰",
            Self::Preview => "󰍉",
            Self::Editor => "󰅩",
            Self::Terminal => "󰞷",
            Self::SystemMonitor => "󰨇",
            Self::Plugins => "◇",
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
