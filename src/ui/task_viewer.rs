//! Task Output Workbench UI — live streaming process log viewer.
//!
//! Renders active and completed developer task runs, elapsed execution durations,
//! exit codes, and scrollable stdout/stderr streams with ANSI styling.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::theme::Theme;
use crate::ui::hit_test::{AppAction, HitRegion, HitRegistry, WidgetId};
use crate::workflow::tasks::{TaskExecution, TaskOutputStream, TaskStatus};

pub fn render_task_viewer(
    frame: &mut Frame,
    area: Rect,
    executions: &[TaskExecution],
    selected_idx: usize,
    scroll_offset: usize,
    theme: &Theme,
    hit_reg: &mut HitRegistry,
) {
    frame.render_widget(Clear, area);

    let main_block = Block::default()
        .title(Span::styled(
            " 󰑮 TASK RUNNER WORKBENCH  ─  [F5] Run  │  [Esc] Close ",
            theme.header_style(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(theme.border_focused_style());

    let inner = main_block.inner(area);
    frame.render_widget(main_block, area);

    if inner.width < 30 || inner.height < 6 {
        return;
    }

    // Split horizontally: Left sidebar of executions (30%), Right details & logs (70%)
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(28), Constraint::Percentage(72)])
        .split(inner);

    // ── 1. Executions sidebar ─────────────────────────────────────────────────
    let list_block = Block::default()
        .title(Span::styled(
            " Executions ",
            Style::default().fg(theme.accent),
        ))
        .borders(Borders::RIGHT)
        .border_style(theme.border_style());

    let list_inner = list_block.inner(horizontal_chunks[0]);
    frame.render_widget(list_block, horizontal_chunks[0]);

    if executions.is_empty() {
        let empty_para = Paragraph::new(Line::from(vec![Span::styled(
            " No tasks executed yet.\n Press F5, F6, F7 or Ctrl+Shift+P to run.",
            Style::default().fg(theme.hidden_color),
        )]));
        frame.render_widget(empty_para, list_inner);
    } else {
        let mut list_lines = Vec::new();
        for (idx, exec) in executions.iter().enumerate() {
            let is_sel = idx == selected_idx;
            let status_color = match exec.status {
                TaskStatus::Running => Color::Rgb(250, 179, 135), // Peach
                TaskStatus::Completed(0) => Color::Rgb(166, 227, 161), // Green
                TaskStatus::Failed(_) => Color::Rgb(243, 139, 168), // Red
                TaskStatus::Cancelled => Color::Rgb(147, 153, 178), // Overlay2
                TaskStatus::TimedOut => Color::Rgb(235, 160, 172), // Maroon
                _ => Color::Rgb(203, 166, 247),                   // Mauve
            };

            let prefix = if is_sel { "▶ " } else { "  " };
            let row_style = if is_sel {
                theme.selection_style()
            } else {
                theme.normal_style()
            };

            let dur_sec = exec.duration_ms as f64 / 1000.0;
            list_lines.push(Line::from(vec![
                Span::styled(prefix, Style::default().fg(theme.accent)),
                Span::styled(
                    format!("[{:<7}] ", exec.status.label()),
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(&exec.task_name, row_style),
                Span::styled(
                    format!(" ({:.1}s)", dur_sec),
                    Style::default().fg(theme.size_color),
                ),
            ]));
        }

        let executions_para = Paragraph::new(list_lines);
        frame.render_widget(executions_para, list_inner);
    }

    // ── 2. Right output details ───────────────────────────────────────────────
    let right_area = horizontal_chunks[1];
    let selected_exec = executions.get(selected_idx);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header status banner & action buttons
            Constraint::Min(4),    // Log scroll area
        ])
        .split(right_area);

    // Header banner & buttons
    let banner_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(theme.border_style());

    if let Some(exec) = selected_exec {
        let status_color = match exec.status {
            TaskStatus::Running => Color::Rgb(250, 179, 135),
            TaskStatus::Completed(0) => Color::Rgb(166, 227, 161),
            TaskStatus::Failed(_) => Color::Rgb(243, 139, 168),
            _ => Color::Rgb(147, 153, 178),
        };

        let exit_str = exec
            .exit_code
            .map(|c| format!(" (Exit code: {})", c))
            .unwrap_or_default();

        let header_line = Line::from(vec![
            Span::styled(
                " Task: ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                &exec.task_name,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Status: ", Style::default().fg(theme.accent)),
            Span::styled(
                format!("{}{}", exec.status.label(), exit_str),
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  Duration: {:.2}s", exec.duration_ms as f64 / 1000.0),
                Style::default().fg(theme.size_color),
            ),
            Span::styled(
                "   [R] Rerun",
                Style::default().fg(Color::Rgb(166, 227, 161)),
            ),
            Span::styled(
                "   [C] Cancel",
                Style::default().fg(Color::Rgb(243, 139, 168)),
            ),
        ]);

        let banner_para = Paragraph::new(header_line).block(banner_block);
        frame.render_widget(banner_para, right_chunks[0]);

        // Register interactive hit targets for Rerun & Cancel buttons
        let btn_rerun_rect = Rect {
            x: right_chunks[0].x + right_chunks[0].width.saturating_sub(25),
            y: right_chunks[0].y + 1,
            width: 10,
            height: 1,
        };
        hit_reg.register(HitRegion {
            id: WidgetId::TaskViewerRerun,
            rect: btn_rerun_rect,
            action: AppAction::RerunTask(exec.execution_id),
            enabled: true,
            z_index: 10,
        });

        let btn_cancel_rect = Rect {
            x: right_chunks[0].x + right_chunks[0].width.saturating_sub(13),
            y: right_chunks[0].y + 1,
            width: 11,
            height: 1,
        };
        hit_reg.register(HitRegion {
            id: WidgetId::TaskViewerCancel,
            rect: btn_cancel_rect,
            action: AppAction::CancelTask(exec.execution_id),
            enabled: exec.status.is_active(),
            z_index: 10,
        });

        // Log lines rendering
        let log_area = right_chunks[1];
        let max_visible = log_area.height as usize;
        let total_lines = exec.output_lines.len();

        let start_line = if total_lines > max_visible {
            (total_lines - max_visible).saturating_sub(scroll_offset)
        } else {
            0
        };

        let rendered_lines: Vec<Line> = exec
            .output_lines
            .iter()
            .skip(start_line)
            .take(max_visible)
            .map(|l| {
                let stream_tag = match l.stream {
                    TaskOutputStream::Stdout => {
                        Span::styled(" [out] ", Style::default().fg(Color::Rgb(116, 199, 236)))
                    }
                    TaskOutputStream::Stderr => {
                        Span::styled(" [err] ", Style::default().fg(Color::Rgb(243, 139, 168)))
                    }
                    TaskOutputStream::System => {
                        Span::styled(" [sys] ", Style::default().fg(Color::Rgb(250, 179, 135)))
                    }
                };

                let time_tag = Span::styled(
                    format!("{:>6.2}s ", l.timestamp_ms as f64 / 1000.0),
                    Style::default().fg(theme.hidden_color),
                );

                Line::from(vec![
                    time_tag,
                    stream_tag,
                    Span::styled(&l.text, Style::default().fg(theme.status_fg)),
                ])
            })
            .collect();

        let log_para = Paragraph::new(rendered_lines);
        frame.render_widget(log_para, log_area);
    } else {
        let banner_para = Paragraph::new(Line::from(vec![Span::styled(
            " No execution selected",
            Style::default().fg(theme.hidden_color),
        )]))
        .block(banner_block);
        frame.render_widget(banner_para, right_chunks[0]);
    }
}
