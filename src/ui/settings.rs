//! Interactive Settings Studio — modern developer workstation settings interface.

use crate::settings::{get_section_items, SettingValue, SettingsSection};
use crate::state::AppState;
use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub fn render_settings_studio(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: &Theme,
    selected_section_idx: usize,
    selected_item_idx: usize,
    in_items_pane: bool,
) {
    frame.render_widget(Clear, area);

    let outer_block = Block::default()
        .title(Span::styled(
            " 󰒓 SETTINGS STUDIO  ─  [Tab] Switch Pane  │  [Space/Enter] Toggle  │  [←/→] Adjust  │  [F10 / Esc] Exit ",
            theme.header_style(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(theme.border_focused_style());

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    if inner.width < 30 || inner.height < 10 {
        return;
    }

    // Split horizontally: Left navigation (15 sections), Right items + details
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(36), Constraint::Min(40)])
        .split(inner);

    let left_area = horizontal_chunks[0];
    let right_area = horizontal_chunks[1];

    // ── Left: Sections list ───────────────────────────────────────────────────
    let sections = SettingsSection::ALL;
    let section_items: Vec<ListItem> = sections
        .iter()
        .enumerate()
        .map(|(i, sec)| {
            let is_sel = i == selected_section_idx;
            let style = if is_sel && !in_items_pane {
                theme.selection_style()
            } else if is_sel {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                theme.normal_style()
            };
            let prefix = if is_sel { "▌ " } else { "  " };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(
                    format!("{:02} ", i + 1),
                    Style::default().fg(theme.hidden_color),
                ),
                Span::styled(format!("{} ", sec.icon()), style),
                Span::styled(sec.title(), style),
            ]))
        })
        .collect();

    let section_border_style = if !in_items_pane {
        theme.border_focused_style()
    } else {
        theme.border_style()
    };

    let section_block = Block::default()
        .title(Span::styled(" Categories (15) ", theme.header_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(section_border_style);

    frame.render_widget(List::new(section_items).block(section_block), left_area);

    // ── Right: Split into items list (top) and description box (bottom) ──────
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(6)])
        .split(right_area);

    let items_area = right_chunks[0];
    let desc_area = right_chunks[1];

    let active_section = sections[selected_section_idx.min(sections.len() - 1)];
    let items = get_section_items(active_section, &state.config);

    let item_widgets: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_sel = i == selected_item_idx;
            let row_style = if is_sel && in_items_pane {
                theme.selection_style()
            } else if is_sel {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                theme.normal_style()
            };

            let prefix = if is_sel { "▌ " } else { "  " };

            let (type_badge, value_span) = match &item.value {
                SettingValue::Bool(b) => {
                    let badge = Span::styled("[TOGGLE] ", Style::default().fg(theme.hidden_color));
                    let val = if *b {
                        Span::styled(
                            "[ ● ON  ]",
                            Style::default()
                                .fg(Color::Black)
                                .bg(theme.success)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        Span::styled("[ ○ OFF ]", Style::default().fg(theme.hidden_color))
                    };
                    (badge, val)
                }
                SettingValue::Choice { current, options } => {
                    let badge = Span::styled("[SELECT] ", Style::default().fg(theme.hidden_color));
                    let opt_name = options
                        .get(*current)
                        .cloned()
                        .unwrap_or_else(|| "Default".to_string());
                    let val = Span::styled(
                        format!("◀  {}  ▶", opt_name),
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD),
                    );
                    (badge, val)
                }
                SettingValue::Number { value, .. } => {
                    let badge = Span::styled("[SLIDER] ", Style::default().fg(theme.hidden_color));
                    let val = Span::styled(
                        format!("◀  {}  ▶", value),
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD),
                    );
                    (badge, val)
                }
                SettingValue::Text(t) => {
                    let badge = Span::styled("[TEXT]   ", Style::default().fg(theme.hidden_color));
                    let val = Span::styled(format!("\"{}\"", t), Style::default().fg(Color::White));
                    (badge, val)
                }
                SettingValue::ReadOnly(s) => {
                    let badge = Span::styled("[INFO]   ", Style::default().fg(theme.hidden_color));
                    let val = Span::styled(s.clone(), Style::default().fg(theme.size_color));
                    (badge, val)
                }
            };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, row_style),
                type_badge,
                Span::styled(format!("{:<28}", item.label), row_style),
                Span::raw("  "),
                value_span,
            ]))
        })
        .collect();

    let items_border_style = if in_items_pane {
        theme.border_focused_style()
    } else {
        theme.border_style()
    };

    let items_block = Block::default()
        .title(Span::styled(
            format!(
                " {} {} Settings ",
                active_section.icon(),
                active_section.title()
            ),
            theme.header_style(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(items_border_style);

    frame.render_widget(List::new(item_widgets).block(items_block), items_area);

    // ── Description panel ────────────────────────────────────────────────────
    let selected_desc = items
        .get(selected_item_idx)
        .map(|i| i.description)
        .unwrap_or("Select a setting to view details and options.");

    let desc_block = Block::default()
        .title(Span::styled(
            " 󰋽 Description & Shortcut Hints ",
            theme.header_style(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_style());

    let desc_paragraph = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("  Description: ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::raw(selected_desc),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  Hint:        ", Style::default().fg(theme.size_color)),
            Span::styled(
                "Press [Space] or [Enter] to toggle or select, [←/→] to adjust, [Tab] to switch panels.",
                Style::default().fg(theme.hidden_color),
            ),
        ]),
    ])
    .block(desc_block);

    frame.render_widget(desc_paragraph, desc_area);
}
