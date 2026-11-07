//! Core application struct, event loop, and input handling.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use crate::{
    commands::CommandRegistry,
    config::Config,
    editor::{auto_pair, buffer::Buffer},
    events::{start_event_loop, AppEvent},
    filesystem::{browser::list_dir, operations},
    git::GitRepoStatus,
    monitor::{Monitor, MonitorSnapshot},
    preview::{PreviewManager, PreviewTab},
    settings::{get_section_items, ProfileKind, SettingsPersistence, SettingsSection},
    shell::{input::key_to_pty_bytes, TerminalSession},
    state::{ActivePane, AppMode, AppState, FocusedPanel, ModalKind, SidebarSection, StatusKind},
    theme::Theme,
    ui::{
        command_palette::render_command_palette,
        editor::render_editor,
        file_explorer::{render_file_explorer, render_secondary_explorer},
        git_panel::render_git_panel,
        header::render_header,
        help::render_help,
        layout::{compute_layout, LayoutConfig},
        onboarding::{render_onboarding, OnboardingStep},
        preview::render_preview,
        settings::render_settings_studio,
        sidebar::render_sidebar,
        status_bar::render_status_bar,
        system_monitor::render_system_monitor,
        terminal::render_terminal,
    },
};

// ── App ───────────────────────────────────────────────────────────────────────

pub struct App {
    state: AppState,
    theme: Theme,
    preview_manager: PreviewManager,
    editor_buf: Option<Buffer>,
    terminal_session: Option<TerminalSession>,
    monitor: Monitor,
    monitor_snapshot: Option<MonitorSnapshot>,
    monitor_last_refresh: Instant,
    terminal_scrollback: Vec<u8>,
    pty_rx: Option<mpsc::UnboundedReceiver<Vec<u8>>>,
    _watcher: Option<notify::RecommendedWatcher>,
    fs_rx: Option<mpsc::UnboundedReceiver<notify::Result<notify::Event>>>,
}

impl App {
    pub async fn new(config: Config) -> Result<Self> {
        let theme = Theme::from_name(&config.theme.name, config.theme.transparent_background);
        let max_preview_bytes = config.max_preview_bytes;
        let mut state = AppState::new(config);

        // Initial directory load.
        let entries = list_dir(
            &state.explorer.current_dir.clone(),
            state.config.show_hidden,
            state.explorer.sort_order,
        )
        .unwrap_or_default();
        state.explorer.entries = entries;

        let (_watcher, fs_rx) =
            match crate::filesystem::watcher::watch_directory(&state.explorer.current_dir) {
                Ok((w, rx)) => (Some(w), Some(rx)),
                Err(_) => (None, None),
            };

        Ok(Self {
            state,
            theme,
            preview_manager: PreviewManager::new(max_preview_bytes),
            editor_buf: None,
            terminal_session: None,
            monitor: Monitor::new(),
            monitor_snapshot: None,
            monitor_last_refresh: Instant::now(),
            terminal_scrollback: Vec::new(),
            pty_rx: None,
            _watcher,
            fs_rx,
        })
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }

    // ── Main event loop ──────────────────────────────────────────────────────

    pub async fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<()> {
        let mut events = start_event_loop(100);

        loop {
            // ── Render ───────────────────────────────────────────────────────
            self.draw(terminal)?;

            // ── Process next event ───────────────────────────────────────────
            let event = tokio::select! {
                // Background PTY shell output
                Some(bytes) = async {
                    match &mut self.pty_rx {
                        Some(rx) => rx.recv().await,
                        None => std::future::pending().await,
                    }
                } => {
                    self.terminal_scrollback.extend_from_slice(&bytes);
                    if self.terminal_scrollback.len() > 1024 * 1024 {
                        let excess = self.terminal_scrollback.len() - 1024 * 1024;
                        self.terminal_scrollback.drain(0..excess);
                    }
                    if let Some(session) = &mut self.terminal_session {
                        session.push_output(&bytes);
                    }
                    continue;
                },
                // Filesystem changes
                Some(_fs) = async {
                    match &mut self.fs_rx {
                        Some(rx) => rx.recv().await,
                        None => std::future::pending().await,
                    }
                } => {
                    AppEvent::FsChange
                },
                // UI & Keyboard events
                e = events.recv() => match e {
                    Some(ev) => ev,
                    None => break,
                },
            };

            match event {
                AppEvent::Quit => break,
                AppEvent::Tick => self.on_tick().await,
                AppEvent::Resize(w, h) => self.on_resize(w, h),
                AppEvent::Key(key) => {
                    if self.handle_key(key).await? {
                        break;
                    }
                }
                AppEvent::Mouse(_) => {}
                AppEvent::FsChange => self.refresh_dir().await,
                AppEvent::MonitorRefresh => {}
                AppEvent::PtyOutput => {}
                AppEvent::ClearStatus => self.state.clear_status(),
            }
        }

        // Clean up terminal session.
        self.cleanup_terminal_session();
        Ok(())
    }

    // ── Rendering ────────────────────────────────────────────────────────────

    fn draw(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        let state = &self.state;
        let theme = &self.theme;
        let editor_buf = &self.editor_buf;
        let monitor_snap = &self.monitor_snapshot;
        let scrollback = &self.terminal_scrollback;

        terminal.draw(|frame| {
            let area = frame.area();

            // ── Minimum Terminal Size Check ───────────────────────────────
            if area.width < 40 || area.height < 8 {
                let msg = format!(
                    "\n  󰀪 Terminal window too small ({}×{}).\n  Minimum required: 40×8. Please enlarge.",
                    area.width, area.height
                );
                frame.render_widget(
                    ratatui::widgets::Paragraph::new(msg)
                        .style(ratatui::style::Style::default().fg(theme.warning)),
                    area,
                );
                return;
            }

            // ── Full-screen editor mode ───────────────────────────────────
            if matches!(state.mode, AppMode::Edit) {
                if let Some(buf) = editor_buf {
                    render_editor(frame, area, buf, state, theme);
                    return;
                }
            }

            // ── Compute layout ────────────────────────────────────────────
            let show_bottom = state.show_terminal || state.show_monitor;
            let bottom_height = if state.show_terminal {
                state.config.terminal_height + 2
            } else {
                state.config.monitor_height + 2
            };

            let layout_cfg = LayoutConfig {
                show_sidebar: state.show_sidebar,
                show_preview: state.show_preview,
                show_bottom,
                dual_pane: state.dual_pane_enabled,
                sidebar_pct: state.config.sidebar_width,
                preview_pct: state.config.preview_width,
                bottom_height,
            };

            let areas = compute_layout(area, &layout_cfg);

            // ── Header (Workstation Command Bar) ──────────────────────────
            render_header(frame, areas.header, state, monitor_snap.as_ref(), theme);

            // ── Sidebar ───────────────────────────────────────────────────
            if let Some(sb_area) = areas.sidebar {
                render_sidebar(frame, sb_area, state, theme);
            }

            // ── File explorer ─────────────────────────────────────────────
            render_file_explorer(frame, areas.explorer, state, theme);

            // ── Dual Pane Secondary Explorer ──────────────────────────────
            if let Some(sec_area) = areas.explorer_secondary {
                render_secondary_explorer(frame, sec_area, state, theme);
            }

            // ── Preview ───────────────────────────────────────────────────
            if let Some(prev_area) = areas.preview {
                render_preview(frame, prev_area, state, theme);
            }

            // ── Bottom panel ──────────────────────────────────────────────
            if let Some(bottom_area) = areas.bottom_panel {
                if state.show_terminal {
                    render_terminal(frame, bottom_area, scrollback, state, theme);
                } else if state.show_monitor {
                    if let Some(snap) = monitor_snap {
                        render_system_monitor(frame, bottom_area, snap, theme);
                    }
                }
            }

            // ── Status bar ────────────────────────────────────────────────
            render_status_bar(frame, areas.status_bar, state, theme);

            // ── Help overlay ──────────────────────────────────────────────
            if matches!(state.mode, AppMode::Help) {
                render_help(frame, area, theme);
            }

            // ── Modal dialog ──────────────────────────────────────────────
            if let AppMode::Modal(kind) = &state.mode {
                render_modal(frame, area, kind, &state.modal_input, theme);
            }

            // ── Interactive Settings Studio ───────────────────────────────
            if let AppMode::SettingsStudio {
                section_idx,
                item_idx,
                in_items,
            } = state.mode
            {
                render_settings_studio(frame, area, state, theme, section_idx, item_idx, in_items);
            }

            // ── Floating Command Palette ──────────────────────────────────
            if let AppMode::CommandPalette {
                query,
                selected_idx,
            } = &state.mode
            {
                let items = CommandRegistry::search(query);
                render_command_palette(frame, area, query, &items, *selected_idx, theme);
            }

            // ── Git Dashboard ─────────────────────────────────────────────
            if let AppMode::GitDashboard { selected_idx } = state.mode {
                render_git_panel(frame, area, &state.git_status, selected_idx, theme);
            }

            // ── Onboarding Wizard ─────────────────────────────────────────
            if let AppMode::Onboarding {
                step,
                selected_option,
            } = state.mode
            {
                render_onboarding(frame, area, step, selected_option, theme);
            }
        })?;

        Ok(())
    }

    // ── Tick (periodic) ──────────────────────────────────────────────────────

    async fn on_tick(&mut self) {
        // Refresh monitor at configured interval.
        if self.state.show_monitor {
            let interval = Duration::from_millis(self.state.config.monitor.refresh_ms);
            if self.monitor_last_refresh.elapsed() >= interval {
                self.monitor_snapshot = Some(self.monitor.refresh());
                self.monitor_last_refresh = Instant::now();
            }
        }
    }

    fn on_resize(&mut self, w: u16, _h: u16) {
        if let Some(session) = &mut self.terminal_session {
            let term_h = self.state.config.terminal_height;
            let _ = session.resize(w, term_h);
        }
    }

    fn cleanup_terminal_session(&mut self) {
        // PTY child is cleaned up on Drop.
        self.terminal_session = None;
    }

    // ── Directory loading ────────────────────────────────────────────────────

    async fn refresh_dir(&mut self) {
        let is_secondary =
            self.state.dual_pane_enabled && self.state.active_pane == ActivePane::Secondary;
        let (dir, sort) = if is_secondary {
            (
                self.state.explorer_secondary.current_dir.clone(),
                self.state.explorer_secondary.sort_order,
            )
        } else {
            (
                self.state.explorer.current_dir.clone(),
                self.state.explorer.sort_order,
            )
        };
        let show_hidden = self.state.config.show_hidden;

        match list_dir(&dir, show_hidden, sort) {
            Ok(entries) => {
                let exp = self.state.active_explorer_mut();
                exp.entries = entries;
                exp.selected_index = exp.selected_index.min(exp.entries.len().saturating_sub(1));
                self.update_preview().await;
            }
            Err(e) => {
                self.state
                    .set_status(format!("Cannot read directory: {e}"), StatusKind::Error);
            }
        }
    }

    pub async fn navigate_into(&mut self, dir: PathBuf) {
        let is_secondary =
            self.state.dual_pane_enabled && self.state.active_pane == ActivePane::Secondary;
        if is_secondary {
            self.state.explorer_secondary.current_dir = dir.clone();
            self.state.explorer_secondary.selected_index = 0;
            self.state.explorer_secondary.scroll_offset = 0;
            self.refresh_dir().await;
        } else {
            self.state
                .recent_dirs
                .push(self.state.explorer.current_dir.clone());
            self.state.explorer.current_dir = dir.clone();
            self.state.explorer.selected_index = 0;
            self.state.explorer.scroll_offset = 0;
            if let Ok((w, rx)) = crate::filesystem::watcher::watch_directory(&dir) {
                self._watcher = Some(w);
                self.fs_rx = Some(rx);
            }
            self.refresh_dir().await;
        }
        self.state
            .set_status(format!("→ {}", dir.display()), StatusKind::Info);
    }

    pub async fn navigate_up(&mut self) {
        let is_secondary =
            self.state.dual_pane_enabled && self.state.active_pane == ActivePane::Secondary;
        let current = if is_secondary {
            self.state.explorer_secondary.current_dir.clone()
        } else {
            self.state.explorer.current_dir.clone()
        };

        if let Some(parent) = current.parent().map(|p| p.to_path_buf()) {
            let prev_name = current.file_name().map(|n| n.to_string_lossy().to_string());
            self.navigate_into(parent).await;
            if let Some(name) = prev_name {
                let exp = self.state.active_explorer_mut();
                if let Some(idx) = exp.entries.iter().position(|e| e.name == name) {
                    exp.selected_index = idx;
                }
            }
            self.update_preview().await;
        } else {
            self.state
                .set_status("At filesystem root", StatusKind::Info);
        }
    }

    // ── Preview loading ──────────────────────────────────────────────────────

    async fn update_preview(&mut self) {
        let path = match self.state.selected_path() {
            Some(p) => p,
            None => {
                self.state.preview.path = None;
                self.state.preview.full_data = None;
                return;
            }
        };

        self.state.preview.path = Some(path.clone());

        if path.is_dir() {
            self.state.preview.lines = vec!["  (directory)".to_string()];
            self.state.preview.highlighted_spans.clear();
            self.state.preview.is_binary = false;
            self.state.preview.total_lines = 1;
            self.state.preview.full_data = None;
            return;
        }

        let full_data =
            self.preview_manager
                .load_file(&path, self.state.preview.active_tab, 60, 30);

        self.state.preview.file_size = full_data
            .metadata
            .as_ref()
            .map(|m| m.size_bytes)
            .unwrap_or(0);
        self.state.preview.mime_type = full_data.format_info.mime_type.clone();
        self.state.preview.is_binary = full_data.format_info.is_binary;

        if let Some(src) = &full_data.source {
            self.state.preview.lines = src.lines.clone();
            self.state.preview.highlighted_spans = src.highlighted_spans.clone();
            self.state.preview.total_lines = src.total_lines;
        } else if let Some(md) = &full_data.markdown {
            self.state.preview.lines = md.formatted_lines.clone();
            self.state.preview.highlighted_spans.clear();
            self.state.preview.total_lines = md.total_lines;
        } else {
            self.state.preview.lines.clear();
            self.state.preview.highlighted_spans.clear();
            self.state.preview.total_lines = 0;
        }

        self.state.preview.full_data = Some(full_data);
        self.state.preview.scroll_offset = 0;
    }

    // ── Terminal session ──────────────────────────────────────────────────────

    fn start_terminal_session(&mut self) {
        if self.terminal_session.is_some() {
            return;
        }

        let dir = self.state.explorer.current_dir.clone();
        let cols = 120u16;
        let rows = self.state.config.terminal_height;
        let (tx, rx) = mpsc::unbounded_channel();
        self.pty_rx = Some(rx);

        match TerminalSession::new(dir, cols, rows, tx) {
            Ok(session) => {
                self.terminal_session = Some(session);
                self.state
                    .set_status("Terminal started", StatusKind::Success);
            }
            Err(e) => {
                self.state
                    .set_status(format!("Failed to start terminal: {e}"), StatusKind::Error);
            }
        }
    }

    // ── Keyboard handling ─────────────────────────────────────────────────────

    /// Returns true if the application should quit.
    pub async fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        // ── Terminal mode: forward most keys to PTY ──────────────────────
        if self.state.focused == FocusedPanel::Terminal {
            return self.handle_terminal_key(key).await;
        }

        // ── Editor mode ───────────────────────────────────────────────────
        if matches!(self.state.mode, AppMode::Edit) {
            return self.handle_editor_key(key).await;
        }

        // ── Modal dialogs ─────────────────────────────────────────────────
        if let AppMode::Modal(_) = &self.state.mode.clone() {
            return self.handle_modal_key(key).await;
        }

        // ── Search mode ───────────────────────────────────────────────────
        if matches!(self.state.mode, AppMode::Search) {
            return self.handle_search_key(key).await;
        }

        // ── Interactive Settings Studio ───────────────────────────────────
        if matches!(self.state.mode, AppMode::SettingsStudio { .. }) {
            return self.handle_settings_key(key).await;
        }

        // ── Floating Command Palette ──────────────────────────────────────
        if matches!(self.state.mode, AppMode::CommandPalette { .. }) {
            return self.handle_command_palette_key(key).await;
        }

        // ── Git Dashboard ─────────────────────────────────────────────────
        if matches!(self.state.mode, AppMode::GitDashboard { .. }) {
            return self.handle_git_dashboard_key(key).await;
        }

        // ── Onboarding Wizard ─────────────────────────────────────────────
        if matches!(self.state.mode, AppMode::Onboarding { .. }) {
            return self.handle_onboarding_key(key).await;
        }

        // ── Global shortcuts (always active) ──────────────────────────────
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('q')) => return Ok(true),
            (KeyModifiers::CONTROL, KeyCode::Char('b')) => {
                self.state.show_sidebar = !self.state.show_sidebar;
                return Ok(false);
            }
            (KeyModifiers::CONTROL, KeyCode::Char('`'))
            | (KeyModifiers::CONTROL, KeyCode::Char('t')) => {
                self.toggle_terminal();
                return Ok(false);
            }
            (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                self.state.mode = AppMode::CommandPalette {
                    query: String::new(),
                    selected_idx: 0,
                };
                return Ok(false);
            }
            _ => {}
        }

        if key.modifiers.contains(KeyModifiers::CONTROL)
            && key.modifiers.contains(KeyModifiers::SHIFT)
            && key.code == KeyCode::Char('P')
        {
            self.state.mode = AppMode::CommandPalette {
                query: String::new(),
                selected_idx: 0,
            };
            return Ok(false);
        }

        match key.code {
            KeyCode::F(1) => {
                if matches!(self.state.mode, AppMode::Help) {
                    self.state.mode = AppMode::Browse;
                } else {
                    self.state.mode = AppMode::Help;
                }
            }
            KeyCode::F(6) => self.toggle_monitor(),
            KeyCode::F(7) => {
                self.toggle_dual_pane().await;
                return Ok(false);
            }
            KeyCode::F(10) => {
                self.state.mode = AppMode::SettingsStudio {
                    section_idx: 0,
                    item_idx: 0,
                    in_items: false,
                };
                return Ok(false);
            }
            KeyCode::Esc => {
                self.state.mode = AppMode::Browse;
                self.state.search_query.clear();
                if self.state.focused == FocusedPanel::Preview {
                    self.state.focused = if self.state.dual_pane_enabled
                        && self.state.active_pane == ActivePane::Secondary
                    {
                        FocusedPanel::ExplorerSecondary
                    } else {
                        FocusedPanel::Explorer
                    };
                }
            }
            _ => {}
        }

        // ── Help mode: Esc already handled above ──────────────────────────
        if matches!(self.state.mode, AppMode::Help) {
            return Ok(false);
        }

        // ── Focus switching with Tab / arrow keys ─────────────────────────
        match key.code {
            KeyCode::Tab => {
                if self.state.dual_pane_enabled {
                    if self.state.focused == FocusedPanel::Explorer {
                        self.state.focused = FocusedPanel::ExplorerSecondary;
                        self.state.active_pane = ActivePane::Secondary;
                    } else if self.state.focused == FocusedPanel::ExplorerSecondary {
                        self.state.focused = FocusedPanel::Explorer;
                        self.state.active_pane = ActivePane::Primary;
                    } else {
                        self.cycle_focus();
                    }
                } else {
                    self.cycle_focus();
                }
            }
            KeyCode::BackTab => self.cycle_focus_back(),
            _ => {}
        }

        // ── Panel-specific key handling ───────────────────────────────────
        match self.state.focused {
            FocusedPanel::Sidebar => {
                self.handle_sidebar_key(key).await?;
            }
            FocusedPanel::Explorer | FocusedPanel::ExplorerSecondary => {
                self.handle_explorer_key(key).await?;
            }
            FocusedPanel::Preview => {
                self.handle_preview_key(key).await?;
            }
            _ => {}
        }

        Ok(false)
    }

    async fn handle_terminal_key(&mut self, key: KeyEvent) -> Result<bool> {
        // Global exit shortcut takes priority.
        if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('q') {
            return Ok(true);
        }
        // Toggle terminal back.
        if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('`') {
            self.toggle_terminal();
            return Ok(false);
        }

        if let Some(session) = &self.terminal_session {
            let bytes = key_to_pty_bytes(key);
            if !bytes.is_empty() {
                let _ = session.send_input(&bytes);
            }
        }
        Ok(false)
    }

    async fn handle_editor_key(&mut self, key: KeyEvent) -> Result<bool> {
        // Global exit.
        if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('q') {
            return Ok(true);
        }

        let buf = match &mut self.editor_buf {
            Some(b) => b,
            None => {
                self.state.mode = AppMode::Browse;
                return Ok(false);
            }
        };

        // If completion popup is active, route navigation and acceptance keys to it.
        if buf.completion.active {
            match key.code {
                KeyCode::Down => {
                    buf.completion_next();
                    return Ok(false);
                }
                KeyCode::Up => {
                    buf.completion_prev();
                    return Ok(false);
                }
                KeyCode::Tab | KeyCode::Enter => {
                    if buf.accept_completion() {
                        return Ok(false);
                    }
                }
                KeyCode::Esc => {
                    buf.dismiss_completion();
                    return Ok(false);
                }
                _ => {}
            }
        }

        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('s')) => match buf.save() {
                Ok(_) => self.state.set_status("File saved", StatusKind::Success),
                Err(e) => self
                    .state
                    .set_status(format!("Save error: {e}"), StatusKind::Error),
            },
            (KeyModifiers::CONTROL, KeyCode::Char('z')) => buf.undo(),
            (KeyModifiers::CONTROL, KeyCode::Char('y')) => buf.redo(),
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => buf.copy_line(),
            (KeyModifiers::CONTROL, KeyCode::Char('x')) => buf.cut_line(),
            (KeyModifiers::CONTROL, KeyCode::Char('v')) => buf.paste(),
            (KeyModifiers::CONTROL, KeyCode::Char('f')) => {
                self.state.mode = AppMode::Modal(ModalKind::Message("Search: ".to_string()));
            }
            (KeyModifiers::NONE, KeyCode::Esc) => {
                if buf.dirty {
                    self.state.mode = AppMode::Modal(ModalKind::Message(
                        "Unsaved changes. Press Esc again to discard, Ctrl+S to save.".to_string(),
                    ));
                } else {
                    self.state.mode = AppMode::Browse;
                    self.state.focused = FocusedPanel::Explorer;
                }
            }
            (KeyModifiers::NONE, KeyCode::Up) => buf.move_up(),
            (KeyModifiers::NONE, KeyCode::Down) => buf.move_down(),
            (KeyModifiers::NONE, KeyCode::Left) => buf.move_left(),
            (KeyModifiers::NONE, KeyCode::Right) => buf.move_right(),
            (KeyModifiers::NONE, KeyCode::Home) => buf.move_home(),
            (KeyModifiers::NONE, KeyCode::End) => buf.move_end(),
            (KeyModifiers::NONE, KeyCode::PageUp) => buf.move_page_up(20),
            (KeyModifiers::NONE, KeyCode::PageDown) => buf.move_page_down(20),
            (KeyModifiers::NONE, KeyCode::Enter) => {
                buf.insert_newline();
                buf.dismiss_completion();
            }
            (KeyModifiers::NONE, KeyCode::Tab) => {
                buf.insert_tab(self.state.config.editor.tab_width);
            }
            (KeyModifiers::NONE, KeyCode::Backspace) => {
                if !auto_pair::handle_backspace(buf) {
                    buf.backspace();
                }
                buf.update_completion();
            }
            (KeyModifiers::NONE, KeyCode::Delete) => {
                buf.delete_char();
                buf.update_completion();
            }
            (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(c)) => {
                if self.state.config.editor.auto_pair {
                    if !auto_pair::handle_char(buf, c) {
                        buf.insert_char(c);
                    }
                } else {
                    buf.insert_char(c);
                }
                buf.update_completion();
            }
            _ => {}
        }

        // Update scroll so cursor stays visible.
        if let Some(buf) = &mut self.editor_buf {
            buf.scroll_to_cursor(40, 100);
        }

        Ok(false)
    }

    async fn handle_modal_key(&mut self, key: KeyEvent) -> Result<bool> {
        let kind = match &self.state.mode {
            AppMode::Modal(k) => k.clone(),
            _ => return Ok(false),
        };

        match key.code {
            KeyCode::Esc => {
                self.state.mode = AppMode::Browse;
                self.state.modal_input.clear();
            }
            KeyCode::Enter => {
                let input = self.state.modal_input.clone();
                self.state.modal_input.clear();
                self.state.mode = AppMode::Browse;
                self.execute_modal(kind, &input).await;
            }
            KeyCode::Backspace => {
                self.state.modal_input.pop();
            }
            KeyCode::Char(c)
                if key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT =>
            {
                self.state.modal_input.push(c);
            }
            _ => {}
        }
        Ok(false)
    }

    async fn execute_modal(&mut self, kind: ModalKind, input: &str) {
        match kind {
            ModalKind::NewFile => {
                let path = self.state.explorer.current_dir.join(input);
                match operations::create_file(&path) {
                    Ok(_) => {
                        self.state.set_status(
                            format!("Created: {}", path.display()),
                            StatusKind::Success,
                        );
                        self.refresh_dir().await;
                    }
                    Err(e) => self
                        .state
                        .set_status(format!("Error: {e}"), StatusKind::Error),
                }
            }
            ModalKind::NewDir => {
                let path = self.state.explorer.current_dir.join(input);
                match operations::create_dir(&path) {
                    Ok(_) => {
                        self.state.set_status(
                            format!("Created: {}", path.display()),
                            StatusKind::Success,
                        );
                        self.refresh_dir().await;
                    }
                    Err(e) => self
                        .state
                        .set_status(format!("Error: {e}"), StatusKind::Error),
                }
            }
            ModalKind::Rename => {
                if let Some(src) = self.state.selected_path() {
                    let dst = src.parent().unwrap_or(&src).join(input);
                    match operations::rename_entry(&src, &dst) {
                        Ok(_) => {
                            self.state
                                .set_status(format!("Renamed → {}", input), StatusKind::Success);
                            self.refresh_dir().await;
                        }
                        Err(e) => self
                            .state
                            .set_status(format!("Error: {e}"), StatusKind::Error),
                    }
                }
            }
            ModalKind::Delete => {
                if input.to_lowercase() == "y" || input.to_lowercase() == "yes" {
                    if let Some(path) = self.state.selected_path() {
                        match operations::delete_entry(&path) {
                            Ok(_) => {
                                self.state.set_status(
                                    format!("Deleted: {}", path.display()),
                                    StatusKind::Success,
                                );
                                self.refresh_dir().await;
                            }
                            Err(e) => self
                                .state
                                .set_status(format!("Error: {e}"), StatusKind::Error),
                        }
                    }
                } else {
                    self.state.set_status("Delete cancelled", StatusKind::Info);
                }
            }
            ModalKind::PinFolder => {
                let path = if input.starts_with('/') {
                    PathBuf::from(input)
                } else {
                    self.state.explorer.current_dir.join(input)
                };
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| input.to_string());
                self.state.config.pin_folder(name, path);
                let _ = self.state.config.save();
                self.state.set_status("Folder pinned", StatusKind::Success);
            }
            ModalKind::GoToLine => {
                if let (Some(buf), Ok(n)) = (&mut self.editor_buf, input.parse::<usize>()) {
                    buf.goto_line(n);
                }
            }
            ModalKind::Message(_) | ModalKind::Error(_) => {}
        }
    }

    async fn handle_search_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.state.mode = AppMode::Browse;
                self.state.search_query.clear();
            }
            KeyCode::Backspace => {
                self.state.search_query.pop();
            }
            KeyCode::Char(c)
                if key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT =>
            {
                self.state.search_query.push(c);
            }
            KeyCode::Enter => {
                self.state.mode = AppMode::Browse;
            }
            _ => {}
        }
        Ok(false)
    }

    async fn handle_explorer_key(&mut self, key: KeyEvent) -> Result<()> {
        match (key.modifiers, key.code) {
            // ── Navigation ────────────────────────────────────────────────
            (KeyModifiers::NONE, KeyCode::Up) | (KeyModifiers::NONE, KeyCode::Char('k')) => {
                let moved = {
                    let exp = self.state.active_explorer_mut();
                    if exp.selected_index > 0 {
                        exp.selected_index -= 1;
                        true
                    } else {
                        false
                    }
                };
                if moved {
                    self.update_preview().await;
                }
            }
            (KeyModifiers::NONE, KeyCode::Down) | (KeyModifiers::NONE, KeyCode::Char('j')) => {
                let max = self.state.active_explorer().entries.len().saturating_sub(1);
                let moved = {
                    let exp = self.state.active_explorer_mut();
                    if exp.selected_index < max {
                        exp.selected_index += 1;
                        true
                    } else {
                        false
                    }
                };
                if moved {
                    self.update_preview().await;
                }
            }
            (KeyModifiers::NONE, KeyCode::Enter)
            | (KeyModifiers::NONE, KeyCode::Right)
            | (KeyModifiers::NONE, KeyCode::Char('l')) => {
                if let Some(entry) = self.state.selected_entry() {
                    if entry.is_dir() {
                        let dir = self.state.active_explorer().current_dir.join(&entry.name);
                        self.navigate_into(dir).await;
                    } else {
                        // Open in preview.
                        self.state.focused = FocusedPanel::Preview;
                    }
                }
            }
            (KeyModifiers::NONE, KeyCode::Backspace)
            | (KeyModifiers::NONE, KeyCode::Left)
            | (KeyModifiers::NONE, KeyCode::Char('h')) => {
                self.navigate_up().await;
            }

            // ── File operations ───────────────────────────────────────────
            (KeyModifiers::NONE, KeyCode::F(2)) => {
                // Open editor.
                if let Some(path) = self.state.selected_path() {
                    if path.is_file() {
                        match Buffer::from_file(path) {
                            Ok(buf) => {
                                self.editor_buf = Some(buf);
                                self.state.mode = AppMode::Edit;
                                self.state.focused = FocusedPanel::Editor;
                            }
                            Err(e) => {
                                self.state
                                    .set_status(format!("Cannot open: {e}"), StatusKind::Error);
                            }
                        }
                    }
                }
            }
            (KeyModifiers::NONE, KeyCode::F(4)) => {
                self.state.mode = AppMode::Modal(ModalKind::NewFile);
                self.state.modal_input.clear();
            }
            (KeyModifiers::NONE, KeyCode::F(5)) => {
                self.refresh_dir().await;
            }
            (KeyModifiers::NONE, KeyCode::F(7)) => {
                self.state.mode = AppMode::Modal(ModalKind::NewDir);
                self.state.modal_input.clear();
            }
            (KeyModifiers::NONE, KeyCode::F(8)) => {
                if let Some(path) = self.state.selected_path() {
                    self.state.mode = AppMode::Modal(ModalKind::Delete);
                    self.state.modal_input.clear();
                    self.state.set_status(
                        format!("Delete {}? Type y to confirm.", path.display()),
                        StatusKind::Warning,
                    );
                }
            }
            (KeyModifiers::NONE, KeyCode::Char('r')) => {
                self.state.mode = AppMode::Modal(ModalKind::Rename);
                self.state.modal_input.clear();
            }
            (KeyModifiers::NONE, KeyCode::Char('.'))
            | (KeyModifiers::CONTROL, KeyCode::Char('h')) => {
                self.state.config.show_hidden = !self.state.config.show_hidden;
                self.refresh_dir().await;
                self.state.set_status(
                    format!(
                        "Hidden files: {}",
                        if self.state.config.show_hidden {
                            "shown"
                        } else {
                            "hidden"
                        }
                    ),
                    StatusKind::Info,
                );
            }

            // ── Search / filter ───────────────────────────────────────────
            (KeyModifiers::CONTROL, KeyCode::Char('f')) => {
                self.state.mode = AppMode::Search;
                self.state.search_query.clear();
            }
            (KeyModifiers::NONE, KeyCode::Char('/')) => {
                self.state.mode = AppMode::Search;
                self.state.search_query.clear();
            }

            // ── Sort cycling ─────────────────────────────────────────────
            (KeyModifiers::NONE, KeyCode::Char('s')) => {
                self.state.explorer.sort_order = self.state.explorer.sort_order.cycle();
                self.refresh_dir().await;
                self.state.set_status(
                    format!("Sort: {}", self.state.explorer.sort_order.label()),
                    StatusKind::Info,
                );
            }

            // ── Copy / Cut / Paste ────────────────────────────────────────
            (KeyModifiers::NONE, KeyCode::Char('c') | KeyCode::Char('y')) => {
                if let Some(path) = self.state.selected_path() {
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    self.state.clipboard_entry = Some((path, false));
                    self.state
                        .set_status(format!("Copied '{name}'"), StatusKind::Info);
                }
            }
            (KeyModifiers::NONE, KeyCode::Char('x')) => {
                if let Some(path) = self.state.selected_path() {
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    self.state.clipboard_entry = Some((path, true));
                    self.state
                        .set_status(format!("Cut '{name}'"), StatusKind::Info);
                }
            }
            (KeyModifiers::NONE, KeyCode::Char('p') | KeyCode::Char('v')) => {
                if let Some((src, is_cut)) = self.state.clipboard_entry.clone() {
                    if let Some(filename) = src.file_name() {
                        let dst = self.state.explorer.current_dir.join(filename);
                        if dst == src {
                            self.state.set_status(
                                "Source and destination are identical",
                                StatusKind::Warning,
                            );
                        } else {
                            let res = if is_cut {
                                operations::rename_entry(&src, &dst)
                            } else {
                                operations::copy_entry(&src, &dst)
                            };
                            match res {
                                Ok(_) => {
                                    if is_cut {
                                        self.state.clipboard_entry = None;
                                    }
                                    self.refresh_dir().await;
                                    self.state.set_status(
                                        format!("Pasted '{}'", filename.to_string_lossy()),
                                        StatusKind::Success,
                                    );
                                }
                                Err(e) => {
                                    self.state
                                        .set_status(format!("Paste error: {e}"), StatusKind::Error);
                                }
                            }
                        }
                    }
                } else {
                    self.state
                        .set_status("Clipboard is empty", StatusKind::Info);
                }
            }

            // ── Pin current dir ───────────────────────────────────────────
            (KeyModifiers::ALT, KeyCode::Char('p')) => {
                let dir = self.state.explorer.current_dir.clone();
                let name = dir
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| dir.to_string_lossy().to_string());
                self.state.config.pin_folder(name, dir);
                let _ = self.state.config.save();
                self.state
                    .set_status("Directory pinned", StatusKind::Success);
            }

            _ => {}
        }
        Ok(())
    }

    async fn handle_sidebar_key(&mut self, key: KeyEvent) -> Result<()> {
        let shortcuts = crate::ui::sidebar::os_shortcuts();
        let pinned_len = self.state.config.pinned_folders.len();
        let shortcuts_len = shortcuts.len();

        match key.code {
            KeyCode::Up => match self.state.sidebar.section {
                SidebarSection::Pinned => {
                    if self.state.sidebar.selected_index > 0 {
                        self.state.sidebar.selected_index -= 1;
                    }
                }
                SidebarSection::Shortcuts => {
                    if self.state.sidebar.selected_index > 0 {
                        self.state.sidebar.selected_index -= 1;
                    } else if pinned_len > 0 {
                        self.state.sidebar.section = SidebarSection::Pinned;
                        self.state.sidebar.selected_index = pinned_len.saturating_sub(1);
                    }
                }
            },
            KeyCode::Down => match self.state.sidebar.section {
                SidebarSection::Pinned => {
                    if self.state.sidebar.selected_index + 1 < pinned_len {
                        self.state.sidebar.selected_index += 1;
                    } else if shortcuts_len > 0 {
                        self.state.sidebar.section = SidebarSection::Shortcuts;
                        self.state.sidebar.selected_index = 0;
                    }
                }
                SidebarSection::Shortcuts => {
                    if self.state.sidebar.selected_index + 1 < shortcuts_len {
                        self.state.sidebar.selected_index += 1;
                    }
                }
            },
            KeyCode::Enter | KeyCode::Right => {
                let target_dir = match self.state.sidebar.section {
                    SidebarSection::Pinned => self
                        .state
                        .config
                        .pinned_folders
                        .get(self.state.sidebar.selected_index)
                        .map(|p| p.path.clone()),
                    SidebarSection::Shortcuts => shortcuts
                        .get(self.state.sidebar.selected_index)
                        .map(|(_, p, _)| p.clone()),
                };
                if let Some(dir) = target_dir {
                    if dir.exists() {
                        self.navigate_into(dir).await;
                        self.state.focused = FocusedPanel::Explorer;
                    } else {
                        self.state
                            .set_status("Location not found", StatusKind::Warning);
                    }
                }
            }
            KeyCode::Char('d') | KeyCode::Delete
                if self.state.sidebar.section == SidebarSection::Pinned && pinned_len > 0 =>
            {
                self.state
                    .config
                    .unpin_folder(self.state.sidebar.selected_index);
                let _ = self.state.config.save();
                if self.state.sidebar.selected_index >= self.state.config.pinned_folders.len() {
                    self.state.sidebar.selected_index =
                        self.state.config.pinned_folders.len().saturating_sub(1);
                }
                self.state
                    .set_status("Folder unpinned", StatusKind::Success);
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_preview_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.state.preview.scroll_offset > 0 {
                    self.state.preview.scroll_offset -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = self.state.preview.total_lines.saturating_sub(1);
                if self.state.preview.scroll_offset < max {
                    self.state.preview.scroll_offset += 1;
                }
            }
            KeyCode::PageUp => {
                self.state.preview.scroll_offset =
                    self.state.preview.scroll_offset.saturating_sub(20);
            }
            KeyCode::PageDown => {
                let max = self.state.preview.total_lines.saturating_sub(1);
                self.state.preview.scroll_offset = (self.state.preview.scroll_offset + 20).min(max);
            }
            KeyCode::Esc | KeyCode::Backspace => {
                self.state.focused = if self.state.dual_pane_enabled
                    && self.state.active_pane == ActivePane::Secondary
                {
                    FocusedPanel::ExplorerSecondary
                } else {
                    FocusedPanel::Explorer
                };
            }
            KeyCode::Char('[') => {
                self.state.preview.active_tab = match self.state.preview.active_tab {
                    PreviewTab::Preview => PreviewTab::Actions,
                    PreviewTab::Information => PreviewTab::Preview,
                    PreviewTab::Hex => PreviewTab::Information,
                    PreviewTab::Actions => PreviewTab::Hex,
                };
                self.update_preview().await;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if self.state.preview.active_tab == PreviewTab::Preview {
                    self.state.focused = if self.state.dual_pane_enabled
                        && self.state.active_pane == ActivePane::Secondary
                    {
                        FocusedPanel::ExplorerSecondary
                    } else {
                        FocusedPanel::Explorer
                    };
                } else {
                    self.state.preview.active_tab = match self.state.preview.active_tab {
                        PreviewTab::Information => PreviewTab::Preview,
                        PreviewTab::Hex => PreviewTab::Information,
                        PreviewTab::Actions => PreviewTab::Hex,
                        PreviewTab::Preview => PreviewTab::Preview,
                    };
                    self.update_preview().await;
                }
            }
            KeyCode::Char(']') | KeyCode::Right => {
                self.state.preview.active_tab = match self.state.preview.active_tab {
                    PreviewTab::Preview => PreviewTab::Information,
                    PreviewTab::Information => PreviewTab::Hex,
                    PreviewTab::Hex => PreviewTab::Actions,
                    PreviewTab::Actions => PreviewTab::Preview,
                };
                self.update_preview().await;
            }
            KeyCode::Char(c @ '1'..='9')
                if self.state.preview.active_tab == PreviewTab::Actions =>
            {
                let idx = (c as u8 - b'1') as usize;
                self.execute_preview_action(idx).await;
            }
            _ => {}
        }
        Ok(())
    }

    async fn execute_preview_action(&mut self, idx: usize) {
        let Some(data) = &self.state.preview.full_data else {
            return;
        };
        let Some(action) = data.actions.get(idx) else {
            return;
        };
        let path = self.state.preview.path.clone();
        let Some(path) = path else {
            return;
        };

        match action.command {
            "xdg-open" => {
                let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
                self.state.set_status(
                    format!("Launched xdg-open on {}", path.display()),
                    StatusKind::Success,
                );
            }
            "edit" => {
                if path.is_file() {
                    match Buffer::from_file(path) {
                        Ok(buf) => {
                            self.editor_buf = Some(buf);
                            self.state.mode = AppMode::Edit;
                            self.state.focused = FocusedPanel::Editor;
                        }
                        Err(e) => {
                            self.state
                                .set_status(format!("Cannot edit: {e}"), StatusKind::Error);
                        }
                    }
                }
            }
            "copy_path" => {
                self.state
                    .set_status(format!("Path: {}", path.display()), StatusKind::Info);
            }
            "extract" => {
                let target = path
                    .parent()
                    .unwrap_or(&path)
                    .join(path.file_stem().unwrap_or_default());
                if let Some(arch) = &data.archive {
                    match arch.extract_all(&path, &target) {
                        Ok(n) => self.state.set_status(
                            format!("Extracted {} items to {}", n, target.display()),
                            StatusKind::Success,
                        ),
                        Err(e) => self
                            .state
                            .set_status(format!("Extraction error: {}", e), StatusKind::Error),
                    }
                    self.refresh_dir().await;
                }
            }
            "play_mpv" => {
                let _ = std::process::Command::new("mpv")
                    .arg("--no-video")
                    .arg(&path)
                    .spawn();
                self.state.set_status(
                    format!("Streaming audio with mpv: {}", path.display()),
                    StatusKind::Success,
                );
            }
            _ => {}
        }
    }

    async fn toggle_dual_pane(&mut self) {
        self.state.dual_pane_enabled = !self.state.dual_pane_enabled;
        if self.state.dual_pane_enabled {
            if self.state.explorer_secondary.entries.is_empty() {
                self.state.explorer_secondary.current_dir = self.state.explorer.current_dir.clone();
                let entries = list_dir(
                    &self.state.explorer_secondary.current_dir.clone(),
                    self.state.config.show_hidden,
                    self.state.explorer_secondary.sort_order,
                )
                .unwrap_or_default();
                self.state.explorer_secondary.entries = entries;
            }
            self.state.set_status(
                "Dual-Pane mode enabled (Tab switches active pane)",
                StatusKind::Success,
            );
        } else {
            self.state.active_pane = ActivePane::Primary;
            self.state.focused = FocusedPanel::Explorer;
            self.state
                .set_status("Dual-Pane mode disabled", StatusKind::Info);
        }
    }

    async fn handle_settings_key(&mut self, key: KeyEvent) -> Result<bool> {
        let (mut section_idx, mut item_idx, mut in_items) = match self.state.mode {
            AppMode::SettingsStudio {
                section_idx,
                item_idx,
                in_items,
            } => (section_idx, item_idx, in_items),
            _ => return Ok(false),
        };

        match key.code {
            KeyCode::Esc | KeyCode::F(10) => {
                let _ = SettingsPersistence::save_with_backup(&self.state.config);
                self.state.mode = AppMode::Browse;
                self.state
                    .set_status("Settings saved successfully", StatusKind::Success);
                return Ok(false);
            }
            KeyCode::Tab => {
                in_items = !in_items;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !in_items {
                    if section_idx > 0 {
                        section_idx -= 1;
                        item_idx = 0;
                    }
                } else {
                    item_idx = item_idx.saturating_sub(1);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !in_items {
                    if section_idx + 1 < SettingsSection::ALL.len() {
                        section_idx += 1;
                        item_idx = 0;
                    }
                } else {
                    let sec = SettingsSection::ALL[section_idx];
                    let count = get_section_items(sec, &self.state.config).len();
                    if item_idx + 1 < count {
                        item_idx += 1;
                    }
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Left | KeyCode::Right => {
                let sec = SettingsSection::ALL[section_idx];
                let items = get_section_items(sec, &self.state.config);
                if let Some(item) = items.get(item_idx) {
                    match item.key {
                        "transparent_bg" => {
                            self.state.config.theme.transparent_background =
                                !self.state.config.theme.transparent_background;
                            self.theme = Theme::from_name(
                                &self.state.config.theme.name,
                                self.state.config.theme.transparent_background,
                            );
                        }
                        "theme" => {
                            let themes = Theme::list_themes();
                            let curr = themes
                                .iter()
                                .position(|t| {
                                    t.0.eq_ignore_ascii_case(&self.state.config.theme.name)
                                })
                                .unwrap_or(0);
                            let next = if key.code == KeyCode::Left {
                                if curr == 0 {
                                    themes.len() - 1
                                } else {
                                    curr - 1
                                }
                            } else {
                                (curr + 1) % themes.len()
                            };
                            let new_theme_name = themes[next].0;
                            self.state.config.theme.name = new_theme_name.to_string();
                            self.theme = Theme::from_name(
                                new_theme_name,
                                self.state.config.theme.transparent_background,
                            );
                        }
                        "icon_pack" => {
                            let packs = crate::icons::IconPack::all();
                            let curr = packs
                                .iter()
                                .position(|&p| p == self.state.icon_manager.pack)
                                .unwrap_or(0);
                            let next = if key.code == KeyCode::Left {
                                if curr == 0 {
                                    packs.len() - 1
                                } else {
                                    curr - 1
                                }
                            } else {
                                (curr + 1) % packs.len()
                            };
                            let new_pack = packs[next];
                            self.state.icon_manager.pack = new_pack;
                            self.state.config.icon_pack = new_pack.as_str().to_string();
                        }
                        "show_hidden" => {
                            self.state.config.show_hidden = !self.state.config.show_hidden;
                            self.refresh_dir().await;
                        }
                        "auto_pair" => {
                            self.state.config.editor.auto_pair =
                                !self.state.config.editor.auto_pair;
                        }
                        "line_numbers" => {
                            self.state.config.editor.line_numbers =
                                !self.state.config.editor.line_numbers;
                        }
                        "show_completion" => {
                            self.state.config.editor.show_completion =
                                !self.state.config.editor.show_completion;
                        }
                        "tab_width" => {
                            self.state.config.editor.tab_width =
                                match self.state.config.editor.tab_width {
                                    2 => 4,
                                    4 => 8,
                                    _ => 2,
                                };
                        }
                        "sidebar_width" => {
                            if key.code == KeyCode::Left {
                                self.state.config.sidebar_width =
                                    self.state.config.sidebar_width.saturating_sub(2).max(12);
                            } else {
                                self.state.config.sidebar_width =
                                    (self.state.config.sidebar_width + 2).min(40);
                            }
                        }
                        "preview_width" => {
                            if key.code == KeyCode::Left {
                                self.state.config.preview_width =
                                    self.state.config.preview_width.saturating_sub(5).max(20);
                            } else {
                                self.state.config.preview_width =
                                    (self.state.config.preview_width + 5).min(60);
                            }
                        }
                        "profile_preset" => {
                            let profiles = ProfileKind::ALL;
                            let next = (item_idx + 1) % profiles.len();
                            profiles[next].apply_to_config(&mut self.state.config);
                            self.state.show_sidebar = self.state.config.show_sidebar;
                            self.state.show_preview = self.state.config.show_preview;
                            self.state.show_terminal = self.state.config.show_terminal;
                            self.state.show_monitor = self.state.config.show_monitor;
                        }
                        _ => {}
                    }
                    let _ = self.state.config.save();
                }
            }
            _ => {}
        }

        self.state.mode = AppMode::SettingsStudio {
            section_idx,
            item_idx,
            in_items,
        };
        Ok(false)
    }

    async fn handle_command_palette_key(&mut self, key: KeyEvent) -> Result<bool> {
        let (mut query, mut selected_idx) = match &self.state.mode {
            AppMode::CommandPalette {
                query,
                selected_idx,
            } => (query.clone(), *selected_idx),
            _ => return Ok(false),
        };

        match key.code {
            KeyCode::Esc => {
                self.state.mode = AppMode::Browse;
                return Ok(false);
            }
            KeyCode::Enter => {
                let items = CommandRegistry::search(&query);
                if let Some(cmd) = items.get(selected_idx) {
                    let id = cmd.id;
                    self.state.mode = AppMode::Browse;
                    self.execute_palette_command(id).await;
                    return Ok(false);
                }
                self.state.mode = AppMode::Browse;
                return Ok(false);
            }
            KeyCode::Up => {
                selected_idx = selected_idx.saturating_sub(1);
            }
            KeyCode::Down => {
                let count = CommandRegistry::search(&query).len();
                if selected_idx + 1 < count {
                    selected_idx += 1;
                }
            }
            KeyCode::Backspace => {
                query.pop();
                selected_idx = 0;
            }
            KeyCode::Char(c) => {
                query.push(c);
                selected_idx = 0;
            }
            _ => {}
        }

        self.state.mode = AppMode::CommandPalette {
            query,
            selected_idx,
        };
        Ok(false)
    }

    async fn execute_palette_command(&mut self, id: &str) {
        match id {
            "toggle_dual_pane" => self.toggle_dual_pane().await,
            "open_settings" => {
                self.state.mode = AppMode::SettingsStudio {
                    section_idx: 0,
                    item_idx: 0,
                    in_items: false,
                };
            }
            "open_help" => self.state.mode = AppMode::Help,
            "toggle_hidden" => {
                self.state.config.show_hidden = !self.state.config.show_hidden;
                self.refresh_dir().await;
            }
            "cycle_sort" => {
                self.state.explorer.sort_order = self.state.explorer.sort_order.next();
                self.refresh_dir().await;
            }
            "new_file" => {
                self.state.mode = AppMode::Modal(ModalKind::NewFile);
                self.state.modal_input.clear();
            }
            "new_dir" => {
                self.state.mode = AppMode::Modal(ModalKind::NewDir);
                self.state.modal_input.clear();
            }
            "refresh_view" => self.refresh_dir().await,
            "toggle_terminal" => self.toggle_terminal(),
            "toggle_monitor" => self.toggle_monitor(),
            "toggle_sidebar" => self.state.show_sidebar = !self.state.show_sidebar,
            "toggle_preview" => self.state.show_preview = !self.state.show_preview,
            id if id.starts_with("theme_") => {
                let theme_name = match id {
                    "theme_catppuccin_mocha" => "Catppuccin Mocha",
                    "theme_catppuccin_macchiato" => "Catppuccin Macchiato",
                    "theme_catppuccin_frappe" => "Catppuccin Frappé",
                    "theme_catppuccin_latte" => "Catppuccin Latte",
                    "theme_dracula" => "Dracula",
                    "theme_nord" => "Nord",
                    "theme_gruvbox" => "Gruvbox",
                    "theme_tokyo_night" => "Tokyo Night",
                    "theme_one_dark" => "One Dark",
                    "theme_solarized_dark" => "Solarized Dark",
                    "theme_solarized_light" => "Solarized Light",
                    "theme_monokai" => "Monokai",
                    "theme_default_light" => "Default Light",
                    _ => "Default Dark",
                };
                self.theme =
                    Theme::from_name(theme_name, self.state.config.theme.transparent_background);
                self.state.config.theme.name = theme_name.to_string();
                let _ = self.state.config.save();
                self.state.set_status(
                    format!("Theme switched to {}", theme_name),
                    StatusKind::Success,
                );
            }
            id if id.starts_with("profile_") => {
                let profile = match id {
                    "profile_developer" => ProfileKind::Developer,
                    "profile_minimal" => ProfileKind::Minimal,
                    "profile_terminal" => ProfileKind::TerminalPowerUser,
                    "profile_monitor" => ProfileKind::Monitoring,
                    _ => ProfileKind::Default,
                };
                profile.apply_to_config(&mut self.state.config);
                self.state.show_sidebar = self.state.config.show_sidebar;
                self.state.show_preview = self.state.config.show_preview;
                self.state.show_terminal = self.state.config.show_terminal;
                self.state.show_monitor = self.state.config.show_monitor;
                if self.state.show_terminal {
                    self.start_terminal_session();
                }
                let _ = self.state.config.save();
                self.state.set_status(
                    format!("Applied profile: {}", profile.name()),
                    StatusKind::Success,
                );
            }
            id if id.starts_with("icons_") => {
                let pack = match id {
                    "icons_catppuccin" => crate::icons::IconPack::Catppuccin,
                    "icons_nerd_font" => crate::icons::IconPack::NerdFont,
                    "icons_unicode" => crate::icons::IconPack::Unicode,
                    "icons_ascii" => crate::icons::IconPack::Ascii,
                    "icons_none" => crate::icons::IconPack::None,
                    _ => crate::icons::IconPack::Catppuccin,
                };
                self.state.icon_manager.pack = pack;
                self.state.config.icon_pack = pack.as_str().to_string();
                let _ = self.state.config.save();
                self.state.set_status(
                    format!("Icon pack switched to {}", pack.display_name()),
                    StatusKind::Success,
                );
            }
            "fonts_setup_guide" => {
                self.state.mode = AppMode::SettingsStudio {
                    section_idx: 1,
                    item_idx: 2,
                    in_items: true,
                };
            }
            "plugins_reload" => {
                let dir = self
                    .state
                    .explorer
                    .current_dir
                    .to_string_lossy()
                    .to_string();
                self.state.plugin_manager = crate::plugins::PluginManager::new();
                self.state
                    .plugin_manager
                    .discover_and_load(&crate::plugins::PluginManager::user_plugins_dir(), &dir);
                self.state.set_status(
                    format!(
                        "Plugins reloaded ({} found in ~/.local/share/termexplorer/plugins)",
                        self.state.plugin_manager.plugins.len()
                    ),
                    StatusKind::Info,
                );
            }
            "git_panel" => {
                self.state.git_status = GitRepoStatus::query(&self.state.explorer.current_dir);
                self.state.mode = AppMode::GitDashboard { selected_idx: 0 };
            }
            "git_status" => {
                self.state.git_status = GitRepoStatus::query(&self.state.explorer.current_dir);
                let msg = if self.state.git_status.is_dirty {
                    format!(
                        "Git ({}): Working directory has unstaged/modified changes",
                        self.state.git_status.branch
                    )
                } else {
                    format!("Git ({}): Clean working tree", self.state.git_status.branch)
                };
                self.state.set_status(msg, StatusKind::Info);
            }
            "action_vscode" => {
                let _ = std::process::Command::new("code")
                    .arg(&self.state.explorer.current_dir)
                    .spawn();
                self.state
                    .set_status("Launched VS Code", StatusKind::Success);
            }
            "action_cargo_test" => {
                if let Some(session) = &mut self.terminal_session {
                    let _ = session.send_input(b"cargo test\n");
                    self.state.show_terminal = true;
                    self.state.focused = FocusedPanel::Terminal;
                }
            }
            "action_cargo_build" => {
                if let Some(session) = &mut self.terminal_session {
                    let _ = session.send_input(b"cargo build\n");
                    self.state.show_terminal = true;
                    self.state.focused = FocusedPanel::Terminal;
                }
            }
            "action_open_terminal" => {
                let _ = std::process::Command::new("x-terminal-emulator")
                    .current_dir(&self.state.explorer.current_dir)
                    .spawn();
                self.state
                    .set_status("Launched external terminal", StatusKind::Success);
            }
            _ => {}
        }
    }

    async fn handle_git_dashboard_key(&mut self, key: KeyEvent) -> Result<bool> {
        let mut selected_idx = match self.state.mode {
            AppMode::GitDashboard { selected_idx } => selected_idx,
            _ => return Ok(false),
        };

        match key.code {
            KeyCode::Esc => {
                self.state.mode = AppMode::Browse;
            }
            KeyCode::Up => {
                selected_idx = selected_idx.saturating_sub(1);
            }
            KeyCode::Down => {
                let count = self.state.git_status.staged_files.len().max(1);
                if selected_idx + 1 < count {
                    selected_idx += 1;
                }
            }
            KeyCode::Char('s') => {
                if let Some(f) = self.state.git_status.unstaged_files.first().or(self
                    .state
                    .git_status
                    .untracked_files
                    .first())
                {
                    let path = f.path.clone();
                    crate::git::GitRepoStatus::stage_file(&self.state.explorer.current_dir, &path);
                    self.state.git_status = GitRepoStatus::query(&self.state.explorer.current_dir);
                    self.state
                        .set_status(format!("Staged: {}", path), StatusKind::Success);
                }
            }
            KeyCode::Char('u') => {
                if let Some(f) = self.state.git_status.staged_files.get(selected_idx) {
                    let path = f.path.clone();
                    crate::git::GitRepoStatus::unstage_file(
                        &self.state.explorer.current_dir,
                        &path,
                    );
                    self.state.git_status = GitRepoStatus::query(&self.state.explorer.current_dir);
                    self.state
                        .set_status(format!("Unstaged: {}", path), StatusKind::Info);
                }
            }
            KeyCode::Char('c') => {
                self.state.mode =
                    AppMode::Modal(ModalKind::Message("Enter commit message:".into()));
            }
            _ => {}
        }

        if matches!(self.state.mode, AppMode::GitDashboard { .. }) {
            self.state.mode = AppMode::GitDashboard { selected_idx };
        }
        Ok(false)
    }

    async fn handle_onboarding_key(&mut self, key: KeyEvent) -> Result<bool> {
        let (step, mut selected_option) = match self.state.mode {
            AppMode::Onboarding {
                step,
                selected_option,
            } => (step, selected_option),
            _ => return Ok(false),
        };

        match key.code {
            KeyCode::Esc => {
                self.state.mode = AppMode::Browse;
                return Ok(false);
            }
            KeyCode::Up => {
                selected_option = selected_option.saturating_sub(1);
            }
            KeyCode::Down => {
                selected_option += 1;
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let next_step = match step {
                    OnboardingStep::Welcome => OnboardingStep::ThemeSelection,
                    OnboardingStep::ThemeSelection => {
                        let themes = Theme::list_themes();
                        if let Some(t) = themes.get(selected_option) {
                            self.state.config.theme.name = t.0.to_string();
                            self.theme = Theme::from_name(
                                t.0,
                                self.state.config.theme.transparent_background,
                            );
                        }
                        OnboardingStep::LayoutSelection
                    }
                    OnboardingStep::LayoutSelection => {
                        let profiles = ProfileKind::ALL;
                        if let Some(p) = profiles.get(selected_option) {
                            p.apply_to_config(&mut self.state.config);
                            self.state.show_sidebar = self.state.config.show_sidebar;
                            self.state.show_preview = self.state.config.show_preview;
                            self.state.show_terminal = self.state.config.show_terminal;
                            self.state.show_monitor = self.state.config.show_monitor;
                        }
                        OnboardingStep::IntegrationsScan
                    }
                    OnboardingStep::IntegrationsScan => OnboardingStep::Complete,
                    OnboardingStep::Complete => {
                        let _ = self.state.config.save();
                        self.state.mode = AppMode::Browse;
                        return Ok(false);
                    }
                };
                self.state.mode = AppMode::Onboarding {
                    step: next_step,
                    selected_option: 0,
                };
                return Ok(false);
            }
            _ => {}
        }

        self.state.mode = AppMode::Onboarding {
            step,
            selected_option,
        };
        Ok(false)
    }

    // ── Focus cycling ────────────────────────────────────────────────────────

    fn cycle_focus(&mut self) {
        self.state.focused = match self.state.focused {
            FocusedPanel::Sidebar => FocusedPanel::Explorer,
            FocusedPanel::Explorer => {
                if self.state.dual_pane_enabled {
                    FocusedPanel::ExplorerSecondary
                } else if self.state.show_preview {
                    FocusedPanel::Preview
                } else if self.state.show_terminal {
                    FocusedPanel::Terminal
                } else {
                    FocusedPanel::Sidebar
                }
            }
            FocusedPanel::ExplorerSecondary => {
                if self.state.show_preview {
                    FocusedPanel::Preview
                } else if self.state.show_terminal {
                    FocusedPanel::Terminal
                } else {
                    FocusedPanel::Sidebar
                }
            }
            FocusedPanel::Preview => {
                if self.state.show_terminal {
                    FocusedPanel::Terminal
                } else {
                    FocusedPanel::Sidebar
                }
            }
            FocusedPanel::Terminal => FocusedPanel::Sidebar,
            FocusedPanel::Editor => FocusedPanel::Editor,
            FocusedPanel::Monitor => FocusedPanel::Explorer,
        };
    }

    fn cycle_focus_back(&mut self) {
        self.state.focused = match self.state.focused {
            FocusedPanel::Sidebar => {
                if self.state.show_terminal {
                    FocusedPanel::Terminal
                } else if self.state.show_preview {
                    FocusedPanel::Preview
                } else if self.state.dual_pane_enabled {
                    FocusedPanel::ExplorerSecondary
                } else {
                    FocusedPanel::Explorer
                }
            }
            FocusedPanel::Explorer => FocusedPanel::Sidebar,
            FocusedPanel::ExplorerSecondary => FocusedPanel::Explorer,
            FocusedPanel::Preview => {
                if self.state.dual_pane_enabled {
                    FocusedPanel::ExplorerSecondary
                } else {
                    FocusedPanel::Explorer
                }
            }
            FocusedPanel::Terminal => FocusedPanel::Preview,
            FocusedPanel::Editor => FocusedPanel::Editor,
            FocusedPanel::Monitor => FocusedPanel::Explorer,
        };
    }

    // ── Toggle helpers ────────────────────────────────────────────────────────

    fn toggle_terminal(&mut self) {
        self.state.show_terminal = !self.state.show_terminal;
        self.state.show_monitor = false; // only one bottom panel at a time

        if self.state.show_terminal {
            self.start_terminal_session();
            self.state.focused = FocusedPanel::Terminal;
        } else {
            self.state.focused = FocusedPanel::Explorer;
        }
    }

    fn toggle_monitor(&mut self) {
        self.state.show_monitor = !self.state.show_monitor;
        self.state.show_terminal = false;

        if self.state.show_monitor {
            if self.monitor_snapshot.is_none() {
                self.monitor_snapshot = Some(self.monitor.refresh());
            }
            self.state.focused = FocusedPanel::Monitor;
        } else {
            self.state.focused = FocusedPanel::Explorer;
        }
    }
}

// ── Modal dialog renderer ─────────────────────────────────────────────────────

fn render_modal(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    kind: &ModalKind,
    input: &str,
    theme: &Theme,
) {
    use ratatui::{
        layout::Rect,
        style::Style,
        text::{Line, Span},
        widgets::{Block, BorderType, Borders, Clear, Paragraph},
    };

    let (title, prompt) = match kind {
        ModalKind::NewFile => ("New File", "Enter filename:"),
        ModalKind::NewDir => ("New Directory", "Enter directory name:"),
        ModalKind::Rename => ("Rename", "New name:"),
        ModalKind::Delete => ("Confirm Delete", "Type 'y' to confirm:"),
        ModalKind::PinFolder => ("Pin Folder", "Path to pin:"),
        ModalKind::GoToLine => ("Go to Line", "Line number:"),
        ModalKind::Message(m) => ("Info", m.as_str()),
        ModalKind::Error(m) => ("Error", m.as_str()),
    };

    let popup_w = area.width.min(50);
    let popup_h = 5u16;
    let popup = Rect {
        x: area.x + (area.width.saturating_sub(popup_w)) / 2,
        y: area.y + (area.height / 2).saturating_sub(popup_h / 2),
        width: popup_w,
        height: popup_h,
    };

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(Span::styled(format!(" {} ", title), theme.header_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focused_style());

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let lines = vec![
        Line::from(Span::styled(prompt, theme.normal_style())),
        Line::from(vec![
            Span::styled("> ", theme.accent_style()),
            Span::styled(input, Style::default().fg(ratatui::style::Color::White)),
            Span::styled("█", theme.accent_style()),
        ]),
    ];
    frame.render_widget(Paragraph::new(lines), inner);
}
