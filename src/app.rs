//! Core application struct, event loop, and input handling.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use crate::{
    config::Config,
    editor::{auto_pair, buffer::Buffer},
    events::{start_event_loop, AppEvent},
    filesystem::{
        browser::list_dir,
        metadata::{get_metadata, is_binary},
        operations,
    },
    monitor::{Monitor, MonitorSnapshot},
    shell::{input::key_to_pty_bytes, TerminalSession},
    state::{AppMode, AppState, FocusedPanel, ModalKind, SidebarSection, StatusKind},
    theme::Theme,
    ui::{
        editor::render_editor,
        file_explorer::render_file_explorer,
        help::render_help,
        layout::{compute_layout, LayoutConfig},
        preview::render_preview,
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
        let theme = Theme::dark();
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
                sidebar_pct: state.config.sidebar_width,
                preview_pct: state.config.preview_width,
                bottom_height,
            };

            let areas = compute_layout(area, &layout_cfg);

            // ── Header ────────────────────────────────────────────────────
            {
                use ratatui::{
                    text::{Line, Span},
                    widgets::Paragraph,
                };
                let title = Line::from(vec![
                    Span::styled(" TermExplorer ", theme.accent_style()),
                    Span::styled(
                        format!("  {} ", state.explorer.current_dir.display()),
                        theme.status_style(),
                    ),
                ]);
                frame.render_widget(Paragraph::new(title), areas.header);
            }

            // ── Sidebar ───────────────────────────────────────────────────
            if let Some(sb_area) = areas.sidebar {
                render_sidebar(frame, sb_area, state, theme);
            }

            // ── File explorer ─────────────────────────────────────────────
            render_file_explorer(frame, areas.explorer, state, theme);

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
        let dir = self.state.explorer.current_dir.clone();
        let show_hidden = self.state.config.show_hidden;
        let sort = self.state.explorer.sort_order;

        match list_dir(&dir, show_hidden, sort) {
            Ok(entries) => {
                self.state.explorer.entries = entries;
                self.state.explorer.selected_index = self
                    .state
                    .explorer
                    .selected_index
                    .min(self.state.explorer.entries.len().saturating_sub(1));
                self.update_preview().await;
            }
            Err(e) => {
                self.state
                    .set_status(format!("Cannot read directory: {e}"), StatusKind::Error);
            }
        }
    }

    async fn navigate_into(&mut self, dir: PathBuf) {
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
        self.state
            .set_status(format!("→ {}", dir.display()), StatusKind::Info);
    }

    async fn navigate_up(&mut self) {
        if let Some(parent) = self
            .state
            .explorer
            .current_dir
            .parent()
            .map(|p| p.to_path_buf())
        {
            self.navigate_into(parent).await;
        }
    }

    // ── Preview loading ──────────────────────────────────────────────────────

    async fn update_preview(&mut self) {
        let path = match self.state.selected_path() {
            Some(p) => p,
            None => {
                self.state.preview.path = None;
                return;
            }
        };

        self.state.preview.path = Some(path.clone());

        if path.is_dir() {
            self.state.preview.lines = vec!["  (directory)".to_string()];
            self.state.preview.highlighted_spans.clear();
            self.state.preview.is_binary = false;
            self.state.preview.total_lines = 1;
            return;
        }

        let max_bytes = self.state.config.max_preview_bytes;

        // Read file (with size limit).
        let content = match std::fs::metadata(&path) {
            Ok(meta) if meta.len() > max_bytes => {
                let mut f = match std::fs::File::open(&path) {
                    Ok(f) => f,
                    Err(_) => {
                        self.state.preview.is_binary = false;
                        self.state.preview.lines = vec!["  (unreadable)".to_string()];
                        return;
                    }
                };
                let mut buf = vec![0u8; max_bytes as usize];
                use std::io::Read;
                let _ = f.read(&mut buf);
                buf
            }
            Ok(_) => std::fs::read(&path).unwrap_or_default(),
            Err(_) => {
                self.state.preview.is_binary = false;
                self.state.preview.lines = vec!["  (unreadable)".to_string()];
                return;
            }
        };

        // Check for binary.
        if is_binary(&content) {
            self.state.preview.is_binary = true;
            self.state.preview.highlighted_spans.clear();
            self.state.preview.file_size = content.len() as u64;
            if let Ok(meta) = get_metadata(&path) {
                self.state.preview.mime_type = meta.mime_guess;
                self.state.preview.file_size = meta.size_bytes;
            }
            return;
        }

        self.state.preview.is_binary = false;
        let text = String::from_utf8_lossy(&content).to_string();
        self.state.preview.lines = text.lines().map(|l| l.to_string()).collect();
        self.state.preview.highlighted_spans = crate::editor::syntax::highlight_file(&path, &text);
        self.state.preview.total_lines = self.state.preview.lines.len();
        self.state.preview.scroll_offset = 0;

        if let Ok(meta) = get_metadata(&path) {
            self.state.preview.mime_type = meta.mime_guess;
            self.state.preview.file_size = meta.size_bytes;
        }
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

        // ── Global shortcuts (always active) ──────────────────────────────
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('q')) => return Ok(true),
            (KeyModifiers::CONTROL, KeyCode::Char('b')) => {
                self.state.show_sidebar = !self.state.show_sidebar;
                return Ok(false);
            }
            (KeyModifiers::CONTROL, KeyCode::Char('`')) => {
                self.toggle_terminal();
                return Ok(false);
            }
            _ => {}
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
            KeyCode::Esc => {
                self.state.mode = AppMode::Browse;
                self.state.search_query.clear();
            }
            _ => {}
        }

        // ── Help mode: Esc already handled above ──────────────────────────
        if matches!(self.state.mode, AppMode::Help) {
            return Ok(false);
        }

        // ── Focus switching with Tab / arrow keys ─────────────────────────
        match key.code {
            KeyCode::Tab => self.cycle_focus(),
            KeyCode::BackTab => self.cycle_focus_back(),
            _ => {}
        }

        // ── Panel-specific key handling ───────────────────────────────────
        match self.state.focused {
            FocusedPanel::Sidebar => {
                self.handle_sidebar_key(key).await?;
            }
            FocusedPanel::Explorer => {
                self.handle_explorer_key(key).await?;
            }
            FocusedPanel::Preview => {
                self.handle_preview_key(key);
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
            (KeyModifiers::NONE, KeyCode::Up) => {
                if self.state.explorer.selected_index > 0 {
                    self.state.explorer.selected_index -= 1;
                    self.update_preview().await;
                }
            }
            (KeyModifiers::NONE, KeyCode::Down) => {
                let max = self.state.explorer.entries.len().saturating_sub(1);
                if self.state.explorer.selected_index < max {
                    self.state.explorer.selected_index += 1;
                    self.update_preview().await;
                }
            }
            (KeyModifiers::NONE, KeyCode::Enter) => {
                if let Some(entry) = self.state.selected_entry() {
                    if entry.is_dir() {
                        let dir = self.state.explorer.current_dir.join(&entry.name);
                        self.navigate_into(dir).await;
                    } else {
                        // Open in preview.
                        self.state.focused = FocusedPanel::Preview;
                    }
                }
            }
            (KeyModifiers::NONE, KeyCode::Backspace) => {
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
            (KeyModifiers::NONE, KeyCode::Char('h')) => {
                self.state.config.show_hidden = !self.state.config.show_hidden;
                self.refresh_dir().await;
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
            KeyCode::Enter => {
                let target_dir = match self.state.sidebar.section {
                    SidebarSection::Pinned => self
                        .state
                        .config
                        .pinned_folders
                        .get(self.state.sidebar.selected_index)
                        .map(|p| p.path.clone()),
                    SidebarSection::Shortcuts => shortcuts
                        .get(self.state.sidebar.selected_index)
                        .map(|(_, p)| p.clone()),
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

    fn handle_preview_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                if self.state.preview.scroll_offset > 0 {
                    self.state.preview.scroll_offset -= 1;
                }
            }
            KeyCode::Down => {
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
            _ => {}
        }
    }

    // ── Focus cycling ────────────────────────────────────────────────────────

    fn cycle_focus(&mut self) {
        self.state.focused = match self.state.focused {
            FocusedPanel::Sidebar => FocusedPanel::Explorer,
            FocusedPanel::Explorer => {
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
                } else {
                    FocusedPanel::Explorer
                }
            }
            FocusedPanel::Explorer => FocusedPanel::Sidebar,
            FocusedPanel::Preview => FocusedPanel::Explorer,
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
