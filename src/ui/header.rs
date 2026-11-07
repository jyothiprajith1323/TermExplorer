//! Workstation Command Bar — top navigation and workspace header.

use crate::monitor::MonitorSnapshot;
use crate::state::AppState;
use crate::theme::Theme;
use bytesize::ByteSize;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render_header(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    monitor_snap: Option<&MonitorSnapshot>,
    theme: &Theme,
) {
    if area.height < 1 || area.width < 10 {
        return;
    }

    let width = area.width as usize;

    // ── 1. Left Section: Brand & Workspace ────────────────────────────────────
    let mut left_spans = vec![
        Span::styled(
            " ◆ TERMEXPLORER ",
            Style::default()
                .fg(Color::Black)
                .bg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ];

    if width >= 90 {
        left_spans.push(Span::styled(
            format!("󰲋 {} ", state.workspace.name),
            Style::default()
                .fg(theme.header_fg)
                .add_modifier(Modifier::BOLD),
        ));
        left_spans.push(Span::styled(
            format!("[{}] ", state.workspace.project_kind.badge()),
            Style::default().fg(theme.accent),
        ));
        left_spans.push(Span::styled("│ ", Style::default().fg(theme.border)));
    }

    // ── 2. Center Section: Breadcrumbs & Git ──────────────────────────────────
    let mut center_spans = Vec::new();
    let current_dir = &state.explorer.current_dir;
    let home_dir = dirs::home_dir();

    let display_path = if let Some(home) = &home_dir {
        if let Ok(rel) = current_dir.strip_prefix(home) {
            format!("~/{}", rel.display())
        } else {
            current_dir.display().to_string()
        }
    } else {
        current_dir.display().to_string()
    };

    // Breadcrumb formatting with styled separator
    let parts: Vec<&str> = display_path.split('/').filter(|s| !s.is_empty()).collect();
    if width >= 110 && !parts.is_empty() {
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                center_spans.push(Span::styled(" › ", Style::default().fg(theme.hidden_color)));
            }
            let is_last = i + 1 == parts.len();
            if is_last {
                center_spans.push(Span::styled(
                    *part,
                    Style::default()
                        .fg(theme.header_fg)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                center_spans.push(Span::styled(*part, Style::default().fg(theme.size_color)));
            }
        }
    } else {
        let folder_name = current_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "/".to_string());
        center_spans.push(Span::styled(
            format!("󰝰 {} ", folder_name),
            Style::default()
                .fg(theme.header_fg)
                .add_modifier(Modifier::BOLD),
        ));
    }

    // Git branch indicator in header
    if state.git_status.is_repo {
        center_spans.push(Span::styled(" │ ", Style::default().fg(theme.border)));
        let git_color = if state.git_status.is_dirty {
            theme.warning
        } else {
            theme.success
        };
        let dirty_suffix = if state.git_status.is_dirty {
            let total_dirty = state.git_status.staged_files.len()
                + state.git_status.unstaged_files.len()
                + state.git_status.untracked_files.len();
            format!(" ● {}", total_dirty)
        } else {
            " ✓".to_string()
        };
        center_spans.push(Span::styled(
            format!("󰊢 {}{}", state.git_status.branch, dirty_suffix),
            Style::default().fg(git_color).add_modifier(Modifier::BOLD),
        ));
    }

    // ── 3. Right Section: Telemetry & Global Actions ──────────────────────────
    let mut right_spans = Vec::new();

    // CPU / RAM telemetry pill
    if width >= 120 {
        if let Some(snap) = monitor_snap {
            right_spans.push(Span::styled(" │ ", Style::default().fg(theme.border)));
            right_spans.push(Span::styled(
                format!(
                    "󰻠 {:.0}%  󰍛 {} ",
                    snap.cpu.global_usage,
                    ByteSize(snap.mem.used)
                ),
                Style::default().fg(theme.status_fg),
            ));
            right_spans.push(Span::styled("│ ", Style::default().fg(theme.border)));
        }
    }

    if width >= 80 {
        right_spans.push(Span::styled(
            "[󰍉 Ctrl+P] ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));
    }

    right_spans.push(Span::styled(
        "[󰒓 F10]",
        Style::default().fg(theme.size_color),
    ));

    // Calculate total widths and assemble
    let left_len: usize = left_spans.iter().map(|s| s.content.len()).sum();
    let center_len: usize = center_spans.iter().map(|s| s.content.len()).sum();
    let right_len: usize = right_spans.iter().map(|s| s.content.len()).sum();

    let mut line_spans = left_spans;
    if left_len + center_len + right_len + 4 <= width {
        let left_pad = ((width.saturating_sub(center_len)) / 2).saturating_sub(left_len);
        line_spans.push(Span::raw(" ".repeat(left_pad)));
        line_spans.extend(center_spans);
        let current_total: usize = line_spans.iter().map(|s| s.content.len()).sum();
        let right_pad = width.saturating_sub(current_total + right_len);
        line_spans.push(Span::raw(" ".repeat(right_pad)));
        line_spans.extend(right_spans);
    } else {
        line_spans.push(Span::raw(" "));
        line_spans.extend(center_spans);
        let current_total: usize = line_spans.iter().map(|s| s.content.len()).sum();
        let right_pad = width.saturating_sub(current_total + right_len);
        line_spans.push(Span::raw(" ".repeat(right_pad)));
        line_spans.extend(right_spans);
    }

    let header_bg = theme.header_bg.unwrap_or(Color::Rgb(20, 24, 32));
    let paragraph = Paragraph::new(Line::from(line_spans)).style(Style::default().bg(header_bg));
    frame.render_widget(paragraph, area);
}
