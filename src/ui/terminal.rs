//! Embedded terminal panel rendering.
//!
//! The TUI renders the scrollback as plain (but ANSI-coloured) text.
//! For full VT100 emulation inside a Ratatui widget, we use a simplified
//! approach: render the raw output and trust the terminal emulator for
//! display, while Ratatui handles the surrounding UI chrome.

use crate::state::{AppState, FocusedPanel};
use crate::theme::Theme;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

pub fn render_terminal(
    frame: &mut Frame,
    area: Rect,
    scrollback: &[u8],
    state: &AppState,
    theme: &Theme,
) {
    let focused = state.focused == FocusedPanel::Terminal;
    let border_style = if focused {
        theme.border_focused_style()
    } else {
        theme.border_style()
    };

    let shell_name = std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string());
    let shell_base = std::path::Path::new(&shell_name)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| shell_name.clone());

    let title = format!(" 󰆍  Terminal ({}) [Ctrl+` toggle] ", shell_base);
    let block = Block::default()
        .title(Span::styled(title, theme.header_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 2 {
        return;
    }

    let all_lines = process_terminal_output(scrollback);
    let visible_h = inner.height as usize;
    let start = all_lines.len().saturating_sub(visible_h);

    let mut display_lines: Vec<Line> = all_lines[start..]
        .iter()
        .map(|l| Line::from(Span::styled(l.clone(), Style::default().fg(theme.fg))))
        .collect();

    // If focused, render cursor on the last line
    if focused {
        if let Some(last_line) = display_lines.last_mut() {
            last_line
                .spans
                .push(Span::styled("█", theme.accent_style()));
        } else {
            display_lines.push(Line::from(vec![Span::styled("█", theme.accent_style())]));
        }
    }

    let para = Paragraph::new(display_lines);
    frame.render_widget(para, inner);
}

/// Process raw terminal output bytes, handling ANSI sequences, CR, BS, tabs.
fn process_terminal_output(raw: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(raw);
    let mut lines: Vec<String> = Vec::new();
    let mut current_line = String::new();

    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\x1B' => {
                if chars.peek() == Some(&'[') {
                    chars.next();
                    while let Some(&next) = chars.peek() {
                        chars.next();
                        if next.is_ascii_alphabetic()
                            || next == '~'
                            || next == 'm'
                            || next == 'K'
                            || next == 'H'
                            || next == 'J'
                        {
                            break;
                        }
                    }
                } else if chars.peek() == Some(&']') {
                    chars.next();
                    while let Some(&next) = chars.peek() {
                        chars.next();
                        if next == '\x07' || next == '\x1B' {
                            break;
                        }
                    }
                }
            }
            '\r' => {
                if chars.peek() != Some(&'\n') {
                    current_line.clear();
                }
            }
            '\n' => {
                lines.push(current_line);
                current_line = String::new();
            }
            '\x08' => {
                current_line.pop();
            }
            '\t' => {
                current_line.push_str("    ");
            }
            other if !other.is_control() => {
                current_line.push(other);
            }
            _ => {}
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    lines
}
