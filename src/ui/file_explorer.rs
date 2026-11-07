//! File explorer panel rendering — modern developer workstation aesthetic.

use crate::filesystem::browser::EntryKind;
use crate::state::{AppMode, AppState, FocusedPanel};
use crate::theme::Theme;
use crate::ui::hit_test::{AppAction, HitRegistry, WidgetId};
use bytesize::ByteSize;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

pub fn format_relative_date(dt: chrono::DateTime<chrono::Local>) -> String {
    use chrono::{Datelike, Duration, Local};
    let now = Local::now();
    let today = now.date_naive();
    let dt_date = dt.date_naive();

    if dt_date == today {
        "Today".to_string()
    } else if dt_date == today - Duration::days(1) {
        "Yesterday".to_string()
    } else if now.year() == dt.year() {
        dt.format("%b %d").to_string()
    } else {
        dt.format("%Y-%m-%d").to_string()
    }
}

pub fn render_file_explorer(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: &Theme,
    hit_registry: &mut HitRegistry,
) {
    let focused = state.focused == FocusedPanel::Explorer;
    let border_style = if focused {
        theme.border_focused_style()
    } else {
        theme.border_style()
    };

    hit_registry.register_button(WidgetId::ExplorerPanel, area, AppAction::FocusExplorer, 1);

    let dir_str = state.explorer.current_dir.to_string_lossy();
    let sort_lbl = state.explorer.sort_order.label();
    let hidden_lbl = if state.config.show_hidden {
        " [· Hidden]"
    } else {
        ""
    };
    let clip_lbl = if let Some((path, is_cut)) = &state.clipboard_entry {
        format!(
            " [{} {}]",
            if *is_cut { "✂ cut:" } else { "📋 copy:" },
            path.file_name().unwrap_or_default().to_string_lossy()
        )
    } else {
        String::new()
    };

    let dir_display = if area.width < 50 {
        state
            .explorer
            .current_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| dir_str.to_string())
    } else {
        dir_str.to_string()
    };

    let title = if area.width >= 35 {
        format!(
            " 󰝰 {}  │  󰒺 {}{}{} ",
            dir_display, sort_lbl, hidden_lbl, clip_lbl
        )
    } else {
        format!(" 󰝰 {} ", dir_display)
    };

    let block = Block::default()
        .title(Span::styled(title, theme.header_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 3 || inner.width < 10 {
        return;
    }

    // Filtered entries
    let entries = &state.explorer.entries;
    let query = &state.search_query;

    let filtered_indices: Vec<usize> = if query.is_empty() {
        (0..entries.len()).collect()
    } else {
        crate::filesystem::search::fuzzy_filter(entries, query)
            .into_iter()
            .map(|(_, idx)| idx)
            .collect()
    };

    // Precompute Git file map for current directory entries
    let repo_root = state.git_status.repo_root.as_deref();
    let current_dir = &state.explorer.current_dir;

    let rows: Vec<Row> = filtered_indices
        .iter()
        .map(|&i| {
            let entry = &entries[i];
            let is_selected = i == state.explorer.selected_index;

            let (icon_str, icon_color) = entry.icon_styled(
                state.icon_manager.pack,
                &crate::theme::CatppuccinPalette::mocha(),
            );
            let name_style = if entry.is_hidden {
                Style::default().fg(theme.hidden_color)
            } else {
                match &entry.kind {
                    EntryKind::Directory => Style::default()
                        .fg(theme.dir_color)
                        .add_modifier(Modifier::BOLD),
                    EntryKind::Symlink { broken: true, .. } => Style::default().fg(theme.error),
                    EntryKind::Symlink { .. } => Style::default().fg(theme.symlink_color),
                    _ => Style::default().fg(theme.file_color),
                }
            };

            // Git badge per entry
            let git_badge = if state.git_status.is_repo {
                let entry_path = current_dir.join(&entry.name);
                let rel_path = repo_root
                    .and_then(|r| entry_path.strip_prefix(r).ok())
                    .map(|p| p.to_string_lossy().to_string());

                if let Some(rel) = rel_path {
                    if state.git_status.staged_files.iter().any(|f| f.path == rel) {
                        Some((" ●", theme.success))
                    } else if state
                        .git_status
                        .unstaged_files
                        .iter()
                        .any(|f| f.path == rel)
                    {
                        Some((" ●", theme.warning))
                    } else if state
                        .git_status
                        .untracked_files
                        .iter()
                        .any(|f| f.path == rel)
                    {
                        Some((" ?", theme.accent))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let size_str = match &entry.kind {
                EntryKind::Directory => String::from("--"),
                _ => ByteSize(entry.size).to_string(),
            };

            let modified_str = entry
                .modified
                .map(format_relative_date)
                .unwrap_or_else(|| "--".to_string());

            let perms = &entry.permissions_str;

            let row_style = if is_selected {
                theme.selection_style()
            } else {
                Style::default()
            };

            let mut name_spans = vec![
                Span::styled(icon_str, Style::default().fg(icon_color)),
                Span::styled(&entry.name, name_style),
            ];

            if let Some((badge_char, badge_color)) = git_badge {
                name_spans.push(Span::styled(
                    badge_char,
                    Style::default()
                        .fg(badge_color)
                        .add_modifier(Modifier::BOLD),
                ));
            }

            let show_perms = inner.width >= 95;
            let show_modified = inner.width >= 34;

            let mut cells = vec![
                Cell::from(Line::from(name_spans)),
                Cell::from(size_str).style(Style::default().fg(theme.size_color)),
            ];
            if show_modified {
                cells.push(Cell::from(modified_str).style(Style::default().fg(theme.date_color)));
            }
            if show_perms {
                cells.push(Cell::from(perms.as_str()).style(Style::default().fg(Color::DarkGray)));
            }

            Row::new(cells).style(row_style)
        })
        .collect();

    let show_perms = inner.width >= 95;
    let show_modified = inner.width >= 34;

    let (header, constraints) = if show_perms {
        (
            Row::new(vec![
                Cell::from(" Name").style(theme.header_style()),
                Cell::from("Size").style(theme.header_style()),
                Cell::from("Modified").style(theme.header_style()),
                Cell::from("Perms").style(theme.header_style()),
            ]),
            vec![
                Constraint::Min(24),
                Constraint::Length(10),
                Constraint::Length(12),
                Constraint::Length(10),
            ],
        )
    } else if show_modified {
        (
            Row::new(vec![
                Cell::from(" Name").style(theme.header_style()),
                Cell::from("Size").style(theme.header_style()),
                Cell::from("Modified").style(theme.header_style()),
            ]),
            vec![
                Constraint::Min(14),
                Constraint::Length(8),
                Constraint::Length(10),
            ],
        )
    } else if inner.width >= 24 {
        (
            Row::new(vec![
                Cell::from(" Name").style(theme.header_style()),
                Cell::from("Size").style(theme.header_style()),
            ]),
            vec![Constraint::Min(12), Constraint::Length(8)],
        )
    } else {
        (
            Row::new(vec![
                Cell::from(" Name").style(theme.header_style()),
                Cell::from("Size").style(theme.header_style()),
            ]),
            vec![Constraint::Min(10), Constraint::Length(6)],
        )
    };

    let table = Table::new(rows, constraints)
        .header(header)
        .row_highlight_style(theme.selection_style())
        .highlight_symbol("▎ ");

    let visual_selected = filtered_indices
        .iter()
        .position(|&i| i == state.explorer.selected_index);

    let mut table_state = TableState::default();
    table_state.select(visual_selected);

    frame.render_stateful_widget(table, inner, &mut table_state);

    let offset = table_state.offset();
    let visible_rows = inner.height.saturating_sub(1) as usize;
    for (screen_row, &orig_idx) in filtered_indices
        .iter()
        .skip(offset)
        .take(visible_rows)
        .enumerate()
    {
        let row_rect = Rect {
            x: inner.x,
            y: inner.y + 1 + screen_row as u16,
            width: inner.width,
            height: 1,
        };
        hit_registry.register_button(
            WidgetId::ExplorerRow(orig_idx),
            row_rect,
            AppAction::SelectExplorerIndex(orig_idx),
            2,
        );
    }

    // Search indicator overlay
    if matches!(state.mode, AppMode::Search) && !query.is_empty() {
        let search_area = Rect {
            x: inner.x,
            y: inner.y + inner.height.saturating_sub(1),
            width: inner.width,
            height: 1,
        };
        let search_text = format!(" 󰍉 Filter: {}█ ", query);
        frame.render_widget(
            Paragraph::new(search_text).style(
                Style::default()
                    .fg(theme.accent)
                    .bg(Color::Rgb(25, 30, 40))
                    .add_modifier(Modifier::BOLD),
            ),
            search_area,
        );
    }

    // Loading indicator
    if state.explorer.loading {
        let load_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width.min(20),
            height: 1,
        };
        frame.render_widget(
            Paragraph::new(" ⏳ Loading…").style(Style::default().fg(theme.warning)),
            load_area,
        );
    }
}

pub fn render_secondary_explorer(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let focused = state.focused == FocusedPanel::ExplorerSecondary;
    let border_style = if focused {
        theme.border_focused_style()
    } else {
        theme.border_style()
    };

    let dir_str = state.explorer_secondary.current_dir.to_string_lossy();
    let title = format!(" 󰝰 [Pane 2] {} ", dir_str);

    let block = Block::default()
        .title(Span::styled(title, theme.header_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width < 10 || inner.height < 3 {
        return;
    }

    let entries = &state.explorer_secondary.entries;
    let rows: Vec<Row> = entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_sel = i == state.explorer_secondary.selected_index;
            let (icon_str, icon_color) = entry.icon_styled(
                state.icon_manager.pack,
                &crate::theme::CatppuccinPalette::mocha(),
            );
            let style = if is_sel {
                theme.selection_style()
            } else if entry.kind == EntryKind::Directory {
                Style::default().fg(theme.dir_color)
            } else {
                theme.normal_style()
            };
            let size_str = if entry.kind == EntryKind::Directory {
                "--".to_string()
            } else {
                ByteSize(entry.size).to_string()
            };

            Row::new(vec![
                Cell::from(ratatui::text::Line::from(vec![
                    Span::styled(icon_str, Style::default().fg(icon_color)),
                    Span::styled(&entry.name, style),
                ])),
                Cell::from(size_str).style(Style::default().fg(theme.size_color)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [Constraint::Percentage(70), Constraint::Percentage(30)],
    )
    .highlight_symbol("▎ ");

    let mut table_state = TableState::default();
    table_state.select(Some(state.explorer_secondary.selected_index));
    frame.render_stateful_widget(table, inner, &mut table_state);
}
