//! Workstation Diagnostics & Layout Inspector Overlay (F12).
//!
//! Provides real-time inspection of:
//! - Active App Mode / Focused Panel
//! - Pointer coordinates and hovered widget
//! - Hit testing registry (rectangles, z-index, actions)
//! - Frame duration and rendering latency
//! - Hitbox visualization overlay

use crate::state::AppState;
use crate::theme::Theme;
use crate::ui::hit_test::HitRegistry;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Row, Table},
    Frame,
};

pub fn render_diagnostics(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: &Theme,
    hit_registry: &HitRegistry,
) {
    let popup_w = area.width.clamp(70, 110).min(area.width);
    let popup_h = area.height.clamp(20, 36).min(area.height);
    let popup = Rect {
        x: area.x + (area.width.saturating_sub(popup_w)) / 2,
        y: area.y + (area.height.saturating_sub(popup_h)) / 2,
        width: popup_w,
        height: popup_h,
    };

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .title(Span::styled(
            " 󰘳 Workstation Diagnostics & Layout Inspector (F12) ",
            Style::default()
                .fg(Color::Black)
                .bg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.bg.unwrap_or(Color::Rgb(20, 24, 32))));

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // System & State Overview
            Constraint::Min(8),    // Hit Registry Table
            Constraint::Length(2), // Footer / Controls
        ])
        .split(inner);

    // Overview lines:
    let mouse_info = match state.last_mouse_pos {
        Some((c, r)) => {
            let hit = hit_registry.hit_test_region(c, r);
            let target = match hit {
                Some(reg) => format!("{:?} ({:?})", reg.id, reg.action),
                None => "None (Unmapped area)".to_string(),
            };
            format!("({}, {}) -> Target: {}", c, r, target)
        }
        None => "No mouse events recorded yet".to_string(),
    };

    let overview = vec![
        Line::from(vec![
            Span::styled(
                " Mode: ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:?}", state.mode),
                Style::default().fg(theme.header_fg),
            ),
            Span::raw("   │   "),
            Span::styled(
                "Focused: ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:?}", state.focused),
                Style::default().fg(theme.header_fg),
            ),
            Span::raw("   │   "),
            Span::styled(
                "Frame Latency: ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{} µs", state.last_frame_duration_us),
                Style::default().fg(theme.success),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                " Pointer: ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(mouse_info, Style::default().fg(theme.header_fg)),
        ]),
        Line::from(vec![
            Span::styled(
                " Hit Regions: ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "{} active hitboxes registered in frame",
                    hit_registry.regions().len()
                ),
                Style::default().fg(theme.header_fg),
            ),
            Span::raw("   │   "),
            Span::styled("Hitbox Bounds: ", Style::default().fg(theme.accent)),
            Span::styled(
                if state.debug_hitboxes {
                    "VISIBLE [H to toggle]"
                } else {
                    "HIDDEN [H to toggle]"
                },
                Style::default().fg(if state.debug_hitboxes {
                    theme.success
                } else {
                    theme.size_color
                }),
            ),
        ]),
    ];
    frame.render_widget(Paragraph::new(overview), chunks[0]);

    // Hit Registry Table
    let header_row = Row::new(vec![
        "Widget ID",
        "Bounding Rect (x, y, w, h)",
        "Dispatched Action",
        "Z-Index",
    ])
    .style(
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = hit_registry
        .regions()
        .iter()
        .take(16)
        .map(|r| {
            Row::new(vec![
                format!("{:?}", r.id),
                format!(
                    "({}, {}, {}, {})",
                    r.rect.x, r.rect.y, r.rect.width, r.rect.height
                ),
                format!("{:?}", r.action),
                format!("{}", r.z_index),
            ])
            .style(Style::default().fg(theme.header_fg))
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(25),
            Constraint::Percentage(35),
            Constraint::Percentage(10),
        ],
    )
    .header(header_row)
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(theme.border)),
    );

    frame.render_widget(table, chunks[1]);

    // Footer
    let footer = Line::from(vec![
        Span::styled(
            " [F12 / Esc] ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Close Inspector   "),
        Span::styled(
            " [H] ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Toggle Screen Hitboxes   "),
        Span::styled(
            " [Left Click] ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Test Hit Dispatch"),
    ]);
    frame.render_widget(Paragraph::new(footer), chunks[2]);
}
