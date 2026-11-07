//! Interactive Settings Studio full-screen UI rendering.

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
            " 󰒓 SETTINGS STUDIO  ─  [F10 / Esc] Exit  [Tab] Switch Pane  [Enter / Space] Toggle  [←/→] Adjust ",
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
        .constraints([Constraint::Length(30), Constraint::Min(40)])
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
            let prefix = if is_sel { "▶ " } else { "  " };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
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
        .title(Span::styled(" Categories ", theme.header_style()))
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

            let prefix = if is_sel { "▶ " } else { "  " };

            let value_span = match &item.value {
                SettingValue::Bool(b) => {
                    if *b {
                        Span::styled(
                            "[✓ Enabled] ",
                            Style::default()
                                .fg(theme.success)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        Span::styled("[✗ Disabled]", Style::default().fg(Color::DarkGray))
                    }
                }
                SettingValue::Choice { current, options } => {
                    let opt_name = options
                        .get(*current)
                        .cloned()
                        .unwrap_or_else(|| "Default".to_string());
                    Span::styled(
                        format!("< {} >", opt_name),
                        Style::default().fg(theme.accent),
                    )
                }
                SettingValue::Number { value, .. } => {
                    Span::styled(format!("< {} >", value), Style::default().fg(theme.accent))
                }
                SettingValue::Text(t) => Span::styled(t.clone(), Style::default().fg(Color::White)),
                SettingValue::ReadOnly(s) => {
                    Span::styled(s.clone(), Style::default().fg(Color::Gray))
                }
            };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, row_style),
                Span::styled(format!("{:<32}", item.label), row_style),
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
            format!(" {} Settings ", active_section.title()),
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
        .title(Span::styled(" Description & Hint ", theme.header_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_style());

    let desc_paragraph = Paragraph::new(format!(
        "  {}\n\n  Press [Space] or [Enter] to toggle or select, [←/→] to adjust values.",
        selected_desc
    ))
    .style(Style::default().fg(Color::Rgb(180, 190, 205)))
    .block(desc_block);

    frame.render_widget(desc_paragraph, desc_area);
}
