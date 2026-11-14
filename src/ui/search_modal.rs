//! Unified Search Modal UI — Floating fuzzy file and live content grep search.

use crate::search::engine::SearchMode;
use crate::state::AppState;
use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_search_modal(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let engine = &state.search_engine;

    // Center a floating dialog
    let dlg_width = (area.width * 75 / 100).clamp(60, 110);
    let dlg_height = (area.height * 75 / 100)
        .clamp(18, 32)
        .min(area.height.saturating_sub(2));
    let x = (area.width.saturating_sub(dlg_width)) / 2;
    let y = (area.height.saturating_sub(dlg_height)) / 2;

    let dlg_area = Rect {
        x,
        y,
        width: dlg_width,
        height: dlg_height,
    };

    frame.render_widget(Clear, dlg_area);

    // Header title with mode indicators
    let files_mode_active = engine.mode == SearchMode::Files;
    let files_style = if files_mode_active {
        Style::default()
            .fg(Color::Black)
            .bg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.hidden_color)
    };

    let content_style = if !files_mode_active {
        Style::default()
            .fg(Color::Black)
            .bg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.hidden_color)
    };

    let search_status = if engine.is_searching {
        Span::styled(
            " 󰑮 Searching... ",
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        let total = engine.current_len();
        Span::styled(
            format!(" {} results ", total),
            Style::default().fg(theme.success),
        )
    };

    let title_line = Line::from(vec![
        Span::styled(" 󰍉 UNIFIED WORKSPACE SEARCH  ", theme.header_style()),
        Span::styled(" [Files: Ctrl+P] ", files_style),
        Span::raw(" "),
        Span::styled(" [Content Grep: Ctrl+Shift+F] ", content_style),
        Span::raw("  "),
        search_status,
    ]);

    let block = Block::default()
        .title(title_line)
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(theme.border_focused_style());

    let inner = block.inner(dlg_area);
    frame.render_widget(block, dlg_area);

    if inner.height < 6 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input box
            Constraint::Min(4),    // Search results list
            Constraint::Length(2), // Status & filter pills / hotkeys
        ])
        .split(inner);

    // ── 1. Search Query Input Box ──────────────────────────────────────────
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focused_style());

    let prompt_icon = match engine.mode {
        SearchMode::Files => " 󰈔 File > ",
        SearchMode::Content => " 󰈙 Grep > ",
    };

    let query_display = if engine.query.is_empty() {
        match engine.mode {
            SearchMode::Files => Span::styled(
                " Type fuzzy filename to locate across workspace (e.g. main.rs, cargo)...",
                Style::default().fg(theme.hidden_color),
            ),
            SearchMode::Content => Span::styled(
                " Type text or regex to grep file contents in real-time...",
                Style::default().fg(theme.hidden_color),
            ),
        }
    } else {
        Span::styled(
            &engine.query,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
    };

    let filter_pills = {
        let mut pills = Vec::new();
        if engine.filters.case_sensitive {
            pills.push(Span::styled(" [Aa] ", Style::default().fg(theme.accent)));
        }
        if engine.filters.is_regex {
            pills.push(Span::styled(" [.*] ", Style::default().fg(theme.accent)));
        }
        if engine.filters.whole_word {
            pills.push(Span::styled(" [\\b] ", Style::default().fg(theme.accent)));
        }
        if !engine.filters.respect_gitignore {
            pills.push(Span::styled(
                " [no-git] ",
                Style::default().fg(theme.warning),
            ));
        }
        pills
    };

    let mut input_spans = vec![
        Span::styled(
            prompt_icon,
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        query_display,
        Span::styled("█", Style::default().fg(theme.accent)),
    ];
    if !filter_pills.is_empty() {
        input_spans.push(Span::raw("  "));
        input_spans.extend(filter_pills);
    }

    let input_para = Paragraph::new(Line::from(input_spans)).block(input_block);
    frame.render_widget(input_para, chunks[0]);

    // ── 2. Results List ────────────────────────────────────────────────────
    let visible_count = chunks[1].height as usize;
    let selected_idx = engine.selected_index;
    let offset = if selected_idx >= visible_count {
        selected_idx - visible_count + 1
    } else {
        0
    };

    let mut result_lines = Vec::new();

    match engine.mode {
        SearchMode::Files => {
            if engine.file_results.is_empty() {
                let msg = if engine.query.is_empty() {
                    " Start typing to search files..."
                } else if engine.is_searching {
                    " Searching files across workspace..."
                } else {
                    " No matching files found."
                };
                result_lines.push(Line::from(Span::styled(
                    msg,
                    Style::default().fg(theme.hidden_color),
                )));
            } else {
                for (i, res) in engine
                    .file_results
                    .iter()
                    .skip(offset)
                    .take(visible_count)
                    .enumerate()
                {
                    let actual_idx = offset + i;
                    let is_selected = actual_idx == selected_idx;

                    let (icon, icon_color) = if res.is_dir {
                        crate::icons::mappings::resolve_folder_icon(
                            &res.file_name,
                            false,
                            state.icon_manager.pack,
                            &crate::theme::CatppuccinPalette::mocha(),
                        )
                    } else {
                        let ext = res.path.extension().and_then(|s| s.to_str());
                        crate::icons::mappings::resolve_file_icon(
                            &res.file_name,
                            ext,
                            state.icon_manager.pack,
                            &crate::theme::CatppuccinPalette::mocha(),
                        )
                    };

                    let item_style = if is_selected {
                        Style::default()
                            .bg(theme.selection_bg)
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    let prefix = if is_selected { "▶ " } else { "  " };

                    let spans = vec![
                        Span::styled(
                            prefix,
                            if is_selected {
                                Style::default().fg(theme.accent).bg(theme.selection_bg)
                            } else {
                                Style::default()
                            },
                        ),
                        Span::styled(
                            format!("{} ", icon),
                            Style::default().fg(icon_color).bg(if is_selected {
                                theme.selection_bg
                            } else {
                                Color::Reset
                            }),
                        ),
                        Span::styled(&res.file_name, item_style),
                        Span::styled(
                            format!("  {}", res.relative_path),
                            Style::default().fg(theme.hidden_color).bg(if is_selected {
                                theme.selection_bg
                            } else {
                                Color::Reset
                            }),
                        ),
                    ];
                    result_lines.push(Line::from(spans));
                }
            }
        }
        SearchMode::Content => {
            if engine.content_results.is_empty() {
                let msg = if engine.query.is_empty() {
                    " Start typing to grep workspace content..."
                } else if engine.is_searching {
                    " Grepping workspace files..."
                } else {
                    " No matching text found."
                };
                result_lines.push(Line::from(Span::styled(
                    msg,
                    Style::default().fg(theme.hidden_color),
                )));
            } else {
                for (i, m) in engine
                    .content_results
                    .iter()
                    .skip(offset)
                    .take(visible_count)
                    .enumerate()
                {
                    let actual_idx = offset + i;
                    let is_selected = actual_idx == selected_idx;

                    let (icon, icon_color) = {
                        let file_name = m
                            .path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        let ext = m.path.extension().and_then(|s| s.to_str());
                        crate::icons::mappings::resolve_file_icon(
                            &file_name,
                            ext,
                            state.icon_manager.pack,
                            &crate::theme::CatppuccinPalette::mocha(),
                        )
                    };
                    let prefix = if is_selected { "▶ " } else { "  " };

                    // Highlight matched segment
                    let line_text = &m.line_text;
                    let start = m.match_start.min(line_text.len());
                    let end = m.match_end.min(line_text.len());

                    let before = &line_text[..start];
                    let matched = &line_text[start..end];
                    let after = &line_text[end..];

                    let bg = if is_selected {
                        theme.selection_bg
                    } else {
                        Color::Reset
                    };

                    let spans = vec![
                        Span::styled(
                            prefix,
                            if is_selected {
                                Style::default().fg(theme.accent).bg(bg)
                            } else {
                                Style::default()
                            },
                        ),
                        Span::styled(format!("{} ", icon), Style::default().fg(icon_color).bg(bg)),
                        Span::styled(
                            format!("{}:{}  ", m.relative_path, m.line_number),
                            Style::default()
                                .fg(theme.accent)
                                .bg(bg)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(before, Style::default().fg(theme.fg).bg(bg)),
                        Span::styled(
                            matched,
                            Style::default()
                                .fg(Color::Black)
                                .bg(theme.warning)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(after, Style::default().fg(theme.fg).bg(bg)),
                    ];

                    result_lines.push(Line::from(spans));
                }
            }
        }
    }

    let results_para = Paragraph::new(result_lines);
    frame.render_widget(results_para, chunks[1]);

    // ── 3. Footer Key Navigation Bar ───────────────────────────────────────
    let footer_line = Line::from(vec![
        Span::styled(
            " [Tab] ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Toggle Mode  │ "),
        Span::styled(
            " [↑/↓] ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Navigate  │ "),
        Span::styled(
            " [Enter] ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Open/Jump  │ "),
        Span::styled(
            " [Esc] ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Close  │ "),
        Span::styled(
            " [Alt+C] ",
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Case  │ "),
        Span::styled(
            " [Alt+R] ",
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Regex"),
    ]);

    let footer_para = Paragraph::new(footer_line);
    frame.render_widget(footer_para, chunks[2]);
}
