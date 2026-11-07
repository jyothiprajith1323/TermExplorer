//! Status bar rendering.

use crate::state::{AppMode, AppState, FocusedPanel, StatusKind};
use crate::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render_status_bar(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let left = build_left(state, theme);
    let right = build_right(state);

    let left_width = left.iter().map(|s| s.content.len()).sum::<usize>();
    let right_str = right.join(" │ ");
    let padding = area
        .width
        .saturating_sub(left_width as u16 + right_str.len() as u16 + 2) as usize;

    let mut spans = left;
    spans.push(Span::raw(" ".repeat(padding)));
    spans.push(Span::styled(
        right_str,
        Style::default().fg(Color::DarkGray),
    ));

    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(theme.status_style()),
        area,
    );
}

fn build_left<'a>(state: &'a AppState, theme: &'a Theme) -> Vec<Span<'a>> {
    // Priority: status message > mode indicator
    if let Some((msg, kind)) = &state.status_message {
        let style = match kind {
            StatusKind::Error => theme.error_style(),
            StatusKind::Warning => theme.warning_style(),
            StatusKind::Success => theme.success_style(),
            StatusKind::Info => theme.status_style(),
        };
        return vec![Span::styled(format!(" {} ", msg), style)];
    }

    let mode_str = match &state.mode {
        AppMode::Browse => " BROWSE ",
        AppMode::Edit => " EDIT ",
        AppMode::Search => " SEARCH ",
        AppMode::Help => " HELP ",
        AppMode::Modal(_) => " DIALOG ",
        AppMode::SettingsStudio { .. } => " SETTINGS ",
        AppMode::CommandPalette { .. } => " PALETTE ",
        AppMode::GitDashboard { .. } => " GIT ",
        AppMode::Onboarding { .. } => " SETUP ",
    };

    let focused_str = match state.focused {
        FocusedPanel::Sidebar => "Sidebar",
        FocusedPanel::Explorer => {
            if state.dual_pane_enabled {
                "Explorer (Left)"
            } else {
                "Explorer"
            }
        }
        FocusedPanel::ExplorerSecondary => "Explorer (Right)",
        FocusedPanel::Preview => "Preview",
        FocusedPanel::Terminal => "Terminal",
        FocusedPanel::Editor => "Editor",
        FocusedPanel::Monitor => "Monitor",
    };

    let mut spans = vec![
        Span::styled(mode_str, theme.accent_style()),
        Span::raw(" "),
        Span::styled(focused_str, theme.normal_style()),
        Span::raw(" │ "),
        Span::styled(
            format!("󰲋 {} ", state.workspace.name),
            Style::default().fg(theme.accent),
        ),
        Span::styled(
            format!("({})", state.workspace.project_kind.badge()),
            Style::default().fg(Color::DarkGray),
        ),
    ];

    if state.git_status.is_repo {
        let git_color = if state.git_status.is_dirty {
            theme.warning
        } else {
            theme.success
        };
        spans.push(Span::raw(" │ "));
        spans.push(Span::styled(
            format!(
                "󰊢 {}{}",
                state.git_status.branch,
                if state.git_status.is_dirty { "*" } else { "" }
            ),
            Style::default().fg(git_color),
        ));
    }

    spans
}

fn build_right(state: &AppState) -> Vec<String> {
    let entry_count = state.active_explorer().entries.len();
    let idx = state.active_explorer().selected_index;
    vec![
        "F1 Help │ F2 Edit │ F6 Monitor │ F7 Dual │ F10 Settings │ Ctrl+Shift+P Palette │ Ctrl+Q Quit".to_string(),
        format!("{}/{}", if entry_count > 0 { idx + 1 } else { 0 }, entry_count),
    ]
}
