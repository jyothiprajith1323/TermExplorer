//! Embedded terminal panel rendering — developer workstation terminal experience.

use crate::shell::manager::TerminalSessionManager;
use crate::state::{AppState, FocusedPanel};
use crate::theme::Theme;
use crate::ui::hit_test::{AppAction, HitRegistry, WidgetId};
use ratatui::{
    layout::{Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

/// Backward-compatible terminal rendering entry point for tests and standalone invocations.
pub fn render_terminal(
    frame: &mut Frame,
    area: Rect,
    scrollback: &[u8],
    state: &AppState,
    theme: &Theme,
    hit_registry: &mut HitRegistry,
) {
    render_terminal_manager(frame, area, None, scrollback, state, theme, hit_registry);
}

/// Render the terminal panel backed by the multi-session manager.
pub fn render_terminal_manager(
    frame: &mut Frame,
    area: Rect,
    manager: Option<&TerminalSessionManager>,
    scrollback_fallback: &[u8],
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

    // Render session tabs or fallback shell name
    let active_sess = manager.and_then(|m| m.active_session());
    let active_idx = manager.map(|m| m.active_index()).unwrap_or(0);
    let session_count = manager.map(|m| m.session_count()).unwrap_or(0);

    if let Some(mgr) = manager {
        if mgr.session_count() > 0 {
            for (idx, sess) in mgr.sessions().iter().enumerate() {
                let is_active = idx == active_idx;
                let shell_name = sess.shell_name();
                let is_exited = sess.is_exited();

                let tab_label = if is_exited {
                    format!(" [{}: {}(Exited)] ", idx + 1, shell_name)
                } else if is_active {
                    format!(" [{}: {}] ", idx + 1, shell_name)
                } else {
                    format!(" {}: {} ", idx + 1, shell_name)
                };

                let tab_len = tab_label.chars().count();
                if left_len + tab_len + 30 <= (area.width as usize) {
                    let tab_style = if is_active {
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD)
                    } else if is_exited {
                        Style::default().fg(theme.error)
                    } else {
                        Style::default().fg(theme.header_fg)
                    };

                    let tab_x = area.x + left_len as u16;
                    title_spans.push(Span::styled(tab_label, tab_style));
                    left_len += tab_len;

                    hit_registry.register_button(
                        WidgetId::TerminalTab(idx),
                        Rect {
                            x: tab_x,
                            y: area.y,
                            width: tab_len as u16,
                            height: 1,
                        },
                        AppAction::SwitchTerminalTab(idx),
                        10,
                    );
                }
            }

            // Show active session working directory if space permits
            if let Some(active) = active_sess {
                let cwd_str = active
                    .working_dir
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "~".to_string());
                let cwd_display = format!("~/{} ", cwd_str);
                let cwd_len = cwd_display.chars().count();
                if left_len + cwd_len + 25 <= (area.width as usize) {
                    title_spans.push(Span::styled(
                        cwd_display,
                        Style::default().fg(theme.size_color),
                    ));
                    left_len += cwd_len;
                }

                // If scrolled into scrollback history, show indicator
                let scroll_off = active.screen.scroll_offset();
                if scroll_off > 0 {
                    let scroll_indicator = format!("[▲ +{} lines] ", scroll_off);
                    let ind_len = scroll_indicator.chars().count();
                    if left_len + ind_len + 20 <= (area.width as usize) {
                        title_spans.push(Span::styled(
                            scroll_indicator,
                            Style::default().fg(theme.warning),
                        ));
                        left_len += ind_len;
                    }
                }

                // If exited, show [Restart] button
                if active.is_exited() {
                    let restart_label = "[Restart] ";
                    let rst_len = restart_label.chars().count();
                    if left_len + rst_len + 20 <= (area.width as usize) {
                        let rst_x = area.x + left_len as u16;
                        title_spans.push(Span::styled(
                            restart_label,
                            Style::default()
                                .fg(theme.error)
                                .add_modifier(Modifier::BOLD),
                        ));
                        left_len += rst_len;

                        hit_registry.register_button(
                            WidgetId::TerminalButtonRestart,
                            Rect {
                                x: rst_x,
                                y: area.y,
                                width: rst_len as u16,
                                height: 1,
                            },
                            AppAction::RestartTerminal,
                            10,
                        );
                    }
                }
            }
        }
    } else {
        // Fallback default single-shell header
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
    }

    // Right-side actions
    if area.width >= 70 {
        let max_w = max_label.chars().count();
        // Right side: "[+] " (4), "[×] " (4, if multiple sessions), "[Hide] " (7), "{max_label} " (max_w + 1)
        let close_tab_len = if session_count > 1 { 4 } else { 0 };
        let right_len = 4 + close_tab_len + 7 + max_w + 1;

        if area.width as usize > left_len + right_len + 2 {
            let spaces = (area.width as usize) - 2 - left_len - right_len;
            title_spans.push(Span::raw(" ".repeat(spaces)));
        } else {
            title_spans.push(Span::raw(" "));
        }

        title_spans.push(Span::styled("[+] ", Style::default().fg(theme.success)));
        if session_count > 1 {
            title_spans.push(Span::styled("[×] ", Style::default().fg(theme.error)));
        }
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

        let mut offset = 4;
        if session_count > 1 {
            hit_registry.register_button(
                WidgetId::TerminalButtonCloseSession,
                Rect {
                    x: btn_start_x + offset,
                    y: area.y,
                    width: 3,
                    height: 1,
                },
                AppAction::CloseTerminalSession,
                10,
            );
            offset += 4;
        }

        hit_registry.register_button(
            WidgetId::TerminalButtonClose,
            Rect {
                x: btn_start_x + offset,
                y: area.y,
                width: 6,
                height: 1,
            },
            AppAction::ToggleTerminal,
            10,
        );
        offset += 7;

        hit_registry.register_button(
            WidgetId::TerminalButtonMax,
            Rect {
                x: btn_start_x + offset,
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

    if inner.height < 1 || inner.width < 4 {
        return;
    }

    // Render virtual terminal screen lines or fallback scrollback buffer
    if let Some(active) = active_sess {
        let lines = active
            .screen
            .render_all_rows(theme.fg, theme.bg.unwrap_or(Color::Reset));
        let visible_lines: Vec<Line> = lines.into_iter().take(inner.height as usize).collect();
        let para = Paragraph::new(visible_lines);
        frame.render_widget(para, inner);

        // Position terminal cursor when focused and not hidden by application
        if focused && !active.screen.hide_cursor() && active.screen.scroll_offset() == 0 {
            let (cur_row, cur_col) = active.screen.cursor_position();
            if cur_row < inner.height && cur_col < inner.width {
                frame.set_cursor_position(Position::new(inner.x + cur_col, inner.y + cur_row));
            }
        }
    } else {
        let all_lines = process_terminal_output(scrollback_fallback);
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
