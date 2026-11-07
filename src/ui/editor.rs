//! Integrated editor UI rendering.

use crate::editor::Buffer;
use crate::state::AppState;
use crate::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_editor(
    frame: &mut Frame,
    area: Rect,
    buf: &Buffer,
    _state: &AppState,
    theme: &Theme,
) {
    let dirty_marker = if buf.dirty { "●" } else { "" };
    let file_name = buf
        .path
        .as_ref()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "untitled".to_string());

    let title = format!(" ✏  {} {} ", file_name, dirty_marker);

    let block = Block::default()
        .title(Span::styled(title, theme.header_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focused_style());

    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    if inner.height < 2 || inner.width < 4 {
        return;
    }

    let visible_rows = inner.height as usize;
    let line_num_width = format!("{}", buf.line_count().max(1)).len() + 1;

    let lines: Vec<Line> = buf
        .visible_lines(buf.scroll_row, visible_rows)
        .map(|(row_idx, line)| {
            let line_num = format!("{:>width$} ", row_idx + 1, width = line_num_width);
            let is_cursor_row = row_idx == buf.cursor.row;

            // Clip line to scroll_col.
            let display_text = if buf.scroll_col < line.len() {
                &line[buf.scroll_col..]
            } else {
                ""
            };

            if is_cursor_row {
                let col = buf.cursor.col.saturating_sub(buf.scroll_col);
                let col = col.min(display_text.len());

                // Split around cursor position.
                let (before, rest) = display_text.split_at(col);
                let (cursor_ch, after) = if rest.is_empty() {
                    (" ", "")
                } else {
                    let mid = rest
                        .char_indices()
                        .nth(1)
                        .map(|(i, _)| i)
                        .unwrap_or(rest.len());
                    (&rest[..mid], &rest[mid..])
                };

                Line::from(vec![
                    Span::styled(line_num, Style::default().fg(Color::Rgb(80, 90, 110))),
                    Span::styled(before, Style::default().bg(Color::Rgb(40, 44, 52))),
                    Span::styled(
                        cursor_ch,
                        Style::default()
                            .fg(Color::Black)
                            .bg(theme.accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(after, Style::default().bg(Color::Rgb(40, 44, 52))),
                ])
            } else {
                Line::from(vec![
                    Span::styled(line_num, Style::default().fg(Color::Rgb(60, 70, 90))),
                    Span::raw(display_text),
                ])
            }
        })
        .collect();

    let para = Paragraph::new(lines);
    frame.render_widget(para, inner);

    // ── Autocompletion popup ──────────────────────────────────────────────
    if buf.completion.active && !buf.completion.candidates.is_empty() {
        let cursor_row_on_screen = buf.cursor.row.saturating_sub(buf.scroll_row) as u16;
        let cursor_col_on_screen =
            (line_num_width as u16) + (buf.cursor.col.saturating_sub(buf.scroll_col) as u16);

        let popup_w = 28u16.min(inner.width.saturating_sub(cursor_col_on_screen));
        let popup_h = (buf.completion.candidates.len() as u16 + 2).min(10);

        let popup_x = inner.x + cursor_col_on_screen.min(inner.width.saturating_sub(popup_w));
        let popup_y = if cursor_row_on_screen + popup_h + 1 < inner.height {
            inner.y + cursor_row_on_screen + 1
        } else {
            inner.y + cursor_row_on_screen.saturating_sub(popup_h)
        };

        let popup_rect = Rect {
            x: popup_x,
            y: popup_y,
            width: popup_w.max(18),
            height: popup_h.max(3),
        };

        frame.render_widget(Clear, popup_rect);
        let c_block = Block::default()
            .title(Span::styled(" 󰘦 Suggestions ", theme.accent_style()))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme.border_focused_style());

        let p_inner = c_block.inner(popup_rect);
        frame.render_widget(c_block, popup_rect);

        let items: Vec<Line> = buf
            .completion
            .candidates
            .iter()
            .enumerate()
            .take(p_inner.height as usize)
            .map(|(i, c)| {
                let is_selected = i == buf.completion.selected;
                let symbol = if is_selected { "▶ " } else { "  " };
                let style = if is_selected {
                    theme.selection_style()
                } else {
                    theme.normal_style()
                };
                Line::from(vec![
                    Span::styled(symbol, theme.accent_style()),
                    Span::styled(&c.label, style),
                ])
            })
            .collect();

        frame.render_widget(Paragraph::new(items), p_inner);
    }

    // ── Status line at bottom of editor ───────────────────────────────────
    let status_area = Rect {
        x: inner.x,
        y: inner.y + inner.height.saturating_sub(1),
        width: inner.width,
        height: 1,
    };
    let lang = buf
        .path
        .as_ref()
        .and_then(|p| p.extension())
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_else(|| "text".to_string());
    let status = format!(
        " Ln {}, Col {} │ {} │ {}  Ctrl+S Save │ Tab Complete │ Esc Close",
        buf.cursor.row + 1,
        buf.cursor.col + 1,
        lang,
        if buf.dirty { "Modified" } else { "Saved   " }
    );
    frame.render_widget(
        Paragraph::new(status).style(theme.status_style()),
        status_area,
    );
}
