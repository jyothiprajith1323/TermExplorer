//! File preview panel rendering.

use crate::state::{AppState, FocusedPanel};
use crate::theme::Theme;
use bytesize::ByteSize;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

pub fn render_preview(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let focused = state.focused == FocusedPanel::Preview;
    let border_style = if focused {
        theme.border_focused_style()
    } else {
        theme.border_style()
    };

    let preview = &state.preview;
    let file_name = preview
        .path
        .as_ref()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Preview".to_string());

    let title = format!(" 󰍉  {} ", file_name);

    let block = Block::default()
        .title(Span::styled(title, theme.header_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width < 4 || inner.height < 4 {
        return;
    }

    if preview.path.is_none() {
        // No file selected.
        let hint = Paragraph::new("\n  Select a file to preview")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(hint, inner);
        return;
    }

    if preview.is_binary {
        // Binary file — show metadata only.
        let lines = vec![
            Line::from(Span::styled(
                "  Binary file",
                Style::default().fg(theme.warning),
            )),
            Line::from(Span::raw("")),
            Line::from(format!("  Size: {}", ByteSize(preview.file_size))),
            Line::from(format!("  Type: {}", preview.mime_type)),
        ];
        let p = Paragraph::new(lines);
        frame.render_widget(p, inner);
        return;
    }

    // Render text lines with line numbers.
    let visible_height = inner.height as usize;
    let offset = preview.scroll_offset;
    let line_num_width = format!("{}", preview.total_lines.max(1)).len() + 1;

    let display_lines: Vec<Line> = if !preview.highlighted_spans.is_empty() {
        preview
            .highlighted_spans
            .iter()
            .enumerate()
            .skip(offset)
            .take(visible_height)
            .map(|(i, spans)| {
                let line_num = format!("{:>width$} ", i + 1, width = line_num_width);
                let mut line_spans = vec![Span::styled(
                    line_num,
                    Style::default().fg(Color::Rgb(70, 80, 100)),
                )];
                for (r, g, b, text) in spans {
                    line_spans.push(Span::styled(
                        text.clone(),
                        Style::default().fg(Color::Rgb(*r, *g, *b)),
                    ));
                }
                Line::from(line_spans)
            })
            .collect()
    } else {
        preview
            .lines
            .iter()
            .enumerate()
            .skip(offset)
            .take(visible_height)
            .map(|(i, text)| {
                let line_num = format!("{:>width$} ", i + 1, width = line_num_width);
                Line::from(vec![
                    Span::styled(line_num, Style::default().fg(Color::Rgb(70, 80, 100))),
                    Span::raw(text.clone()),
                ])
            })
            .collect()
    };

    let para = Paragraph::new(display_lines);
    frame.render_widget(para, inner);

    // Scroll indicator.
    if preview.total_lines > visible_height {
        let scroll_pct = (offset * 100) / (preview.total_lines - visible_height).max(1);
        let indicator = format!(" {}% ", scroll_pct);
        let ind_area = Rect {
            x: inner.x + inner.width.saturating_sub(indicator.len() as u16 + 1),
            y: inner.y + inner.height.saturating_sub(1),
            width: indicator.len() as u16,
            height: 1,
        };
        frame.render_widget(
            ratatui::widgets::Paragraph::new(indicator).style(Style::default().fg(Color::DarkGray)),
            ind_area,
        );
    }
}
