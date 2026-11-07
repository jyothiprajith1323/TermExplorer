//! File explorer panel rendering.

use crate::filesystem::browser::EntryKind;
use crate::state::{AppMode, AppState, FocusedPanel};
use crate::theme::Theme;
use bytesize::ByteSize;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Cell, Row, Table, TableState},
    Frame,
};

pub fn render_file_explorer(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let focused = state.focused == FocusedPanel::Explorer;
    let border_style = if focused {
        theme.border_focused_style()
    } else {
        theme.border_style()
    };

    let dir_str = state.explorer.current_dir.to_string_lossy();
    let sort_lbl = state.explorer.sort_order.label();
    let hidden_lbl = if state.config.show_hidden {
        " [Hidden: On]"
    } else {
        ""
    };
    let clip_lbl = if let Some((path, is_cut)) = &state.clipboard_entry {
        format!(
            " [Clip: {} {}]",
            if *is_cut { "cut" } else { "copy" },
            path.file_name().unwrap_or_default().to_string_lossy()
        )
    } else {
        String::new()
    };
    let title = format!(
        " 󰝰  {}  [Sort: {}]{}{} ",
        dir_str, sort_lbl, hidden_lbl, clip_lbl
    );

    let block = Block::default()
        .title(Span::styled(title, theme.header_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 3 {
        return;
    }

    // Show search bar if in search mode.
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

    // Build table rows.
    let rows: Vec<Row> = filtered_indices
        .iter()
        .map(|&i| {
            let entry = &entries[i];
            let is_selected = i == state.explorer.selected_index;

            let icon = entry.icon();
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

            let size_str = match &entry.kind {
                EntryKind::Directory => String::from("--"),
                _ => ByteSize(entry.size).to_string(),
            };

            let modified_str = entry
                .modified
                .map(|m| m.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "--".to_string());

            let perms = entry.permissions_str.clone();

            let row_style = if is_selected {
                theme.selection_style()
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(format!("{} {}", icon, entry.name)).style(name_style),
                Cell::from(size_str).style(Style::default().fg(theme.size_color)),
                Cell::from(modified_str).style(Style::default().fg(theme.date_color)),
                Cell::from(perms).style(Style::default().fg(Color::DarkGray)),
            ])
            .style(row_style)
        })
        .collect();

    let header = Row::new(vec![
        Cell::from("Name").style(theme.header_style()),
        Cell::from("Size").style(theme.header_style()),
        Cell::from("Modified").style(theme.header_style()),
        Cell::from("Perms").style(theme.header_style()),
    ]);

    let table = Table::new(
        rows,
        [
            Constraint::Min(20),
            Constraint::Length(10),
            Constraint::Length(16),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .row_highlight_style(theme.selection_style())
    .highlight_symbol("▶ ");

    // Find the visual selected index within filtered entries.
    let visual_selected = filtered_indices
        .iter()
        .position(|&i| i == state.explorer.selected_index);

    let mut table_state = TableState::default();
    table_state.select(visual_selected);

    frame.render_stateful_widget(table, inner, &mut table_state);

    // Show search indicator.
    if matches!(state.mode, AppMode::Search) && !query.is_empty() {
        // Overlay a small search bar at the bottom of the inner area.
        let search_area = Rect {
            x: inner.x,
            y: inner.y + inner.height.saturating_sub(1),
            width: inner.width,
            height: 1,
        };
        let search_text = format!(" 🔍 Search: {} ", query);
        frame.render_widget(
            ratatui::widgets::Paragraph::new(search_text)
                .style(Style::default().fg(theme.accent).bg(Color::Rgb(30, 30, 40))),
            search_area,
        );
    }

    // Loading indicator.
    if state.explorer.loading {
        let load_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width.min(20),
            height: 1,
        };
        frame.render_widget(
            ratatui::widgets::Paragraph::new(" ⏳ Loading…")
                .style(Style::default().fg(theme.warning)),
            load_area,
        );
    }
}
