//! Global application state.

use crate::config::Config;
use crate::filesystem::browser::{DirEntry, SortOrder};
use crate::git::GitRepoStatus;
use crate::preview::{FullPreviewData, PreviewTab};
use crate::ui::onboarding::OnboardingStep;
use crate::workspace::Workspace;
use std::path::PathBuf;

/// Which panel currently has focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusedPanel {
    Sidebar,
    #[default]
    Explorer,
    ExplorerSecondary,
    Preview,
    Terminal,
    Editor,
    Monitor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActivePane {
    #[default]
    Primary,
    Secondary,
}

/// Top-level application mode.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AppMode {
    /// Normal file-browser mode.
    #[default]
    Browse,
    /// A file is open in the integrated editor.
    Edit,
    /// A modal dialog is open (rename, delete confirm, new file, etc.).
    Modal(ModalKind),
    /// The quick-search / file filter bar is open.
    Search,
    /// The help screen is shown.
    Help,
    /// Interactive Settings Studio is shown (F10).
    SettingsStudio {
        section_idx: usize,
        item_idx: usize,
        in_items: bool,
    },
    /// Floating Command Palette is open (Ctrl+Shift+P).
    CommandPalette { query: String, selected_idx: usize },
    /// Git Dashboard is shown.
    GitDashboard { selected_idx: usize },
    /// First-launch onboarding wizard.
    Onboarding {
        step: OnboardingStep,
        selected_option: usize,
    },
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalKind {
    Rename,
    Delete,
    NewFile,
    NewDir,
    PinFolder,
    GoToLine,
    Message(String),
    Error(String),
}

/// State of the sidebar.
#[derive(Debug, Clone, Default)]
pub struct SidebarState {
    pub selected_index: usize,
    pub section: SidebarSection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SidebarSection {
    #[default]
    Pinned,
    Shortcuts,
}

/// State of the file explorer panel.
#[derive(Debug, Clone, Default)]
pub struct ExplorerState {
    pub current_dir: PathBuf,
    pub entries: Vec<DirEntry>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub sort_order: SortOrder,
    pub loading: bool,
}

/// State of the preview panel.
#[derive(Debug, Clone, Default)]
pub struct PreviewState {
    pub path: Option<PathBuf>,
    pub active_tab: PreviewTab,
    pub full_data: Option<FullPreviewData>,
    pub lines: Vec<String>,
    pub highlighted_spans: Vec<Vec<(u8, u8, u8, String)>>,
    pub scroll_offset: usize,
    pub horizontal_offset: usize,
    pub total_lines: usize,
    pub file_size: u64,
    pub mime_type: String,
    pub is_binary: bool,
}

/// Recent directory list (up to 20 entries).
#[derive(Debug, Clone, Default)]
pub struct RecentDirs {
    pub dirs: Vec<PathBuf>,
}

impl RecentDirs {
    pub fn push(&mut self, path: PathBuf) {
        self.dirs.retain(|p| p != &path);
        self.dirs.insert(0, path);
        self.dirs.truncate(20);
    }
}

/// Top-level shared state of the application.
pub struct AppState {
    pub config: Config,
    pub mode: AppMode,
    pub focused: FocusedPanel,
    pub sidebar: SidebarState,
    pub explorer: ExplorerState,
    pub preview: PreviewState,
    pub recent_dirs: RecentDirs,
    pub modal_input: String,
    pub search_query: String,
    pub status_message: Option<(String, StatusKind)>,
    pub clipboard_entry: Option<(PathBuf, bool)>,
    pub show_sidebar: bool,
    pub show_preview: bool,
    pub show_terminal: bool,
    pub show_monitor: bool,
    pub preview_maximized: bool,
    pub terminal_maximized: bool,
    pub dual_pane_enabled: bool,
    pub active_pane: ActivePane,
    pub explorer_secondary: ExplorerState,
    pub git_status: GitRepoStatus,
    pub workspace: Workspace,
    pub icon_manager: crate::icons::IconManager,
    pub font_status: crate::fonts::FontStatus,
    pub plugin_manager: crate::plugins::PluginManager,
    pub theme_registry: crate::theme::ThemeRegistry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    Info,
    Success,
    Warning,
    Error,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let show_sidebar = config.show_sidebar;
        let show_preview = config.show_preview;
        let show_terminal = config.show_terminal;
        let show_monitor = config.show_monitor;
        let start_dir = std::env::current_dir()
            .unwrap_or_else(|_| dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")));

        let workspace = Workspace::from_dir(&start_dir);
        let git_status = GitRepoStatus::query(&start_dir);

        let mut icon_manager =
            crate::icons::IconManager::new(crate::icons::IconPack::parse(&config.icon_pack));
        icon_manager.load_from_file(&crate::icons::IconManager::default_config_path());

        let font_status = crate::fonts::FontStatus::detect();

        let mut theme_registry = crate::theme::ThemeRegistry::new();
        theme_registry.load_from_dir(&crate::theme::ThemeRegistry::user_themes_dir());

        let mut plugin_manager = crate::plugins::PluginManager::new();
        plugin_manager.discover_and_load(
            &crate::plugins::PluginManager::user_plugins_dir(),
            &start_dir.to_string_lossy(),
        );
        if plugin_manager.plugins.is_empty() {
            let local_plugins = std::path::PathBuf::from("plugins");
            if local_plugins.exists() {
                plugin_manager.discover_and_load(&local_plugins, &start_dir.to_string_lossy());
            }
        }

        Self {
            config,
            mode: AppMode::default(),
            focused: FocusedPanel::default(),
            sidebar: SidebarState::default(),
            explorer: ExplorerState {
                current_dir: start_dir.clone(),
                ..Default::default()
            },
            dual_pane_enabled: false,
            active_pane: ActivePane::Primary,
            explorer_secondary: ExplorerState {
                current_dir: start_dir,
                ..Default::default()
            },
            git_status,
            workspace,
            icon_manager,
            font_status,
            plugin_manager,
            theme_registry,
            preview: PreviewState::default(),
            recent_dirs: RecentDirs::default(),
            modal_input: String::new(),
            search_query: String::new(),
            status_message: None,
            clipboard_entry: None,
            show_sidebar,
            show_preview,
            show_terminal,
            show_monitor,
            preview_maximized: false,
            terminal_maximized: false,
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>, kind: StatusKind) {
        self.status_message = Some((msg.into(), kind));
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    pub fn active_explorer(&self) -> &ExplorerState {
        if self.dual_pane_enabled && self.active_pane == ActivePane::Secondary {
            &self.explorer_secondary
        } else {
            &self.explorer
        }
    }

    pub fn active_explorer_mut(&mut self) -> &mut ExplorerState {
        if self.dual_pane_enabled && self.active_pane == ActivePane::Secondary {
            &mut self.explorer_secondary
        } else {
            &mut self.explorer
        }
    }

    /// Return the currently highlighted entry (if any).
    pub fn selected_entry(&self) -> Option<&crate::filesystem::browser::DirEntry> {
        let exp = self.active_explorer();
        exp.entries.get(exp.selected_index)
    }

    /// Return the full path of the currently highlighted entry.
    pub fn selected_path(&self) -> Option<PathBuf> {
        let exp = self.active_explorer();
        self.selected_entry().map(|e| exp.current_dir.join(&e.name))
    }
}
