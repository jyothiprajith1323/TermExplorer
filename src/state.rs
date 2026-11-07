//! Global application state.

use crate::config::Config;
use crate::filesystem::browser::{DirEntry, SortOrder};
use std::path::PathBuf;

/// Which panel currently has focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusedPanel {
    Sidebar,
    #[default]
    Explorer,
    Preview,
    Terminal,
    Editor,
    Monitor,
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
    pub lines: Vec<String>,
    pub highlighted_spans: Vec<Vec<(u8, u8, u8, String)>>,
    pub scroll_offset: usize,
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

        Self {
            config,
            mode: AppMode::default(),
            focused: FocusedPanel::default(),
            sidebar: SidebarState::default(),
            explorer: ExplorerState {
                current_dir: start_dir,
                ..Default::default()
            },
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
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>, kind: StatusKind) {
        self.status_message = Some((msg.into(), kind));
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Return the currently highlighted entry (if any).
    pub fn selected_entry(&self) -> Option<&crate::filesystem::browser::DirEntry> {
        self.explorer.entries.get(self.explorer.selected_index)
    }

    /// Return the full path of the currently highlighted entry.
    pub fn selected_path(&self) -> Option<PathBuf> {
        self.selected_entry()
            .map(|e| self.explorer.current_dir.join(&e.name))
    }
}
