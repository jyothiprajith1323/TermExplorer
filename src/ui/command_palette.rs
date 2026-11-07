//! Floating Command Palette UI modal rendering.

use crate::commands::palette::CommandItem;
use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub fn render_command_palette(
    frame: &mut Frame,
    area: Rect,
    query: &str,
    items: &[CommandItem],
    selected_idx: usize,
    theme: &Theme,
) {
    // Center a floating dialog
    let dlg_width = (area.width * 7 / 10).clamp(50, 90);
    let dlg_height = 20u16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(dlg_width)) / 2;
    let y = (area.height.saturating_sub(dlg_height)) / 2;

    let dlg_area = Rect {
        x,
        y,
        width: dlg_width,
        height: dlg_height,
    };

    frame.render_widget(Clear, dlg_area);

    let block = Block::default()
        .title(Span::styled(
            " 󰍉 COMMAND PALETTE  ─  [Esc] Cancel  [Enter] Execute ",
            theme.header_style(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(theme.border_focused_style());

    let inner = block.inner(dlg_area);
    frame.render_widget(block, dlg_area);

    if inner.height < 6 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Min(4),    // Results list
            Constraint::Length(3), // Selected description
        ])
        .split(inner);

    // ── 1. Search prompt input ───────────────────────────────────────────────
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focused_style());

    let input_para = Paragraph::new(format!(" > {}█", query))
        .style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .block(input_block);

    frame.render_widget(input_para, chunks[0]);

    // ── 2. Filtered command list ─────────────────────────────────────────────
    let visible_count = chunks[1].height as usize;
    let offset = if selected_idx >= visible_count {
        selected_idx - visible_count + 1
    } else {
        0
    };

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .skip(offset)
        .take(visible_count)
        .map(|(idx, cmd)| {
            let is_sel = idx == selected_idx;
            let row_style = if is_sel {
                theme.selection_style()
            } else {
                theme.normal_style()
            };

            let prefix = if is_sel { "▶ " } else { "  " };
            let shortcut_str = cmd
                .shortcut
                .map(|s| format!(" [{}]", s))
                .unwrap_or_default();

            ListItem::new(Line::from(vec![
                Span::styled(prefix, row_style),
                Span::styled(
                    format!("{:<12} ", cmd.category.badge()),
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("{:<36}", cmd.title), row_style),
                Span::styled(shortcut_str, Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();

    let list_block = Block::default().borders(Borders::NONE);

    frame.render_widget(List::new(list_items).block(list_block), chunks[1]);

    // ── 3. Selected description ──────────────────────────────────────────────
    let desc = items
        .get(selected_idx)
        .map(|c| c.description)
        .unwrap_or("Type to filter actions, workspaces, and themes...");

    let desc_para = Paragraph::new(format!("  󰋼 {}", desc))
        .style(Style::default().fg(Color::Rgb(150, 165, 185)));

    frame.render_widget(desc_para, chunks[2]);
}
