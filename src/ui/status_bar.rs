//! Modern multi-segment workstation status bar.

use crate::state::{AppMode, AppState, FocusedPanel, StatusKind};
use crate::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render_status_bar(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    if area.height < 1 || area.width < 10 {
        return;
    }

    let width = area.width as usize;

    // ── 1. Mode Pill ─────────────────────────────────────────────────────────
    let (mode_str, mode_color) = match &state.mode {
        AppMode::Browse => (" NORMAL ", theme.accent),
        AppMode::Edit => (" EDIT ", theme.success),
        AppMode::Search => (" SEARCH ", theme.warning),
        AppMode::Help => (" HELP ", theme.accent),
        AppMode::Modal(_) => (" DIALOG ", theme.warning),
        AppMode::SettingsStudio { .. } => (" SETTINGS ", Color::Rgb(187, 154, 247)),
        AppMode::CommandPalette { .. } => (" PALETTE ", theme.accent),
        AppMode::GitDashboard { .. } => (" GIT ", Color::Rgb(187, 154, 247)),
        AppMode::Onboarding { .. } => (" SETUP ", theme.accent),
    };

    let mut spans = vec![
        Span::styled(
            mode_str,
            Style::default()
                .fg(Color::Black)
                .bg(mode_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ];

    // ── 2. Focused Panel ─────────────────────────────────────────────────────
    let focused_str = match state.focused {
        FocusedPanel::Sidebar => "SIDEBAR",
        FocusedPanel::Explorer => {
            if state.dual_pane_enabled {
                "EXPLORER [L]"
            } else {
                "EXPLORER"
            }
        }
        FocusedPanel::ExplorerSecondary => "EXPLORER [R]",
        FocusedPanel::Preview => "PREVIEW",
        FocusedPanel::Terminal => "TERMINAL",
        FocusedPanel::Editor => "EDITOR",
        FocusedPanel::Monitor => "MONITOR",
    };

    spans.push(Span::styled(
        format!(" {} ", focused_str),
        Style::default()
            .fg(theme.header_fg)
            .bg(Color::Rgb(35, 42, 54))
            .add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::raw(" "));

    // ── 3. Status Message / Toast Notification (or Position Counter) ──────────
    if let Some((msg, kind)) = &state.status_message {
        let (icon, color) = match kind {
            StatusKind::Error => ("✖", theme.error),
            StatusKind::Warning => ("⚠", theme.warning),
            StatusKind::Success => ("✔", theme.success),
            StatusKind::Info => ("ℹ", theme.accent),
        };
        spans.push(Span::styled(
            format!(" {} {} ", icon, msg),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));
    } else {
        let entry_count = state.active_explorer().entries.len();
        let idx = state.active_explorer().selected_index;
        let pos_str = format!(
            "{}/{}",
            if entry_count > 0 { idx + 1 } else { 0 },
            entry_count
        );
        spans.push(Span::styled(
            format!(" {} ", pos_str),
            Style::default().fg(theme.status_fg),
        ));
    }

    // ── 3b. Active Plugin Status Telemetry ───────────────────────────────────
    if let Some((pname, pstatus)) = state.plugin_manager.first_active_status_text() {
        spans.push(Span::styled(
            format!(" 󰡨 [{}: {}] ", pname, pstatus),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));
    }

    // ── 4. Right: Quick Navigation Keybinding Cheat-sheet ────────────────────
    let right_text = if width >= 120 {
        "? Help │ / Search │ F2 Edit │ F6 Monitor │ F7 Dual │ Ctrl+` Terminal │ F10 Settings"
    } else if width >= 90 {
        "? Help │ / Search │ F2 Edit │ F6 Monitor │ F10 Settings"
    } else if width >= 60 {
        "? Help │ / Search │ F10 Settings"
    } else {
        "? Help"
    };

    let left_len: usize = spans.iter().map(|s| s.content.len()).sum();
    let right_len = right_text.len();

    if left_len + right_len + 2 <= width {
        let pad = width.saturating_sub(left_len + right_len);
        spans.push(Span::raw(" ".repeat(pad)));
        spans.push(Span::styled(
            right_text,
            Style::default().fg(theme.hidden_color),
        ));
    }

    let bg_color = theme.header_bg.unwrap_or(Color::Rgb(16, 20, 26));
    let p = Paragraph::new(Line::from(spans)).style(Style::default().bg(bg_color));
    frame.render_widget(p, area);
}
