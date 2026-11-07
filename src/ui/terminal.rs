//! Embedded terminal panel rendering — developer workstation terminal experience.

use crate::state::{AppState, FocusedPanel};
use crate::theme::Theme;
use crate::ui::hit_test::{AppAction, HitRegistry, WidgetId};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
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
    hit_registry: &mut HitRegistry,
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

    let cwd_str = state
        .explorer
        .current_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "~".to_string());

    let terminal_tag = if focused {
        Span::styled(
            " [Terminal] ",
            Style::default()
                .fg(Color::Black)
                .bg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            " [Terminal] ",
            Style::default()
                .fg(theme.header_fg)
                .add_modifier(Modifier::BOLD),
        )
    };

    let max_label = if state.terminal_maximized {
        "[Restore]"
    } else {
        "[Maximize]"
    };

    let mut title_spans = vec![terminal_tag];
    let mut left_len: usize = 12; // " [Terminal] "

    let session_str = format!("1: {} ", shell_base);
    let session_len = session_str.chars().count();
    let cwd_display = format!("~/{} ", cwd_str);
    let cwd_len = cwd_display.chars().count();

    if area.width >= 70 {
        title_spans.push(Span::styled(
            session_str,
            Style::default()
                .fg(theme.header_fg)
                .add_modifier(Modifier::BOLD),
        ));
        left_len += session_len;

        title_spans.push(Span::styled(
            cwd_display,
            Style::default().fg(theme.size_color),
        ));
        left_len += cwd_len;
    } else if area.width >= 45 {
        title_spans.push(Span::styled(
            session_str,
            Style::default().fg(theme.header_fg),
        ));
        left_len += session_len;
    }

    if area.width >= 70 {
        let max_w = max_label.chars().count();
        // Right side: "[+] " (4), "[Hide] " (7), "{max_label} " (max_w + 1)
        let right_len = 4 + 7 + max_w + 1;

        if area.width as usize > left_len + right_len + 2 {
            let spaces = (area.width as usize) - 2 - left_len - right_len;
            title_spans.push(Span::raw(" ".repeat(spaces)));
        } else {
            title_spans.push(Span::raw(" "));
        }

        title_spans.push(Span::styled("[+] ", Style::default().fg(theme.success)));
        title_spans.push(Span::styled("[Hide] ", Style::default().fg(theme.error)));
        title_spans.push(Span::styled(
            format!("{} ", max_label),
            Style::default().fg(theme.accent),
        ));

        let btn_start_x = area.x + area.width.saturating_sub(1 + right_len as u16);

        hit_registry.register_button(
            WidgetId::TerminalButtonNew,
            Rect {
                x: btn_start_x,
                y: area.y,
                width: 3,
                height: 1,
            },
            AppAction::ClearTerminal,
            10,
        );
        hit_registry.register_button(
            WidgetId::TerminalButtonClose,
            Rect {
                x: btn_start_x + 4,
                y: area.y,
                width: 6,
                height: 1,
            },
            AppAction::ToggleTerminal,
            10,
        );
        hit_registry.register_button(
            WidgetId::TerminalButtonMax,
            Rect {
                x: btn_start_x + 11,
                y: area.y,
                width: max_w as u16,
                height: 1,
            },
            AppAction::MaximizeTerminal,
            10,
        );
    } else if area.width >= 45 {
        let right_len = 12; // "[+] " (4) + "[×] " (4) + "[▲] " (4)
        if area.width as usize > left_len + right_len + 2 {
            let spaces = (area.width as usize) - 2 - left_len - right_len;
            title_spans.push(Span::raw(" ".repeat(spaces)));
        } else {
            title_spans.push(Span::raw(" "));
        }

        let compact_max = if state.terminal_maximized {
            "[▼] "
        } else {
            "[▲] "
        };
        title_spans.push(Span::styled("[+] ", Style::default().fg(theme.success)));
        title_spans.push(Span::styled("[×] ", Style::default().fg(theme.error)));
        title_spans.push(Span::styled(compact_max, Style::default().fg(theme.accent)));

        let btn_start_x = area.x + area.width.saturating_sub(1 + right_len as u16);

        hit_registry.register_button(
            WidgetId::TerminalButtonNew,
            Rect {
                x: btn_start_x,
                y: area.y,
                width: 3,
                height: 1,
            },
            AppAction::ClearTerminal,
            10,
        );
        hit_registry.register_button(
            WidgetId::TerminalButtonClose,
            Rect {
                x: btn_start_x + 4,
                y: area.y,
                width: 3,
                height: 1,
            },
            AppAction::ToggleTerminal,
            10,
        );
        hit_registry.register_button(
            WidgetId::TerminalButtonMax,
            Rect {
                x: btn_start_x + 8,
                y: area.y,
                width: 3,
                height: 1,
            },
            AppAction::MaximizeTerminal,
            10,
        );
    }

    hit_registry.register_button(
        WidgetId::TerminalTitle,
        Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        },
        AppAction::FocusTerminal,
        2,
    );

    let title = Line::from(title_spans);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style);

    let inner = block.inner(area);
    hit_registry.register_button(
        WidgetId::TerminalContent,
        inner,
        AppAction::FocusTerminal,
        1,
    );
    frame.render_widget(block, area);

    if inner.height < 2 || inner.width < 4 {
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
                // Carriage return: reset line cursor if next char is not \n
                if chars.peek() != Some(&'\n') {
                    current_line.clear();
                }
            }
            '\n' => {
                lines.push(std::mem::take(&mut current_line));
            }
            '\t' => {
                current_line.push_str("    ");
            }
            '\x08' => {
                current_line.pop();
            }
            ch if !ch.is_control() => {
                current_line.push(ch);
            }
            _ => {}
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}
