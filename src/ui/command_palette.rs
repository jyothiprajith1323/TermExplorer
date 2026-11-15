//! Floating Command Palette UI modal — Raycast and Linear inspired command surface.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::commands::palette::CommandItem;
use crate::theme::Theme;
use crate::workflow::command::{CommandCategory, CommandSearchResult};

pub fn render_command_palette(
    frame: &mut Frame,
    area: Rect,
    query: &str,
    items: &[CommandItem],
    selected_idx: usize,
    theme: &Theme,
) {
    // Legacy compatibility wrapper: converts CommandItem to visual list
    let dlg_width = (area.width * 7 / 10).clamp(55, 96);
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
            " 󰍉 COMMAND PALETTE  ─  [Enter] Execute  │  [Esc] Cancel ",
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

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focused_style());

    let query_display = if query.is_empty() {
        Span::styled(
            " Type to filter commands, tasks, git, settings...",
            Style::default().fg(theme.hidden_color),
        )
    } else {
        Span::styled(
            query,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
    };

    let input_para = Paragraph::new(Line::from(vec![
        Span::styled(
            " 󰍉 > ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        query_display,
        Span::styled("█", Style::default().fg(theme.accent)),
    ]))
    .block(input_block);

    frame.render_widget(input_para, chunks[0]);

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

            let prefix = if is_sel { "▌ " } else { "  " };
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
                Span::styled(format!("{:<38}", cmd.title), row_style),
                Span::styled(shortcut_str, Style::default().fg(theme.size_color)),
            ]))
        })
        .collect();

    frame.render_widget(List::new(list_items), chunks[1]);

    let desc = items
        .get(selected_idx)
        .map(|c| c.description)
        .unwrap_or("Search actions across navigation, files, git, and workspace settings.");

    let desc_para = Paragraph::new(Line::from(vec![
        Span::styled("  󰋽 ", Style::default().fg(theme.accent)),
        Span::styled(desc, Style::default().fg(theme.status_fg)),
    ]));

    frame.render_widget(desc_para, chunks[2]);
}

/// Renders the unified Sprint 4C Developer Workflow Command Palette with
/// categorized badges, fuzzy scores, recency indicators, and availability guards.
pub fn render_workflow_command_palette(
    frame: &mut Frame,
    area: Rect,
    query: &str,
    results: &[CommandSearchResult],
    selected_idx: usize,
    theme: &Theme,
) {
    let dlg_width = (area.width * 7 / 10).clamp(60, 100);
    let dlg_height = 22u16.min(area.height.saturating_sub(4));
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
            " 󰍉 WORKFLOW COMMAND PALETTE  ─  [Enter] Execute  │  [Esc] Dismiss ",
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
            Constraint::Length(3), // Search prompt input
            Constraint::Min(4),    // Filtered command list
            Constraint::Length(3), // Selected item detail
        ])
        .split(inner);

    // ── 1. Search prompt input ───────────────────────────────────────────────
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focused_style());

    let query_display = if query.is_empty() {
        Span::styled(
            " Type to search actions, tasks, git, or settings...",
            Style::default().fg(theme.hidden_color),
        )
    } else {
        Span::styled(
            query,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
    };

    let input_para = Paragraph::new(Line::from(vec![
        Span::styled(
            " 󰍉 > ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        query_display,
        Span::styled("█", Style::default().fg(theme.accent)),
    ]))
    .block(input_block);

    frame.render_widget(input_para, chunks[0]);

    // ── 2. Filtered command list ─────────────────────────────────────────────
    let visible_count = chunks[1].height as usize;
    let offset = if selected_idx >= visible_count {
        selected_idx - visible_count + 1
    } else {
        0
    };

    let list_items: Vec<ListItem> = if results.is_empty() {
        vec![ListItem::new(Line::from(vec![Span::styled(
            "  No matching commands found. Try another query.",
            Style::default().fg(theme.hidden_color),
        )]))]
    } else {
        results
            .iter()
            .enumerate()
            .skip(offset)
            .take(visible_count)
            .map(|(idx, res)| {
                let cmd = &res.command;
                let is_sel = idx == selected_idx;

                let cat_color = match cmd.category {
                    CommandCategory::File | CommandCategory::Navigation => {
                        Color::Rgb(137, 180, 250) // Blue
                    }
                    CommandCategory::Editor => Color::Rgb(249, 226, 175), // Yellow
                    CommandCategory::Preview => Color::Rgb(148, 226, 213), // Teal
                    CommandCategory::Terminal => Color::Rgb(250, 179, 135), // Peach
                    CommandCategory::Git => Color::Rgb(166, 227, 161),    // Green
                    CommandCategory::Tasks => Color::Rgb(203, 166, 247),  // Mauve
                    CommandCategory::Settings => Color::Rgb(116, 199, 236), // Sapphire
                    CommandCategory::Help => Color::Rgb(180, 190, 254),   // Lavender
                };

                let prefix = if is_sel { "▶ " } else { "  " };
                let row_style = if !res.is_available {
                    Style::default().fg(Color::DarkGray)
                } else if is_sel {
                    theme.selection_style()
                } else {
                    theme.normal_style()
                };

                let recent_tag = if res.is_recent { " ★" } else { "" };
                let disabled_tag = if !res.is_available { " [disabled]" } else { "" };

                let shortcut_str = cmd
                    .default_shortcut
                    .as_deref()
                    .map(|s| format!(" [{}]", s))
                    .unwrap_or_default();

                ListItem::new(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(theme.accent)),
                    Span::styled(
                        format!("[{:<8}] ", cmd.category.label()),
                        Style::default().fg(cat_color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!("{:<34}", cmd.display_name), row_style),
                    Span::styled(recent_tag, Style::default().fg(Color::Rgb(249, 226, 175))),
                    Span::styled(disabled_tag, Style::default().fg(Color::DarkGray)),
                    Span::styled(shortcut_str, Style::default().fg(theme.size_color)),
                ]))
            })
            .collect()
    };

    frame.render_widget(List::new(list_items), chunks[1]);

    // ── 3. Selected description ──────────────────────────────────────────────
    let desc = results
        .get(selected_idx)
        .map(|r| r.command.description.as_str())
        .unwrap_or("Search actions across navigation, files, git, and workspace settings.");

    let desc_para = Paragraph::new(Line::from(vec![
        Span::styled("  󰋽 ", Style::default().fg(theme.accent)),
        Span::styled(desc, Style::default().fg(theme.status_fg)),
    ]));

    frame.render_widget(desc_para, chunks[2]);
}
