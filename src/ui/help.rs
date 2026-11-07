//! Help screen rendering.

use crate::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

const HELP_TEXT: &[(&str, &str)] = &[
    ("NAVIGATION", ""),
    ("↑ / ↓", "Move selection"),
    ("← / →", "Switch panels"),
    ("Enter", "Open folder / select file"),
    ("Backspace", "Navigate to parent directory"),
    ("Tab", "Cycle focus between panels"),
    ("", ""),
    ("FILE OPERATIONS", ""),
    ("F4", "Create new file"),
    ("F7", "Create new directory"),
    ("F8", "Delete selected item"),
    ("r", "Rename selected item"),
    ("y", "Copy selected item (not impl)"),
    ("p", "Paste (not impl)"),
    ("", ""),
    ("PANELS & VIEWS", ""),
    ("F1", "Toggle this help screen"),
    ("F2", "Edit selected file"),
    ("F3", "Preview selected file"),
    ("F5", "Refresh directory"),
    ("F6", "Toggle system monitor"),
    ("Ctrl+`", "Toggle embedded terminal"),
    ("Ctrl+B", "Toggle sidebar"),
    ("h", "Toggle hidden files"),
    ("", ""),
    ("SEARCH", ""),
    ("Ctrl+F", "Search in directory"),
    ("/", "Filter files"),
    ("Esc", "Clear search / close dialog"),
    ("", ""),
    ("EDITOR (when editing)", ""),
    ("Ctrl+S", "Save file"),
    ("Ctrl+Z", "Undo"),
    ("Ctrl+Y", "Redo"),
    ("Ctrl+F", "Find in file"),
    ("Esc", "Close editor"),
    ("", ""),
    ("GENERAL", ""),
    ("Ctrl+Q", "Quit TermExplorer"),
    ("Alt+P", "Pin current directory"),
];

pub fn render_help(frame: &mut Frame, area: Rect, theme: &Theme) {
    // Centre a floating popup.
    let popup_w = area.width.min(60);
    let popup_h = area.height.min(45);
    let popup = Rect {
        x: area.x + (area.width.saturating_sub(popup_w)) / 2,
        y: area.y + (area.height.saturating_sub(popup_h)) / 2,
        width: popup_w,
        height: popup_h,
    };

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(Span::styled("  Keyboard Shortcuts ", theme.header_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(theme.border_focused_style());

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let lines: Vec<Line> = HELP_TEXT
        .iter()
        .map(|(key, desc)| {
            if desc.is_empty() && !key.is_empty() {
                // Section header.
                Line::from(Span::styled(format!(" ── {} ", key), theme.accent_style()))
            } else if key.is_empty() {
                Line::from("")
            } else {
                Line::from(vec![
                    Span::styled(
                        format!("  {:14}", key),
                        Style::default().fg(Color::Rgb(229, 192, 123)),
                    ),
                    Span::styled(format!(" {}", desc), theme.normal_style()),
                ])
            }
        })
        .collect();

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(para, inner);
}
