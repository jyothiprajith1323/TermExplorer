//! Integrated editor UI rendering with full syntax highlighting,
//! Unicode grapheme safety, visual selection, and multi-layer completion popup.

use crate::editor::Buffer;
use crate::fonts::unicode::{slice_by_display_columns, str_display_width};
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

    let line_ending_str = match buf.line_ending {
        crate::editor::buffer::LineEnding::Lf => "LF",
        crate::editor::buffer::LineEnding::Crlf => "CRLF",
    };
    let encoding_str = buf.encoding.label();
    let lang = buf
        .path
        .as_ref()
        .and_then(|p| p.extension())
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_else(|| "text".to_string());

    let title = format!(
        " ✏  {} {} [{} | {} | {} | 󰒍 LSP] ",
        file_name, dirty_marker, lang, encoding_str, line_ending_str
    );

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

    let visible_rows = inner.height.saturating_sub(1) as usize; // reserve 1 for status line
    let line_num_width = format!("{}", buf.line_count().max(1)).len() + 1;

    // Viewport syntax highlighting
    let highlighted_spans = crate::editor::syntax::highlight_viewport(
        buf.path.as_deref(),
        &buf.lines,
        buf.scroll_row,
        visible_rows,
    );

    let mut rendered_lines = Vec::with_capacity(visible_rows);

    for (rel_row, row_idx) in (buf.scroll_row..buf.scroll_row + visible_rows).enumerate() {
        if row_idx >= buf.lines.len() {
            break;
        }

        let line_str = &buf.lines[row_idx];
        let line_num = format!("{:>width$} ", row_idx + 1, width = line_num_width);
        let is_cursor_row = row_idx == buf.cursor.row;

        let num_style = if is_cursor_row {
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(80, 90, 110))
        };

        let mut spans = vec![Span::styled(line_num, num_style)];

        // Retrieve syntax highlighted spans for this line, or fallback to plain
        let line_spans = highlighted_spans.get(rel_row);

        if is_cursor_row {
            // Reconstruct full line and safely render cursor at character index
            let chars: Vec<char> = line_str.chars().collect();
            let col = buf.cursor.col.min(chars.len());

            let before_cursor: String = chars[..col].iter().collect();
            let cursor_char: String = if col < chars.len() {
                chars[col].to_string()
            } else {
                " ".to_string()
            };
            let after_cursor: String = if col + 1 < chars.len() {
                chars[col + 1..].iter().collect()
            } else {
                String::new()
            };

            // Clip by scroll_col
            let (visible_before, _) =
                slice_by_display_columns(&before_cursor, buf.scroll_col, inner.width as usize);

            spans.push(Span::styled(
                visible_before.to_string(),
                Style::default().bg(Color::Rgb(35, 38, 46)),
            ));

            spans.push(Span::styled(
                cursor_char,
                Style::default()
                    .fg(Color::Black)
                    .bg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ));

            spans.push(Span::styled(
                after_cursor,
                Style::default().bg(Color::Rgb(35, 38, 46)),
            ));
        } else if let Some(hl_spans) = line_spans {
            // Render syntax colored spans clipped to horizontal scroll
            let mut skipped_cols = 0;
            for (r, g, b, text) in hl_spans {
                let text_w = str_display_width(text);
                if skipped_cols + text_w <= buf.scroll_col {
                    skipped_cols += text_w;
                    continue;
                }

                let span_text = if skipped_cols < buf.scroll_col {
                    let offset = buf.scroll_col - skipped_cols;
                    skipped_cols = buf.scroll_col;
                    let (sliced, _) = slice_by_display_columns(text, offset, inner.width as usize);
                    sliced.to_string()
                } else {
                    text.clone()
                };

                spans.push(Span::styled(
                    span_text,
                    Style::default().fg(Color::Rgb(*r, *g, *b)),
                ));
            }
        } else {
            // Plain text clipped
            let (visible_text, _) =
                slice_by_display_columns(line_str, buf.scroll_col, inner.width as usize);
            spans.push(Span::raw(visible_text.to_string()));
        }

        rendered_lines.push(Line::from(spans));
    }

    let para = Paragraph::new(rendered_lines);
    frame.render_widget(para, inner);

    // ── Autocompletion popup ──────────────────────────────────────────────
    if buf.completion.active && !buf.completion.candidates.is_empty() {
        let cursor_row_on_screen = buf.cursor.row.saturating_sub(buf.scroll_row) as u16;
        let cursor_col_on_screen =
            (line_num_width as u16) + (buf.cursor.col.saturating_sub(buf.scroll_col) as u16);

        let popup_w = 34u16.min(inner.width.saturating_sub(cursor_col_on_screen));
        let popup_h = (buf.completion.candidates.len() as u16 + 2).min(12);

        let popup_x = inner.x + cursor_col_on_screen.min(inner.width.saturating_sub(popup_w));
        let popup_y = if cursor_row_on_screen + popup_h + 1 < inner.height {
            inner.y + cursor_row_on_screen + 1
        } else {
            inner.y + cursor_row_on_screen.saturating_sub(popup_h)
        };

        let popup_rect = Rect {
            x: popup_x,
            y: popup_y,
            width: popup_w.max(22),
            height: popup_h.max(4),
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

                let (badge, badge_color) = match c.kind {
                    crate::editor::completion::CompletionKind::Word => {
                        ("buf", Color::Rgb(137, 180, 250)) // blue
                    }
                    crate::editor::completion::CompletionKind::Snippet => {
                        ("snip", Color::Rgb(249, 226, 175)) // yellow
                    }
                    crate::editor::completion::CompletionKind::WorkspaceSymbol => {
                        ("sym", Color::Rgb(137, 220, 235)) // sky
                    }
                    crate::editor::completion::CompletionKind::Lsp => {
                        ("lsp", Color::Rgb(203, 166, 247)) // mauve
                    }
                    crate::editor::completion::CompletionKind::Path => {
                        ("path", Color::Rgb(166, 227, 161)) // green
                    }
                };

                Line::from(vec![
                    Span::styled(symbol, theme.accent_style()),
                    Span::styled(&c.label, style),
                    Span::styled(
                        format!(" [{badge}]"),
                        Style::default()
                            .fg(badge_color)
                            .add_modifier(Modifier::BOLD),
                    ),
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
        " Ln {}, Col {} │ {} │ {} │ {} │ Ctrl+S Save │ F2/Esc Exit │ Tab Accept │ Ctrl+Space Complete",
        buf.cursor.row + 1,
        buf.cursor.col + 1,
        lang,
        line_ending_str,
        if buf.dirty { "Modified" } else { "Saved   " }
    );
    frame.render_widget(
        Paragraph::new(status).style(theme.status_style()),
        status_area,
    );
}
