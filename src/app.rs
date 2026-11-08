//! Core application struct, event loop, and input handling.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use crate::{
    commands::CommandRegistry,
    config::Config,
    editor::{auto_pair, buffer::Buffer},
    events::{start_event_loop_with_sender, AppEvent},
    filesystem::{
        browser::list_dir,
        chunked::{load_initial_viewport, stream_dir_chunks},
        ledger::{FsOperationLedger, RecordedOperation},
        operations,
    },
    git::GitRepoStatus,
    monitor::{Monitor, MonitorSnapshot},
    preview::{FullPreviewData, PreviewManager, PreviewTab},
    settings::{get_section_items, ProfileKind, SettingsPersistence, SettingsSection},
    shell::{input::key_to_pty_bytes, TerminalSession},
    state::{ActivePane, AppMode, AppState, FocusedPanel, ModalKind, SidebarSection, StatusKind},
    theme::Theme,
    ui::{
        command_palette::render_command_palette,
        diagnostics::render_diagnostics,
        editor::render_editor,
        file_explorer::{render_file_explorer, render_secondary_explorer},
        git_panel::render_git_panel,
        header::render_header,
        help::render_help,
        hit_test::{AppAction, HitRegistry},
        layout::{compute_layout, AppLayout, LayoutConfig},
        onboarding::{render_onboarding, OnboardingStep},
        plugin_center::render_plugin_center,
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
    fs_rx: Option<mpsc::UnboundedReceiver<()>>,
    hit_registry: HitRegistry,
    pub layout: AppLayout,
    fs_ledger: FsOperationLedger,
    preview_ticket: Arc<AtomicU64>,
    dir_ticket: Arc<AtomicU64>,
    event_tx: Option<mpsc::UnboundedSender<AppEvent>>,
}

impl App {
    pub async fn new(config: Config) -> Result<Self> {
        let theme = Theme::from_name(&config.theme.name, config.theme.transparent_background);
        let max_preview_bytes = config.max_preview_bytes;
        let mut state = AppState::new(config);

        // Initial fast viewport directory load (<30ms)
        let entries = load_initial_viewport(
            &state.explorer.current_dir.clone(),
            state.config.show_hidden,
            state.explorer.sort_order,
            100,
        )
        .unwrap_or_default();
        state.explorer.entries = entries;

        let (_watcher, fs_rx) = match crate::filesystem::watcher::watch_directory_debounced(
            &state.explorer.current_dir,
            Duration::from_millis(150),
        ) {
            Ok((w, rx)) => (Some(w), Some(rx)),
            Err(_) => (None, None),
        };

        let mut app = Self {
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
            hit_registry: HitRegistry::new(),
            layout: AppLayout::default(),
            fs_ledger: FsOperationLedger::new(100),
            preview_ticket: Arc::new(AtomicU64::new(0)),
            dir_ticket: Arc::new(AtomicU64::new(0)),
            event_tx: None,
        };

        if app.state.show_terminal {
            app.start_terminal_session();
            app.state.clear_status();
        }

        Ok(app)
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
        let (tx, mut events) = start_event_loop_with_sender(100);
        self.event_tx = Some(tx);

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
                // Filesystem changes (debounced)
                Some(()) = async {
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
                AppEvent::Mouse(mouse) => {
                    self.handle_mouse(mouse).await?;
                }
                AppEvent::FsChange => self.refresh_dir().await,
                AppEvent::MonitorRefresh => {}
                AppEvent::PtyOutput => {}
                AppEvent::ClearStatus => self.state.clear_status(),
                AppEvent::PreviewReady { ticket, data } => {
                    if ticket == self.preview_ticket.load(Ordering::Relaxed) {
                        let full_data = *data;
                        self.preview_manager
                            .insert_cache(full_data.path.clone(), full_data.clone());
                        self.apply_preview_data(full_data);
                    }
                }
                AppEvent::DirChunkReady { ticket, chunk } => {
                    if ticket == self.dir_ticket.load(Ordering::Relaxed) {
                        let exp = self.state.active_explorer_mut();
                        if chunk.is_initial {
                            exp.entries = chunk.entries;
                        } else {
                            exp.entries.extend(chunk.entries);
                        }
                        if chunk.is_final {
                            crate::filesystem::browser::sort_entries_slice(
                                &mut exp.entries,
                                exp.sort_order,
                            );
                        }
                    }
                }
            }
        }

        // Clean up terminal session.
        self.cleanup_terminal_session();
        Ok(())
    }

    // ── Rendering ────────────────────────────────────────────────────────────

    fn draw(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        let frame_start = Instant::now();
        let mut hit_registry = std::mem::take(&mut self.hit_registry);
        hit_registry.clear();
        let mut current_layout = self.layout;

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

            // ── Full-screen editor mode (only when preview_maximized is active) ──
            if matches!(state.mode, AppMode::Edit) && state.preview_maximized {
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
                preview_maximized: state.preview_maximized,
                terminal_maximized: state.terminal_maximized,
            };

            let areas = compute_layout(area, &layout_cfg);
            current_layout = areas;

            // ── Header (Workstation Command Bar) ──────────────────────────
            render_header(
                frame,
                areas.header,
                state,
                monitor_snap.as_ref(),
                theme,
                &mut hit_registry,
            );

            // ── Sidebar ───────────────────────────────────────────────────
            if let Some(sb_area) = areas.sidebar {
                render_sidebar(frame, sb_area, state, theme);
            }

            // ── File explorer ─────────────────────────────────────────────
            render_file_explorer(
                frame,
                areas.explorer,
                state,
                theme,
                &mut hit_registry,
            );

            // ── Dual Pane Secondary Explorer ──────────────────────────────
            if let Some(sec_area) = areas.explorer_secondary {
                render_secondary_explorer(frame, sec_area, state, theme);
            }

            // ── Preview / Embedded Editor Workspace (75% Panel) ───────────
            if let Some(prev_area) = areas.preview {
                if matches!(state.mode, AppMode::Edit) {
                    if let Some(buf) = editor_buf {
                        render_editor(frame, prev_area, buf, state, theme);
                    }
                } else {
                    render_preview(frame, prev_area, state, theme, &mut hit_registry);
                }
            }

            // ── Bottom panel ──────────────────────────────────────────────
            if let Some(bottom_area) = areas.bottom_panel {
                if state.show_terminal {
                    render_terminal(
                        frame,
                        bottom_area,
                        scrollback,
                        state,
                        theme,
                        &mut hit_registry,
                    );
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
                render_help(
                    frame,
                    area,
                    state.help_category,
                    state.help_scroll,
                    theme,
                    &mut hit_registry,
                );
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
                let items = CommandRegistry::search_with_context(
                    query,
                    &state.config.pinned_folders,
                    &state.recent_dirs.dirs,
                    state.focused,
                );
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

            // ── Interactive Plugin Center & Marketplace ───────────────────
            if let AppMode::PluginCenter {
                tab,
                selected_idx,
                query,
                category_idx,
                confirm_install,
                inspecting_logs,
            } = &state.mode
            {
                let view = crate::ui::plugin_center::PluginCenterView {
                    tab: *tab,
                    selected_idx: *selected_idx,
                    query,
                    category_idx: *category_idx,
                    confirm_install: confirm_install.as_deref(),
                    inspecting_logs: *inspecting_logs,
                };
                render_plugin_center(frame, area, state, theme, &view);
            }

            // ── Screen Hitbox Visualizer (Debug overlay) ──────────────────
            if state.debug_hitboxes {
                for r in hit_registry.regions() {
                    let b = ratatui::widgets::Block::default()
                        .borders(ratatui::widgets::Borders::ALL)
                        .border_style(
                            ratatui::style::Style::default()
                                .fg(ratatui::style::Color::Yellow),
                        );
                    frame.render_widget(b, r.rect);
                }
            }

            // ── Diagnostics Inspector (F12) ───────────────────────────────
            if state.show_diagnostics {
                render_diagnostics(frame, area, state, theme, &hit_registry);
            }
        })?;

        self.state.last_frame_duration_us = frame_start.elapsed().as_micros();
        self.hit_registry = hit_registry;
        self.layout = current_layout;

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

        // Detect external modifications in open editor buffer
        if let Some(buf) = &self.editor_buf {
            if buf.check_external_modification() {
                self.state.set_status(
                    "Warning: File modified externally! Press Ctrl+R to reload or Ctrl+S to overwrite.",
                    StatusKind::Warning,
                );
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

        // Sub-30ms first-viewport loading for instant responsiveness
        let initial = load_initial_viewport(&dir, show_hidden, sort, 100).unwrap_or_default();
        let exp = self.state.active_explorer_mut();
        exp.entries = initial;
        exp.selected_index = exp.selected_index.min(exp.entries.len().saturating_sub(1));
        self.update_preview().await;

        // Monotonic cancellation background streaming
        let ticket = self.dir_ticket.fetch_add(1, Ordering::SeqCst) + 1;
        if let Some(tx) = &self.event_tx {
            let tx = tx.clone();
            let token = Arc::clone(&self.dir_ticket);
            let dir_clone = dir.clone();
            tokio::spawn(async move {
                let (chunk_tx, mut chunk_rx) = mpsc::channel(16);
                let token_bg = Arc::clone(&token);
                tokio::spawn(async move {
                    stream_dir_chunks(
                        dir_clone,
                        show_hidden,
                        sort,
                        250,
                        token_bg,
                        ticket,
                        chunk_tx,
                    )
                    .await;
                });
                while let Some(chunk) = chunk_rx.recv().await {
                    if token.load(Ordering::Relaxed) != ticket {
                        break;
                    }
                    let _ = tx.send(AppEvent::DirChunkReady {
                        ticket,
                        chunk: Box::new(chunk),
                    });
                }
            });
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
            if let Ok((w, rx)) = crate::filesystem::watcher::watch_directory_debounced(
                &dir,
                Duration::from_millis(150),
            ) {
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

            // Only scan directory if not already cached for this exact path
            let needs_scan = match &self.state.preview.dir_data {
                Some(d) => d.path != path,
                None => true,
            };

            if needs_scan {
                if let Ok(read_dir) = std::fs::read_dir(&path) {
                    let mut items = Vec::new();
                    let mut dirs = 0;
                    let mut files = 0;
                    let mut bytes: u64 = 0;
                    for entry in read_dir.flatten() {
                        let file_type = entry.file_type().ok();
                        let is_dir = file_type.map(|t| t.is_dir()).unwrap_or(false);
                        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        if is_dir {
                            dirs += 1;
                        } else {
                            files += 1;
                            bytes = bytes.saturating_add(size);
                        }
                        items.push((
                            entry.file_name().to_string_lossy().to_string(),
                            is_dir,
                            size,
                        ));
                    }
                    items.sort_by(|a, b| match (a.1, b.1) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.0.to_lowercase().cmp(&b.0.to_lowercase()),
                    });
                    let total = items.len();
                    self.state.preview.dir_data = Some(crate::state::DirectoryPreviewData {
                        path: path.clone(),
                        total_count: total,
                        dir_count: dirs,
                        file_count: files,
                        total_bytes: bytes,
                        items,
                    });
                }
            }
            return;
        }

        self.state.preview.dir_data = None;

        // Instant hit if already present in LRU cache
        if let Some(cached) = self
            .preview_manager
            .get_cached(&path, self.state.preview.active_tab)
        {
            self.apply_preview_data(cached);
            return;
        }

        let ticket = self.preview_ticket.fetch_add(1, Ordering::SeqCst) + 1;

        // Set responsive placeholder preview
        self.state.preview.lines = vec!["  Loading preview...".to_string()];
        self.state.preview.highlighted_spans.clear();
        self.state.preview.total_lines = 1;
        self.state.preview.full_data = None;

        if let Some(tx) = &self.event_tx {
            let tx = tx.clone();
            let token = Arc::clone(&self.preview_ticket);
            let path_clone = path.clone();
            let tab = self.state.preview.active_tab;
            let max_bytes = self.state.config.max_preview_bytes;

            tokio::task::spawn_blocking(move || {
                if let Some(data) = PreviewManager::generate_preview_cancellable(
                    &path_clone,
                    tab,
                    max_bytes,
                    60,
                    30,
                    Some((&token, ticket)),
                ) {
                    if token.load(Ordering::Relaxed) == ticket {
                        let _ = tx.send(AppEvent::PreviewReady {
                            ticket,
                            data: Box::new(data),
                        });
                    }
                }
            });
        } else {
            let full_data =
                self.preview_manager
                    .load_file(&path, self.state.preview.active_tab, 60, 30);
            self.apply_preview_data(full_data);
        }
    }

    fn apply_preview_data(&mut self, full_data: FullPreviewData) {
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
        // F12: Diagnostics overlay toggle
        if key.code == KeyCode::F(12) {
            self.state.show_diagnostics = !self.state.show_diagnostics;
            return Ok(false);
        }

        if self.state.show_diagnostics {
            match key.code {
                KeyCode::Esc => {
                    self.state.show_diagnostics = false;
                    return Ok(false);
                }
                KeyCode::Char('h') | KeyCode::Char('H') => {
                    self.state.debug_hitboxes = !self.state.debug_hitboxes;
                    return Ok(false);
                }
                _ => {}
            }
        }

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

        // ── Plugin Center & Marketplace ──────────────────────────────────
        if matches!(self.state.mode, AppMode::PluginCenter { .. }) {
            return self.handle_plugin_center_key(key).await;
        }

        // ── Global shortcuts (always active) ──────────────────────────────
        // Ctrl+, or F10: Interactive Settings Studio
        if (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char(','))
            || (key.modifiers == KeyModifiers::NONE && key.code == KeyCode::F(10))
        {
            self.state.mode = AppMode::SettingsStudio {
                section_idx: 0,
                item_idx: 0,
                in_items: false,
            };
            return Ok(false);
        }

        // Ctrl+G: Git Dashboard
        if key.modifiers == KeyModifiers::CONTROL
            && (key.code == KeyCode::Char('g') || key.code == KeyCode::Char('G'))
        {
            self.state.git_status = GitRepoStatus::query(&self.state.explorer.current_dir);
            self.state.mode = AppMode::GitDashboard { selected_idx: 0 };
            return Ok(false);
        }

        // Ctrl+M: Telemetry Monitor
        if key.modifiers == KeyModifiers::CONTROL
            && (key.code == KeyCode::Char('m') || key.code == KeyCode::Char('M'))
        {
            self.toggle_monitor();
            return Ok(false);
        }

        // F1 or ? (in non-text-entry modes): Shortcuts Reference Modal
        if key.code == KeyCode::F(1)
            || (key.modifiers == KeyModifiers::NONE && key.code == KeyCode::Char('?'))
        {
            if matches!(self.state.mode, AppMode::Help) {
                self.state.mode = AppMode::Browse;
            } else {
                self.state.mode = AppMode::Help;
            }
            return Ok(false);
        }

        // Ctrl+Shift+X: Plugin Marketplace / Center
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && key.modifiers.contains(KeyModifiers::SHIFT)
            && (key.code == KeyCode::Char('X') || key.code == KeyCode::Char('x'))
        {
            self.state.mode = AppMode::PluginCenter {
                tab: 0,
                selected_idx: 0,
                query: String::new(),
                category_idx: 0,
                confirm_install: None,
                inspecting_logs: false,
            };
            return Ok(false);
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

        // Alt+F6 or Ctrl+Shift+T: toggle terminal maximize / normal / minimize
        if (key.modifiers == KeyModifiers::ALT && key.code == KeyCode::F(6))
            || (key.modifiers.contains(KeyModifiers::CONTROL)
                && key.modifiers.contains(KeyModifiers::SHIFT)
                && (key.code == KeyCode::Char('T') || key.code == KeyCode::Char('t')))
        {
            self.toggle_terminal_maximize();
            return Ok(false);
        }

        // F11 or Z (in browse mode): toggle fullscreen preview
        if key.code == KeyCode::F(11)
            || (matches!(self.state.mode, AppMode::Browse)
                && key.modifiers == KeyModifiers::NONE
                && (key.code == KeyCode::Char('z') || key.code == KeyCode::Char('Z')))
        {
            self.toggle_preview_maximize();
            return Ok(false);
        }

        // Panel resizing shortcuts:
        // Ctrl+Left / Alt+[ (and [ when not in Preview): shrink preview width (expand explorer)
        // Ctrl+Right / Alt+] (and ] when not in Preview): expand preview width (shrink explorer)
        let is_preview_shrink = (key.modifiers == KeyModifiers::CONTROL
            && key.code == KeyCode::Left)
            || (key.modifiers == KeyModifiers::ALT && key.code == KeyCode::Char('['))
            || (key.modifiers == KeyModifiers::NONE
                && key.code == KeyCode::Char('[')
                && self.state.focused != FocusedPanel::Preview);
        let is_preview_expand = (key.modifiers == KeyModifiers::CONTROL
            && key.code == KeyCode::Right)
            || (key.modifiers == KeyModifiers::ALT && key.code == KeyCode::Char(']'))
            || (key.modifiers == KeyModifiers::NONE
                && key.code == KeyCode::Char(']')
                && self.state.focused != FocusedPanel::Preview);

        if is_preview_shrink {
            self.state.config.preview_width =
                self.state.config.preview_width.saturating_sub(5).max(15);
            self.state.config.layout.preview_ratio = self.state.config.preview_width as f32 / 100.0;
            self.state.config.layout.explorer_ratio = 1.0 - self.state.config.layout.preview_ratio;
            let _ = self.state.config.save();
            self.state.set_status(
                format!(
                    "Preview: {}% │ Explorer: {}%",
                    self.state.config.preview_width,
                    100 - self.state.config.preview_width
                ),
                StatusKind::Info,
            );
            return Ok(false);
        }
        if is_preview_expand {
            self.state.config.preview_width = (self.state.config.preview_width + 5).min(85);
            self.state.config.layout.preview_ratio = self.state.config.preview_width as f32 / 100.0;
            self.state.config.layout.explorer_ratio = 1.0 - self.state.config.layout.preview_ratio;
            let _ = self.state.config.save();
            self.state.set_status(
                format!(
                    "Preview: {}% │ Explorer: {}%",
                    self.state.config.preview_width,
                    100 - self.state.config.preview_width
                ),
                StatusKind::Info,
            );
            return Ok(false);
        }

        // { / Ctrl+Down / Alt+{ : shrink bottom terminal height
        // } / Ctrl+Up / Alt+} : expand bottom terminal height
        let is_term_shrink = (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Down)
            || (key.modifiers == KeyModifiers::ALT && key.code == KeyCode::Char('{'))
            || (key.modifiers == KeyModifiers::NONE && key.code == KeyCode::Char('{'));
        let is_term_expand = (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Up)
            || (key.modifiers == KeyModifiers::ALT && key.code == KeyCode::Char('}'))
            || (key.modifiers == KeyModifiers::NONE && key.code == KeyCode::Char('}'));

        if is_term_shrink {
            self.state.config.terminal_height =
                self.state.config.terminal_height.saturating_sub(2).max(5);
            self.state.config.layout.terminal_height_percent =
                ((self.state.config.terminal_height as f32 / 40.0) * 100.0).clamp(10.0, 50.0)
                    as u16;
            let _ = self.state.config.save();
            self.state.set_status(
                format!(
                    "Terminal height: {} rows",
                    self.state.config.terminal_height
                ),
                StatusKind::Info,
            );
            return Ok(false);
        }
        if is_term_expand {
            self.state.config.terminal_height = (self.state.config.terminal_height + 2).min(40);
            self.state.config.layout.terminal_height_percent =
                ((self.state.config.terminal_height as f32 / 40.0) * 100.0).clamp(10.0, 50.0)
                    as u16;
            let _ = self.state.config.save();
            self.state.set_status(
                format!(
                    "Terminal height: {} rows",
                    self.state.config.terminal_height
                ),
                StatusKind::Info,
            );
            return Ok(false);
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

        // ── Help mode: interactive category switching & scrolling ─────────
        if matches!(self.state.mode, AppMode::Help) {
            match key.code {
                KeyCode::Esc | KeyCode::F(1) | KeyCode::Char('q') | KeyCode::Char('?') => {
                    self.state.mode = AppMode::Browse;
                }
                KeyCode::Tab => {
                    self.state.help_category = (self.state.help_category + 1) % 6;
                    self.state.help_scroll = 0;
                }
                KeyCode::BackTab => {
                    self.state.help_category = (self.state.help_category + 5) % 6;
                    self.state.help_scroll = 0;
                }
                KeyCode::Char(c @ '1'..='6') => {
                    self.state.help_category = (c as usize) - ('1' as usize);
                    self.state.help_scroll = 0;
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    self.state.help_category = self.state.help_category.saturating_sub(1);
                    self.state.help_scroll = 0;
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    if self.state.help_category < 5 {
                        self.state.help_category += 1;
                        self.state.help_scroll = 0;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.state.help_scroll = self.state.help_scroll.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.state.help_scroll = self.state.help_scroll.saturating_add(1);
                }
                KeyCode::PageUp => {
                    self.state.help_scroll = self.state.help_scroll.saturating_sub(6);
                }
                KeyCode::PageDown => {
                    self.state.help_scroll = self.state.help_scroll.saturating_add(6);
                }
                KeyCode::Home => {
                    self.state.help_scroll = 0;
                }
                _ => {}
            }
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
        if (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('`'))
            || (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('t'))
        {
            self.toggle_terminal();
            return Ok(false);
        }
        // Focus escape from terminal: Alt+Up or Alt+k switches back to Explorer
        if (key.modifiers == KeyModifiers::ALT
            && (key.code == KeyCode::Up || key.code == KeyCode::Char('k')))
            || (key.modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT)
                && key.code == KeyCode::Up)
        {
            self.state.focused = FocusedPanel::Explorer;
            return Ok(false);
        }
        // Alt+F6 or Ctrl+Shift+T: toggle terminal maximize state
        if (key.modifiers == KeyModifiers::ALT && key.code == KeyCode::F(6))
            || (key.modifiers.contains(KeyModifiers::CONTROL)
                && key.modifiers.contains(KeyModifiers::SHIFT)
                && (key.code == KeyCode::Char('T') || key.code == KeyCode::Char('t')))
        {
            self.toggle_terminal_maximize();
            return Ok(false);
        }
        // Terminal height resizing from inside terminal: Ctrl+Down / Ctrl+Up
        if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Down {
            self.state.config.terminal_height =
                self.state.config.terminal_height.saturating_sub(2).max(5);
            let _ = self.state.config.save();
            return Ok(false);
        }
        if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Up {
            self.state.config.terminal_height = (self.state.config.terminal_height + 2).min(40);
            let _ = self.state.config.save();
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
            (KeyModifiers::CONTROL | KeyModifiers::SHIFT, KeyCode::Char('S'))
            | (KeyModifiers::ALT, KeyCode::Char('s')) => match buf.save_force() {
                Ok(_) => self
                    .state
                    .set_status("File force saved (overwritten)", StatusKind::Success),
                Err(e) => self
                    .state
                    .set_status(format!("Save error: {e}"), StatusKind::Error),
            },
            (KeyModifiers::CONTROL, KeyCode::Char('r')) => {
                if let Some(path) = buf.path.clone() {
                    match Buffer::from_file(path) {
                        Ok(new_buf) => {
                            self.editor_buf = Some(new_buf);
                            self.state
                                .set_status("File reloaded from disk", StatusKind::Info);
                        }
                        Err(e) => {
                            self.state
                                .set_status(format!("Reload error: {e}"), StatusKind::Error);
                        }
                    }
                }
            }
            (KeyModifiers::CONTROL, KeyCode::Char('z')) => buf.undo(),
            (KeyModifiers::CONTROL, KeyCode::Char('y')) => buf.redo(),
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => buf.copy_line(),
            (KeyModifiers::CONTROL, KeyCode::Char('x')) => buf.cut_line(),
            (KeyModifiers::CONTROL, KeyCode::Char('v')) => buf.paste(),
            (KeyModifiers::CONTROL, KeyCode::Char('f')) => {
                self.state.mode = AppMode::Modal(ModalKind::Message("Search: ".to_string()));
            }
            (KeyModifiers::NONE, KeyCode::F(3)) => buf.search_next(),
            (KeyModifiers::SHIFT, KeyCode::F(3)) => buf.search_prev(),
            (KeyModifiers::CONTROL, KeyCode::Char('/') | KeyCode::Char('_')) => {
                buf.toggle_comment()
            }
            (KeyModifiers::CONTROL, KeyCode::Char(' ') | KeyCode::Null) => {
                buf.update_completion();
            }
            (KeyModifiers::NONE, KeyCode::F(2)) => {
                if buf.dirty {
                    self.state.mode = AppMode::Modal(ModalKind::Message(
                        "Unsaved changes. Press Esc/F2 again to discard, Ctrl+S to save."
                            .to_string(),
                    ));
                } else {
                    self.state.mode = AppMode::Browse;
                    self.state.focused = FocusedPanel::Explorer;
                }
            }
            (KeyModifiers::ALT, KeyCode::Left) | (KeyModifiers::CONTROL, KeyCode::Left) => {
                buf.move_word_left();
            }
            (KeyModifiers::ALT, KeyCode::Right) | (KeyModifiers::CONTROL, KeyCode::Right) => {
                buf.move_word_right();
            }
            (KeyModifiers::ALT, KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Up) => {
                buf.move_paragraph_prev();
            }
            (KeyModifiers::ALT, KeyCode::Down) | (KeyModifiers::CONTROL, KeyCode::Down) => {
                buf.move_paragraph_next();
            }
            (KeyModifiers::ALT, KeyCode::Backspace)
            | (KeyModifiers::CONTROL, KeyCode::Backspace) => {
                buf.delete_word_backward();
                buf.update_completion();
            }
            (KeyModifiers::ALT, KeyCode::Delete) | (KeyModifiers::CONTROL, KeyCode::Delete) => {
                buf.delete_word_forward();
                buf.update_completion();
            }
            (KeyModifiers::CONTROL, KeyCode::Char('a')) => {
                buf.select_all();
            }
            (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
                buf.select_line();
            }
            (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
                buf.select_word_at_cursor();
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

        // Update scroll so cursor stays visible in viewport.
        if let Some(buf) = &mut self.editor_buf {
            let (visible_h, visible_w) = if let Some(rect) = self.layout.preview {
                (
                    rect.height.saturating_sub(2).max(10) as usize,
                    rect.width.saturating_sub(6).max(20) as usize,
                )
            } else {
                (30, 80)
            };
            buf.scroll_to_cursor(visible_h, visible_w);
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
                            self.fs_ledger.record(RecordedOperation::Rename {
                                from: src.clone(),
                                to: dst.clone(),
                            });
                            self.state.set_status(
                                format!("Renamed → {} (Press 'u' to undo)", input),
                                StatusKind::Success,
                            );
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
                        match operations::trash_entry_tracked(&path) {
                            Ok(trash_filename) => {
                                self.fs_ledger.record(RecordedOperation::Trash {
                                    original_path: path.clone(),
                                    trash_filename,
                                });
                                self.state.set_status(
                                    format!(
                                        "Moved to trash: {} (Press 'u' to undo)",
                                        path.display()
                                    ),
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
                    self.state.set_status("Trash cancelled", StatusKind::Info);
                }
            }
            ModalKind::DeletePermanent { path, .. } => {
                let trimmed = input.trim().to_uppercase();
                if trimmed == "DELETE" || trimmed == "Y" || trimmed == "YES" {
                    match operations::delete_entry(&path) {
                        Ok(_) => {
                            self.state.set_status(
                                format!("Permanently deleted: {}", path.display()),
                                StatusKind::Warning,
                            );
                            self.refresh_dir().await;
                        }
                        Err(e) => self
                            .state
                            .set_status(format!("Permanent delete error: {e}"), StatusKind::Error),
                    }
                } else {
                    self.state
                        .set_status("Permanent delete cancelled", StatusKind::Info);
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
            KeyCode::Tab => {
                // Fuzzy path completion on Tab
                if !self.state.search_query.is_empty() {
                    let cur = self.state.explorer.current_dir.clone();
                    let completions = crate::completion::explorer::complete_path(
                        &self.state.search_query,
                        &cur,
                        5,
                    );
                    if let Some(first) = completions.first() {
                        self.state.search_query = first.completed_path.clone();
                    }
                }
            }
            KeyCode::Char(c)
                if key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT =>
            {
                self.state.search_query.push(c);
            }
            KeyCode::Enter => {
                let q = self.state.search_query.trim();
                if !q.is_empty() {
                    let expanded = shellexpand::tilde(q).to_string();
                    let target_path = if std::path::Path::new(&expanded).is_absolute() {
                        PathBuf::from(&expanded)
                    } else {
                        self.state.explorer.current_dir.join(&expanded)
                    };
                    if target_path.is_dir() {
                        self.navigate_into(target_path).await;
                        self.state.search_query.clear();
                    }
                }
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
            (KeyModifiers::NONE, KeyCode::F(4)) | (KeyModifiers::NONE, KeyCode::Char('a')) => {
                self.state.mode = AppMode::Modal(ModalKind::NewFile);
                self.state.modal_input.clear();
            }
            (KeyModifiers::NONE, KeyCode::F(5)) => {
                self.refresh_dir().await;
            }
            (KeyModifiers::NONE, KeyCode::F(7))
            | (KeyModifiers::SHIFT, KeyCode::Char('A'))
            | (KeyModifiers::NONE, KeyCode::Char('A')) => {
                self.state.mode = AppMode::Modal(ModalKind::NewDir);
                self.state.modal_input.clear();
            }
            (KeyModifiers::NONE, KeyCode::F(8))
            | (KeyModifiers::NONE, KeyCode::Char('d'))
            | (KeyModifiers::NONE, KeyCode::Delete) => {
                if let Some(path) = self.state.selected_path() {
                    self.state.mode = AppMode::Modal(ModalKind::Delete);
                    self.state.modal_input.clear();
                    self.state.set_status(
                        format!("Move {} to trash? Type y to confirm.", path.display()),
                        StatusKind::Warning,
                    );
                }
            }
            (KeyModifiers::SHIFT, KeyCode::Char('D')) | (KeyModifiers::SHIFT, KeyCode::Delete) => {
                if let Some(path) = self.state.selected_path() {
                    let is_dir = path.is_dir();
                    let item_count = if is_dir {
                        std::fs::read_dir(&path).map(|rd| rd.count()).unwrap_or(0)
                    } else {
                        1
                    };
                    self.state.mode = AppMode::Modal(ModalKind::DeletePermanent {
                        path: path.clone(),
                        is_dir,
                        item_count,
                    });
                    self.state.modal_input.clear();
                    self.state.set_status(
                        format!(
                            "PERMANENT DELETE: {} (Type 'DELETE' to confirm)",
                            path.display()
                        ),
                        StatusKind::Warning,
                    );
                }
            }
            (KeyModifiers::NONE, KeyCode::Char('u')) => {
                if self.fs_ledger.can_undo() {
                    match self.fs_ledger.undo() {
                        Ok(msg) => {
                            self.state.set_status(msg, StatusKind::Success);
                            self.refresh_dir().await;
                        }
                        Err(e) => {
                            self.state
                                .set_status(format!("Undo failed: {e}"), StatusKind::Error);
                        }
                    }
                } else {
                    self.state.set_status("Nothing to undo", StatusKind::Info);
                }
            }
            (KeyModifiers::NONE, KeyCode::Char('g')) => {
                self.state.git_status = GitRepoStatus::query(&self.state.explorer.current_dir);
                self.state.mode = AppMode::GitDashboard { selected_idx: 0 };
            }
            (KeyModifiers::NONE, KeyCode::Char('?')) => {
                self.state.mode = AppMode::Help;
            }
            (KeyModifiers::NONE, KeyCode::Char(' ')) => {
                self.state.focused = FocusedPanel::Preview;
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
                                        self.fs_ledger.record(RecordedOperation::Move {
                                            from: src.clone(),
                                            to: dst.clone(),
                                            was_cross_device: false,
                                        });
                                        self.state.clipboard_entry = None;
                                    } else {
                                        self.fs_ledger.record(RecordedOperation::Copy {
                                            src: src.clone(),
                                            dest: dst.clone(),
                                        });
                                    }
                                    self.refresh_dir().await;
                                    self.state.set_status(
                                        format!(
                                            "Pasted '{}' (Press 'u' to undo)",
                                            filename.to_string_lossy()
                                        ),
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
                if max > 0 {
                    self.state.preview.scroll_offset =
                        (self.state.preview.scroll_offset + 20).min(max);
                }
            }
            KeyCode::Home => {
                self.state.preview.scroll_offset = 0;
                self.state.preview.horizontal_offset = 0;
            }
            KeyCode::End => {
                let max = self.state.preview.total_lines.saturating_sub(1);
                self.state.preview.scroll_offset = max;
            }
            KeyCode::Char('H') => {
                self.state.preview.horizontal_offset =
                    self.state.preview.horizontal_offset.saturating_sub(4);
            }
            KeyCode::Char('L') => {
                self.state.preview.horizontal_offset =
                    self.state.preview.horizontal_offset.saturating_add(4);
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
            KeyCode::BackTab | KeyCode::Char('[') => {
                self.state.preview.active_tab = match self.state.preview.active_tab {
                    PreviewTab::Preview => PreviewTab::Actions,
                    PreviewTab::Information => PreviewTab::Preview,
                    PreviewTab::Hex => PreviewTab::Information,
                    PreviewTab::Actions => PreviewTab::Hex,
                };
                self.update_preview().await;
            }
            KeyCode::Tab | KeyCode::Char(']') => {
                self.state.preview.active_tab = match self.state.preview.active_tab {
                    PreviewTab::Preview => PreviewTab::Information,
                    PreviewTab::Information => PreviewTab::Hex,
                    PreviewTab::Hex => PreviewTab::Actions,
                    PreviewTab::Actions => PreviewTab::Preview,
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
            KeyCode::Right | KeyCode::Char('l') => {
                self.state.preview.active_tab = match self.state.preview.active_tab {
                    PreviewTab::Preview => PreviewTab::Information,
                    PreviewTab::Information => PreviewTab::Hex,
                    PreviewTab::Hex => PreviewTab::Actions,
                    PreviewTab::Actions => PreviewTab::Preview,
                };
                self.update_preview().await;
            }
            KeyCode::Char('1') => {
                self.state.preview.active_tab = PreviewTab::Preview;
                self.update_preview().await;
            }
            KeyCode::Char('2') => {
                self.state.preview.active_tab = PreviewTab::Information;
                self.update_preview().await;
            }
            KeyCode::Char('3') => {
                self.state.preview.active_tab = PreviewTab::Hex;
                self.update_preview().await;
            }
            KeyCode::Char('4') => {
                self.state.preview.active_tab = PreviewTab::Actions;
                self.update_preview().await;
            }
            KeyCode::Char(c @ '5'..='9')
                if self.state.preview.active_tab == PreviewTab::Actions =>
            {
                let idx = (c as u8 - b'1') as usize;
                self.execute_preview_action(idx).await;
            }
            KeyCode::Enter if self.state.preview.active_tab == PreviewTab::Actions => {
                self.execute_preview_action(0).await;
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
            KeyCode::Esc | KeyCode::F(10) | KeyCode::Char('q') => {
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
                                    self.state.config.preview_width.saturating_sub(5).max(15);
                            } else {
                                self.state.config.preview_width =
                                    (self.state.config.preview_width + 5).min(85);
                            }
                            self.state.config.layout.preview_ratio =
                                self.state.config.preview_width as f32 / 100.0;
                            self.state.config.layout.explorer_ratio =
                                1.0 - self.state.config.layout.preview_ratio;
                        }
                        "terminal_height" => {
                            if key.code == KeyCode::Left {
                                self.state.config.terminal_height =
                                    self.state.config.terminal_height.saturating_sub(1).max(5);
                            } else {
                                self.state.config.terminal_height =
                                    (self.state.config.terminal_height + 1).min(40);
                            }
                            self.state.config.layout.terminal_height_percent =
                                ((self.state.config.terminal_height as f32 / 40.0) * 100.0)
                                    .clamp(10.0, 50.0) as u16;
                        }
                        "plugin_marketplace" => {
                            self.state.mode = AppMode::PluginCenter {
                                tab: 0,
                                selected_idx: 0,
                                query: String::new(),
                                category_idx: 0,
                                confirm_install: None,
                                inspecting_logs: false,
                            };
                            return Ok(false);
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
                let items = CommandRegistry::search_with_context(
                    &query,
                    &self.state.config.pinned_folders,
                    &self.state.recent_dirs.dirs,
                    self.state.focused,
                );
                if let Some(cmd) = items.get(selected_idx) {
                    let id = cmd.id.to_string();
                    self.state.mode = AppMode::Browse;
                    self.execute_palette_command(&id).await;
                    return Ok(false);
                }
                self.state.mode = AppMode::Browse;
                return Ok(false);
            }
            KeyCode::Up | KeyCode::BackTab => {
                selected_idx = selected_idx.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Tab => {
                let count = CommandRegistry::search_with_context(
                    &query,
                    &self.state.config.pinned_folders,
                    &self.state.recent_dirs.dirs,
                    self.state.focused,
                )
                .len();
                if selected_idx + 1 < count {
                    selected_idx += 1;
                }
            }
            KeyCode::Backspace => {
                query.pop();
                selected_idx = 0;
            }
            KeyCode::Char(c)
                if key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT =>
            {
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
            "terminal_increase_height" => {
                self.state.config.terminal_height = (self.state.config.terminal_height + 2).min(30);
                let _ = self.state.config.save();
                self.state.set_status(
                    format!(
                        "Terminal height: {} rows",
                        self.state.config.terminal_height
                    ),
                    StatusKind::Info,
                );
            }
            "terminal_decrease_height" => {
                self.state.config.terminal_height =
                    self.state.config.terminal_height.saturating_sub(2).max(5);
                let _ = self.state.config.save();
                self.state.set_status(
                    format!(
                        "Terminal height: {} rows",
                        self.state.config.terminal_height
                    ),
                    StatusKind::Info,
                );
            }
            "terminal_maximize" => self.maximize_terminal(),
            "terminal_minimize" => self.minimize_terminal(),
            "terminal_restore" => self.restore_terminal(),
            "focus_terminal" => {
                if !self.state.show_terminal {
                    self.toggle_terminal();
                }
                self.state.focused = FocusedPanel::Terminal;
            }
            "open_plugins" => {
                self.state.mode = AppMode::PluginCenter {
                    tab: 0,
                    selected_idx: 0,
                    query: String::new(),
                    category_idx: 0,
                    confirm_install: None,
                    inspecting_logs: false,
                };
            }
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
            id if id.starts_with("nav_dir:") => {
                let path_str = &id["nav_dir:".len()..];
                let p = PathBuf::from(path_str);
                if p.is_dir() {
                    self.navigate_into(p).await;
                    self.state
                        .set_status(format!("Navigated to {}", path_str), StatusKind::Success);
                } else {
                    self.state.set_status(
                        format!("Directory not found: {}", path_str),
                        StatusKind::Warning,
                    );
                }
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
            KeyCode::Esc | KeyCode::Char('q') => {
                self.state.mode = AppMode::Browse;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                selected_idx = selected_idx.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
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

    fn visible_panels(&self) -> Vec<FocusedPanel> {
        let mut list = Vec::new();
        if self.state.show_sidebar {
            list.push(FocusedPanel::Sidebar);
        }
        list.push(FocusedPanel::Explorer);
        if self.state.dual_pane_enabled {
            list.push(FocusedPanel::ExplorerSecondary);
        }
        if self.state.show_preview {
            list.push(FocusedPanel::Preview);
        }
        if self.state.show_terminal {
            list.push(FocusedPanel::Terminal);
        } else if self.state.show_monitor {
            list.push(FocusedPanel::Monitor);
        }
        list
    }

    fn cycle_focus(&mut self) {
        let panels = self.visible_panels();
        if panels.is_empty() {
            return;
        }
        let current_pos = panels
            .iter()
            .position(|&p| p == self.state.focused)
            .unwrap_or(0);
        let next_pos = (current_pos + 1) % panels.len();
        self.state.focused = panels[next_pos];
    }

    fn cycle_focus_back(&mut self) {
        let panels = self.visible_panels();
        if panels.is_empty() {
            return;
        }
        let current_pos = panels
            .iter()
            .position(|&p| p == self.state.focused)
            .unwrap_or(0);
        let prev_pos = if current_pos == 0 {
            panels.len() - 1
        } else {
            current_pos - 1
        };
        self.state.focused = panels[prev_pos];
    }

    // ── Toggle helpers ────────────────────────────────────────────────────────

    fn toggle_terminal_maximize(&mut self) {
        if !self.state.show_terminal {
            self.state.show_terminal = true;
            self.state.terminal_maximized = false;
            self.start_terminal_session();
            self.state.focused = FocusedPanel::Terminal;
            self.state
                .set_status("Terminal: Normal (~25%)", StatusKind::Info);
        } else if !self.state.terminal_maximized {
            self.state.terminal_maximized = true;
            self.state.focused = FocusedPanel::Terminal;
            self.state
                .set_status("Terminal: Maximized (~80%)", StatusKind::Info);
        } else {
            self.state.terminal_maximized = false;
            self.state.show_terminal = false;
            self.state.focused = FocusedPanel::Explorer;
            self.state.set_status("Terminal: Hidden", StatusKind::Info);
        }
    }

    fn toggle_preview_maximize(&mut self) {
        if !self.state.show_preview {
            self.state.show_preview = true;
            self.state.preview_maximized = true;
            self.state.focused = FocusedPanel::Preview;
            self.state
                .set_status("Preview: Fullscreen (100%)", StatusKind::Info);
        } else if !self.state.preview_maximized {
            self.state.preview_maximized = true;
            self.state.focused = FocusedPanel::Preview;
            self.state
                .set_status("Preview: Fullscreen (100%)", StatusKind::Info);
        } else {
            self.state.preview_maximized = false;
            self.state.focused = FocusedPanel::Explorer;
            self.state.set_status(
                format!("Preview: Split ({}%)", self.state.config.preview_width),
                StatusKind::Info,
            );
        }
    }

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

    fn maximize_terminal(&mut self) {
        if !self.state.show_terminal {
            self.state.show_terminal = true;
            self.start_terminal_session();
        }
        self.state.terminal_maximized = true;
        self.state.focused = FocusedPanel::Terminal;
        self.state
            .set_status("Terminal: Maximized (~80%)", StatusKind::Info);
    }

    fn minimize_terminal(&mut self) {
        if self.state.terminal_maximized {
            self.state.terminal_maximized = false;
            self.state
                .set_status("Terminal: Restored", StatusKind::Info);
        } else {
            self.state.config.terminal_height = 6;
            let _ = self.state.config.save();
            self.state
                .set_status("Terminal: Minimized (6 rows)", StatusKind::Info);
        }
    }

    fn restore_terminal(&mut self) {
        self.state.terminal_maximized = false;
        self.state.show_terminal = true;
        self.state.config.terminal_height = 8;
        let _ = self.state.config.save();
        self.state
            .set_status("Terminal: Restored (8 rows)", StatusKind::Info);
    }

    async fn handle_plugin_center_key(&mut self, key: KeyEvent) -> Result<bool> {
        let (mut tab, mut selected_idx, mut query, mut category_idx, confirm_install) =
            match &self.state.mode {
                AppMode::PluginCenter {
                    tab,
                    selected_idx,
                    query,
                    category_idx,
                    confirm_install,
                    ..
                } => (
                    *tab,
                    *selected_idx,
                    query.clone(),
                    *category_idx,
                    confirm_install.clone(),
                ),
                _ => return Ok(false),
            };

        // Confirmation modal active
        if let Some(target_id) = confirm_install {
            match key.code {
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.state.mode = AppMode::PluginCenter {
                        tab,
                        selected_idx,
                        query,
                        category_idx,
                        confirm_install: None,
                        inspecting_logs: false,
                    };
                    self.state
                        .set_status("Extension installation cancelled", StatusKind::Info);
                }
                KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                    let catalog_item = crate::ui::plugin_center::MARKETPLACE_CATALOG
                        .iter()
                        .find(|p| p.id == target_id);
                    if let Some(item) = catalog_item {
                        let user_plugins = crate::plugins::PluginManager::user_plugins_dir();
                        let plugin_dir = user_plugins.join(item.id);
                        let _ = std::fs::create_dir_all(&plugin_dir);
                        let manifest = crate::plugins::PluginManifest {
                            id: item.id.to_string(),
                            name: item.name.to_string(),
                            version: item.version.to_string(),
                            author: Some(item.author.to_string()),
                            description: Some(item.description.to_string()),
                            entrypoint: "plugin.wasm".to_string(),
                            permissions: item.permissions.iter().copied().collect(),
                            sha256: Some(item.sha256.to_string()),
                        };
                        let manifest_content =
                            toml::to_string_pretty(&manifest).unwrap_or_default();
                        let _ = std::fs::write(plugin_dir.join("plugin.toml"), manifest_content);

                        if !self
                            .state
                            .plugin_manager
                            .plugins
                            .iter()
                            .any(|p| p.manifest.id == item.id)
                        {
                            self.state
                                .plugin_manager
                                .plugins
                                .push(crate::plugins::LoadedPlugin {
                                    manifest,
                                    state: crate::plugins::PluginState::Active,
                                    path: plugin_dir,
                                    instance: None,
                                });
                        }
                        self.state.set_status(
                            format!("Extension '{}' installed successfully", item.name),
                            StatusKind::Success,
                        );
                    }
                    self.state.mode = AppMode::PluginCenter {
                        tab: 1,
                        selected_idx: self.state.plugin_manager.plugins.len().saturating_sub(1),
                        query: String::new(),
                        category_idx: 0,
                        confirm_install: None,
                        inspecting_logs: false,
                    };
                }
                _ => {}
            }
            return Ok(false);
        }

        match key.code {
            KeyCode::Esc => {
                self.state.mode = AppMode::Browse;
                return Ok(false);
            }
            KeyCode::Tab => {
                tab = (tab + 1) % 3;
                selected_idx = 0;
            }
            KeyCode::Char('1') => {
                tab = 0;
                selected_idx = 0;
            }
            KeyCode::Char('2') => {
                tab = 1;
                selected_idx = 0;
            }
            KeyCode::Char('3') => {
                tab = 2;
                selected_idx = 0;
            }
            KeyCode::Char('c') if tab == 0 => {
                category_idx = (category_idx + 1) % crate::ui::plugin_center::CATEGORIES.len();
                selected_idx = 0;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                selected_idx = selected_idx.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max_len = match tab {
                    0 => {
                        let sel_category = crate::ui::plugin_center::CATEGORIES[category_idx];
                        crate::ui::plugin_center::MARKETPLACE_CATALOG
                            .iter()
                            .filter(|p| {
                                (sel_category == "All" || p.category == sel_category)
                                    && (query.is_empty()
                                        || p.name.to_lowercase().contains(&query.to_lowercase())
                                        || p.description
                                            .to_lowercase()
                                            .contains(&query.to_lowercase()))
                            })
                            .count()
                    }
                    1 => self.state.plugin_manager.plugins.len(),
                    _ => 6,
                };
                if selected_idx + 1 < max_len {
                    selected_idx += 1;
                }
            }
            KeyCode::Enter | KeyCode::Char('i') => {
                if tab == 0 {
                    let sel_category = crate::ui::plugin_center::CATEGORIES[category_idx];
                    let filtered: Vec<&crate::ui::plugin_center::MarketplacePlugin> =
                        crate::ui::plugin_center::MARKETPLACE_CATALOG
                            .iter()
                            .filter(|p| {
                                (sel_category == "All" || p.category == sel_category)
                                    && (query.is_empty()
                                        || p.name.to_lowercase().contains(&query.to_lowercase())
                                        || p.description
                                            .to_lowercase()
                                            .contains(&query.to_lowercase()))
                            })
                            .collect();
                    if let Some(&p) = filtered.get(selected_idx) {
                        let is_installed = self
                            .state
                            .plugin_manager
                            .plugins
                            .iter()
                            .any(|lp| lp.manifest.id == p.id);
                        if is_installed {
                            self.state.set_status(
                                format!(
                                    "'{}' is already installed. Switch to tab 2 to manage.",
                                    p.name
                                ),
                                StatusKind::Info,
                            );
                        } else {
                            self.state.mode = AppMode::PluginCenter {
                                tab,
                                selected_idx,
                                query,
                                category_idx,
                                confirm_install: Some(p.id.to_string()),
                                inspecting_logs: false,
                            };
                            return Ok(false);
                        }
                    }
                } else if tab == 1 {
                    let status_info =
                        if let Some(p) = self.state.plugin_manager.plugins.get_mut(selected_idx) {
                            if p.state == crate::plugins::PluginState::Active {
                                p.state = crate::plugins::PluginState::Disabled;
                                p.instance = None;
                                Some((p.manifest.name.clone(), false))
                            } else {
                                p.state = crate::plugins::PluginState::Active;
                                Some((p.manifest.name.clone(), true))
                            }
                        } else {
                            None
                        };
                    if let Some((name, active)) = status_info {
                        if active {
                            self.state.set_status(
                                format!("Extension '{}' activated", name),
                                StatusKind::Success,
                            );
                        } else {
                            self.state.set_status(
                                format!("Extension '{}' disabled", name),
                                StatusKind::Info,
                            );
                        }
                    }
                }
            }
            KeyCode::Char(' ') if tab == 1 => {
                let status_info =
                    if let Some(p) = self.state.plugin_manager.plugins.get_mut(selected_idx) {
                        if p.state == crate::plugins::PluginState::Active {
                            p.state = crate::plugins::PluginState::Disabled;
                            p.instance = None;
                            Some((p.manifest.name.clone(), false))
                        } else {
                            p.state = crate::plugins::PluginState::Active;
                            Some((p.manifest.name.clone(), true))
                        }
                    } else {
                        None
                    };
                if let Some((name, active)) = status_info {
                    if active {
                        self.state.set_status(
                            format!("Extension '{}' activated", name),
                            StatusKind::Success,
                        );
                    } else {
                        self.state
                            .set_status(format!("Extension '{}' disabled", name), StatusKind::Info);
                    }
                }
            }
            KeyCode::Char('d') | KeyCode::Char('u') if tab == 1 => {
                if selected_idx < self.state.plugin_manager.plugins.len() {
                    let removed = self.state.plugin_manager.plugins.remove(selected_idx);
                    let _ = std::fs::remove_dir_all(&removed.path);
                    self.state.set_status(
                        format!("Extension '{}' uninstalled", removed.manifest.name),
                        StatusKind::Success,
                    );
                    selected_idx = selected_idx.saturating_sub(1);
                }
            }
            KeyCode::Char('r') if tab == 1 => {
                let start_dir = self
                    .state
                    .explorer
                    .current_dir
                    .to_string_lossy()
                    .to_string();
                self.state.plugin_manager.discover_and_load(
                    &crate::plugins::PluginManager::user_plugins_dir(),
                    &start_dir,
                );
                self.state
                    .set_status("Extensions reloaded from disk", StatusKind::Success);
            }
            KeyCode::Backspace if tab == 0 => {
                query.pop();
                selected_idx = 0;
            }
            KeyCode::Char(c) if tab == 0 && c != '1' && c != '2' && c != '3' && c != 'c' => {
                query.push(c);
                selected_idx = 0;
            }
            _ => {}
        }

        self.state.mode = AppMode::PluginCenter {
            tab,
            selected_idx,
            query,
            category_idx,
            confirm_install: None,
            inspecting_logs: false,
        };
        Ok(false)
    }

    async fn handle_mouse(&mut self, mouse: MouseEvent) -> Result<()> {
        self.state.last_mouse_pos = Some((mouse.column, mouse.row));

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(action) = self.hit_registry.hit_test(mouse.column, mouse.row) {
                    self.dispatch_action(action).await?;
                    return Ok(());
                }

                // Fallback to panel click routing based on centralized AppLayout
                if self.layout.contains_terminal(mouse.column, mouse.row) {
                    self.state.focused = FocusedPanel::Terminal;
                } else if self.layout.contains_preview(mouse.column, mouse.row) {
                    self.state.focused = FocusedPanel::Preview;
                } else if self.layout.contains_explorer(mouse.column, mouse.row) {
                    self.state.focused = FocusedPanel::Explorer;
                }
            }
            MouseEventKind::ScrollDown => {
                if matches!(self.state.mode, AppMode::Help) {
                    self.state.help_scroll = self.state.help_scroll.saturating_add(2);
                } else if self.layout.contains_preview(mouse.column, mouse.row)
                    || self.state.focused == FocusedPanel::Preview
                {
                    let max = self.state.preview.total_lines.saturating_sub(1);
                    if self.state.preview.scroll_offset < max {
                        self.state.preview.scroll_offset =
                            (self.state.preview.scroll_offset + 3).min(max);
                    }
                } else if (self.layout.contains_explorer(mouse.column, mouse.row)
                    || self.state.focused == FocusedPanel::Explorer)
                    && self.state.explorer.selected_index + 1 < self.state.explorer.entries.len()
                {
                    self.state.explorer.selected_index += 1;
                    self.update_preview().await;
                }
            }
            MouseEventKind::ScrollUp => {
                if matches!(self.state.mode, AppMode::Help) {
                    self.state.help_scroll = self.state.help_scroll.saturating_sub(2);
                } else if self.layout.contains_preview(mouse.column, mouse.row)
                    || self.state.focused == FocusedPanel::Preview
                {
                    self.state.preview.scroll_offset =
                        self.state.preview.scroll_offset.saturating_sub(3);
                } else if self.layout.contains_explorer(mouse.column, mouse.row)
                    || self.state.focused == FocusedPanel::Explorer
                {
                    self.state.explorer.selected_index =
                        self.state.explorer.selected_index.saturating_sub(1);
                    self.update_preview().await;
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub async fn dispatch_action(&mut self, action: AppAction) -> Result<()> {
        match action {
            AppAction::OpenSettings => {
                self.state.mode = AppMode::SettingsStudio {
                    section_idx: 0,
                    item_idx: 0,
                    in_items: false,
                };
            }
            AppAction::OpenPluginCenter => {
                self.state.mode = AppMode::PluginCenter {
                    tab: 0,
                    selected_idx: 0,
                    query: String::new(),
                    category_idx: 0,
                    confirm_install: None,
                    inspecting_logs: false,
                };
            }
            AppAction::OpenSearch => {
                self.state.mode = AppMode::CommandPalette {
                    query: String::new(),
                    selected_idx: 0,
                };
            }
            AppAction::OpenHelp => {
                self.state.mode = AppMode::Help;
            }
            AppAction::OpenGitDashboard => {
                self.state.mode = AppMode::GitDashboard { selected_idx: 0 };
            }
            AppAction::ToggleTerminal => {
                self.toggle_terminal();
            }
            AppAction::MaximizeTerminal => {
                self.toggle_terminal_maximize();
            }
            AppAction::MinimizeTerminal => {
                self.minimize_terminal();
            }
            AppAction::RestoreTerminal => {
                self.state.terminal_maximized = false;
            }
            AppAction::ResizeTerminalCycle => {
                self.state.config.terminal_height = if self.state.config.terminal_height >= 12 {
                    6
                } else {
                    12
                };
                let _ = self.state.config.save();
            }
            AppAction::ClearTerminal => {
                self.terminal_scrollback.clear();
                self.state
                    .set_status("Terminal buffer cleared", StatusKind::Info);
            }
            AppAction::FocusExplorer => {
                self.state.focused = FocusedPanel::Explorer;
            }
            AppAction::FocusPreview => {
                self.state.focused = FocusedPanel::Preview;
            }
            AppAction::FocusTerminal => {
                self.state.focused = FocusedPanel::Terminal;
            }
            AppAction::SelectExplorerIndex(idx) => {
                self.state.focused = FocusedPanel::Explorer;
                if idx < self.state.explorer.entries.len() {
                    self.state.explorer.selected_index = idx;
                    self.update_preview().await;
                }
            }
            AppAction::SelectPreviewTab(idx) => {
                self.state.focused = FocusedPanel::Preview;
                self.state.preview.active_tab = match idx {
                    0 => PreviewTab::Preview,
                    1 => PreviewTab::Information,
                    2 => PreviewTab::Hex,
                    _ => PreviewTab::Actions,
                };
                self.update_preview().await;
            }
            AppAction::SelectPluginCenterTab(tab) => {
                if let AppMode::PluginCenter { tab: t, .. } = &mut self.state.mode {
                    *t = tab;
                }
            }
            AppAction::SelectPluginCenterCategory(cat) => {
                if let AppMode::PluginCenter {
                    category_idx: c, ..
                } = &mut self.state.mode
                {
                    *c = cat;
                }
            }
            AppAction::SelectPluginCenterItem(item) => {
                if let AppMode::PluginCenter {
                    selected_idx: s, ..
                } = &mut self.state.mode
                {
                    *s = item;
                }
            }
            AppAction::SelectSettingsCategory(cat) => {
                if let AppMode::SettingsStudio {
                    section_idx: s,
                    in_items,
                    ..
                } = &mut self.state.mode
                {
                    *s = cat;
                    *in_items = false;
                }
            }
            AppAction::SelectSettingsItem(item) => {
                if let AppMode::SettingsStudio {
                    item_idx: i,
                    in_items,
                    ..
                } = &mut self.state.mode
                {
                    *i = item;
                    *in_items = true;
                }
            }
            AppAction::SelectHelpCategory(cat) => {
                self.state.help_category = cat;
                self.state.help_scroll = 0;
            }
            AppAction::ToggleDiagnostics => {
                self.state.show_diagnostics = !self.state.show_diagnostics;
            }
            AppAction::CloseActiveView => {
                self.state.mode = AppMode::Browse;
                self.state.show_diagnostics = false;
            }
        }
        Ok(())
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

    let (title, prompt, extra_info) = match kind {
        ModalKind::NewFile => ("New File", "Enter filename:", None),
        ModalKind::NewDir => ("New Directory", "Enter directory name:", None),
        ModalKind::Rename => ("Rename", "New name:", None),
        ModalKind::Delete => ("Move to Trash", "Type 'y' to send to Trash:", None),
        ModalKind::DeletePermanent {
            path,
            is_dir,
            item_count,
        } => {
            let info = format!(
                "Target: {}\nContains: {} {} (Permanent, bypasses trash)",
                path.display(),
                item_count,
                if *is_dir { "item(s)" } else { "file" }
            );
            (
                "PERMANENT DELETE WARNING",
                "Type 'DELETE' to confirm destruction:",
                Some(info),
            )
        }
        ModalKind::PinFolder => ("Pin Folder", "Path to pin:", None),
        ModalKind::GoToLine => ("Go to Line", "Line number:", None),
        ModalKind::Message(m) => ("Info", m.as_str(), None),
        ModalKind::Error(m) => ("Error", m.as_str(), None),
    };

    let popup_w = area.width.min(65);
    let popup_h = if extra_info.is_some() { 7u16 } else { 5u16 };
    let popup = Rect {
        x: area.x + (area.width.saturating_sub(popup_w)) / 2,
        y: area.y + (area.height / 2).saturating_sub(popup_h / 2),
        width: popup_w,
        height: popup_h,
    };

    frame.render_widget(Clear, popup);

    let border_style = if matches!(kind, ModalKind::DeletePermanent { .. }) {
        theme.warning_style()
    } else {
        theme.border_focused_style()
    };

    let block = Block::default()
        .title(Span::styled(format!(" {} ", title), theme.header_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style);

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let mut lines = Vec::new();
    if let Some(info) = extra_info {
        for info_line in info.lines() {
            lines.push(Line::from(Span::styled(
                info_line.to_string(),
                theme.warning_style(),
            )));
        }
    }
    lines.push(Line::from(Span::styled(prompt, theme.normal_style())));
    lines.push(Line::from(vec![
        Span::styled("> ", theme.accent_style()),
        Span::styled(input, Style::default().fg(ratatui::style::Color::White)),
        Span::styled("█", theme.accent_style()),
    ]));
    frame.render_widget(Paragraph::new(lines), inner);
}
