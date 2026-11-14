//! File preview panel rendering with tabs for Preview, Information, Hex, and Actions.

use crate::preview::data::DataPreviewKind;
use crate::preview::manager::PreviewTab;
use crate::state::{AppState, FocusedPanel};
use crate::theme::Theme;
use crate::ui::hit_test::{AppAction, HitRegistry, WidgetId};
use bytesize::ByteSize;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

pub fn render_preview(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: &Theme,
    hit_registry: &mut HitRegistry,
) {
    let focused = state.focused == FocusedPanel::Preview;
    let border_style = if focused {
        theme.border_focused_style()
    } else {
        theme.border_style()
    };

    hit_registry.register_button(WidgetId::PreviewPanel, area, AppAction::FocusPreview, 1);

    let preview = &state.preview;
    let file_name = preview
        .path
        .as_ref()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Inspector".to_string());

    // Modern styled tab pills with adaptive formatting to prevent clipping
    let tabs = [
        (PreviewTab::Preview, "󰈈 Preview", "󰈈"),
        (PreviewTab::Information, "󰋽 Info", "󰋽"),
        (PreviewTab::Hex, "󰘳 Metadata", "󰘳"),
        (PreviewTab::Actions, "󰌒 Actions", "󰌒"),
    ];

    let mut title_spans = Vec::new();
    let mut cur_x = area.x + 1;

    if let Some(path) = &preview.path {
        let (icon_str, icon_color) = if path.is_dir() {
            crate::icons::mappings::resolve_folder_icon(
                &file_name,
                false,
                state.icon_manager.pack,
                &crate::theme::CatppuccinPalette::mocha(),
            )
        } else {
            let ext = path.extension().and_then(|s| s.to_str());
            crate::icons::mappings::resolve_file_icon(
                &file_name,
                ext,
                state.icon_manager.pack,
                &crate::theme::CatppuccinPalette::mocha(),
            )
        };

        let type_badge = if path.is_dir() {
            "[DIR]"
        } else if let Some(d) = &preview.full_data {
            match d.format_info.category {
                crate::preview::detector::FileCategory::SourceCode => "[CODE]",
                crate::preview::detector::FileCategory::Markdown => "[MD]",
                crate::preview::detector::FileCategory::Image => "[IMG]",
                crate::preview::detector::FileCategory::Video => "[VID]",
                crate::preview::detector::FileCategory::Audio => "[AUDIO]",
                crate::preview::detector::FileCategory::Pdf => "[PDF]",
                crate::preview::detector::FileCategory::Office => "[DOC]",
                crate::preview::detector::FileCategory::Data => "[DATA]",
                crate::preview::detector::FileCategory::Archive => "[ZIP]",
                crate::preview::detector::FileCategory::Database => "[DB]",
                crate::preview::detector::FileCategory::BinaryHex => "[BIN]",
                crate::preview::detector::FileCategory::Other => "[FILE]",
            }
        } else {
            "[FILE]"
        };

        if area.width >= 50 {
            let icon_span = format!(" {} ", icon_str);
            let icon_w = icon_span.chars().count() as u16;
            title_spans.push(Span::styled(icon_span, Style::default().fg(icon_color)));
            cur_x += icon_w;

            let max_fn_w = (area.width.saturating_sub(50) as usize).clamp(10, 40);
            let display_fn = if file_name.len() > max_fn_w {
                format!("{}… ", &file_name[..max_fn_w.saturating_sub(1)])
            } else {
                format!("{} ", file_name)
            };
            let fn_w = display_fn.chars().count() as u16;
            title_spans.push(Span::styled(display_fn, theme.header_style()));
            cur_x += fn_w;

            let badge_span = format!("{} ", type_badge);
            let badge_w = badge_span.chars().count() as u16;
            title_spans.push(Span::styled(
                badge_span,
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ));
            cur_x += badge_w;

            title_spans.push(Span::styled("│ ", Style::default().fg(theme.border)));
            cur_x += 2;
        }
    }

    for (idx, (t, label, icon)) in tabs.iter().enumerate() {
        let is_active = preview.active_tab == *t;
        let display_label = if area.width >= 54 {
            *label
        } else if area.width >= 42 {
            if *t == PreviewTab::Hex {
                "󰘳 Meta"
            } else {
                *label
            }
        } else if is_active {
            *label
        } else {
            *icon
        };

        let span = if is_active {
            Span::styled(
                format!(" [{}] ", display_label),
                Style::default()
                    .fg(Color::Black)
                    .bg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(
                format!(" {} ", display_label),
                Style::default().fg(theme.size_color),
            )
        };

        let tab_w = (display_label.chars().count() + if is_active { 4 } else { 2 }) as u16;
        let tab_rect = Rect {
            x: cur_x,
            y: area.y,
            width: tab_w,
            height: 1,
        };
        hit_registry.register_button(
            WidgetId::PreviewTab(idx),
            tab_rect,
            AppAction::SelectPreviewTab(idx),
            5,
        );
        cur_x += tab_w;
        title_spans.push(span);
    }

    let visible_height = area.height.saturating_sub(2) as usize;
    let offset = preview.scroll_offset;

    let start_line = if preview.total_lines > 0 {
        offset + 1
    } else {
        0
    };
    let end_line = (offset + visible_height).min(preview.total_lines);
    let footer_text = if preview.total_lines > 0 && preview.active_tab == PreviewTab::Preview {
        format!(
            " Lines {}-{} of {} ",
            start_line, end_line, preview.total_lines
        )
    } else {
        String::new()
    };

    let mut block = Block::default()
        .title(Line::from(title_spans))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style);

    if !footer_text.is_empty() && area.width >= 40 {
        block = block.title_bottom(Line::from(Span::styled(
            footer_text,
            Style::default().fg(theme.size_color),
        )));
    }

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width < 4 || inner.height < 4 {
        return;
    }

    let Some(path) = &preview.path else {
        let top_padding = (inner.height.saturating_sub(9)) / 2;
        let mut empty_lines = Vec::new();
        for _ in 0..top_padding {
            empty_lines.push(Line::raw(""));
        }
        empty_lines.push(Line::from(vec![
            Span::styled(
                "  󰍉 ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Select a file to preview",
                Style::default()
                    .fg(theme.header_fg)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        empty_lines.push(Line::raw(""));
        empty_lines.push(Line::from(Span::styled(
            "  Choose an item in the explorer to inspect its content.",
            Style::default().fg(theme.size_color),
        )));
        empty_lines.push(Line::raw(""));
        empty_lines.push(Line::from(vec![
            Span::styled("  [↑ / ↓]  ", Style::default().fg(theme.accent)),
            Span::styled("Navigate entries", Style::default().fg(Color::DarkGray)),
        ]));
        empty_lines.push(Line::from(vec![
            Span::styled("  [← / →]  ", Style::default().fg(theme.accent)),
            Span::styled("Open / Exit folders", Style::default().fg(Color::DarkGray)),
        ]));
        empty_lines.push(Line::from(vec![
            Span::styled("  [Tab]    ", Style::default().fg(theme.accent)),
            Span::styled("Switch panel focus", Style::default().fg(Color::DarkGray)),
        ]));
        empty_lines.push(Line::from(vec![
            Span::styled("  [/]      ", Style::default().fg(theme.accent)),
            Span::styled("Fuzzy filter entries", Style::default().fg(Color::DarkGray)),
        ]));

        let p = Paragraph::new(empty_lines);
        frame.render_widget(p, inner);
        return;
    };

    if path.is_dir() {
        render_directory_preview(frame, inner, path, theme, preview.dir_data.as_ref());
        return;
    }

    let full_data = match &preview.full_data {
        Some(d) => d,
        None => {
            let top_padding = (inner.height.saturating_sub(6)) / 2;
            let mut empty_lines = Vec::new();
            for _ in 0..top_padding {
                empty_lines.push(Line::raw(""));
            }
            empty_lines.push(Line::from(vec![
                Span::styled(
                    "  󰍉 ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Select a file to preview",
                    Style::default()
                        .fg(theme.header_fg)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            empty_lines.push(Line::raw(""));
            empty_lines.push(Line::from(Span::styled(
                "  Choose an item in the explorer to inspect its content.",
                Style::default().fg(theme.size_color),
            )));
            let p = Paragraph::new(empty_lines);
            frame.render_widget(p, inner);
            return;
        }
    };

    let visible_height = inner.height as usize;
    let offset = preview.scroll_offset;

    match preview.active_tab {
        PreviewTab::Information => {
            render_info_tab(frame, inner, full_data, theme);
        }
        PreviewTab::Hex => {
            render_hex_tab(frame, inner, full_data, offset, visible_height, theme);
        }
        PreviewTab::Actions => {
            render_actions_tab(frame, inner, full_data, offset, theme);
        }
        PreviewTab::Preview => {
            render_content_preview(
                frame,
                inner,
                full_data,
                preview,
                offset,
                visible_height,
                theme,
            );
        }
    }
}

fn render_content_preview(
    frame: &mut Frame,
    area: Rect,
    full_data: &crate::preview::FullPreviewData,
    preview: &crate::state::PreviewState,
    offset: usize,
    visible_height: usize,
    theme: &Theme,
) {
    // 1. Image preview
    if let Some(img) = &full_data.image {
        let mut lines = vec![
            Line::from(vec![
                Span::styled(
                    "  Dimensions: ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(
                    "{} × {} px  ({})",
                    img.width, img.height, img.aspect_ratio
                )),
                Span::styled("  •  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} | {}", img.format_name, img.color_type),
                    Style::default().fg(theme.size_color),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Hints:      ", Style::default().fg(theme.accent)),
                Span::styled(
                    "[f] Fullscreen  •  [+] Zoom In  •  [-] Zoom Out",
                    Style::default().fg(theme.status_fg),
                ),
            ]),
            Line::raw(""),
        ];

        let img_w = img.halfblock_grid.first().map(|r| r.len()).unwrap_or(0);
        let left_pad = (area.width as usize).saturating_sub(img_w) / 2;
        let pad_span = Span::raw(" ".repeat(left_pad));

        for row in &img.halfblock_grid {
            let mut spans = vec![pad_span.clone()];
            for &(fg, bg, _ch) in row {
                spans.push(Span::styled("▀", Style::default().fg(fg).bg(bg)));
            }
            lines.push(Line::from(spans));
        }

        let p = Paragraph::new(lines);
        frame.render_widget(p, area);
        return;
    }

    // 2. Video preview
    if let Some(vid) = &full_data.video {
        let mut lines = vec![
            Line::from(vec![
                Span::styled(
                    "  Video: ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(
                    "{} × {} px  |  Codec: {}",
                    vid.width.unwrap_or(0),
                    vid.height.unwrap_or(0),
                    vid.video_codec.as_deref().unwrap_or("Unknown")
                )),
            ]),
            Line::from(vec![
                Span::styled("  Audio: ", Style::default().fg(theme.accent)),
                Span::raw(format!(
                    "Codec: {}  |  Duration: {:.1}s",
                    vid.audio_codec.as_deref().unwrap_or("None"),
                    vid.duration_seconds.unwrap_or(0.0)
                )),
            ]),
            Line::from(vec![
                Span::styled("  Controls: ", Style::default().fg(theme.accent)),
                Span::styled(
                    "[Space] Play/Pause  •  [←/→] ±5s Seek  •  [m] Mute",
                    Style::default().fg(theme.status_fg),
                ),
            ]),
            Line::raw(""),
        ];

        if !vid.thumbnail_grid.is_empty() {
            let thumb_w = vid.thumbnail_grid.first().map(|r| r.len()).unwrap_or(0);
            let left_pad = (area.width as usize).saturating_sub(thumb_w) / 2;
            let pad_span = Span::raw(" ".repeat(left_pad));

            lines.push(Line::from(Span::styled(
                "  [Extracted First Frame]",
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            )));
            for row in &vid.thumbnail_grid {
                let mut spans = vec![pad_span.clone()];
                for &(fg, bg, _ch) in row {
                    spans.push(Span::styled("▀", Style::default().fg(fg).bg(bg)));
                }
                lines.push(Line::from(spans));
            }
        } else {
            lines.push(Line::from(Span::styled(
                if vid.ffmpeg_available {
                    "  Generating thumbnail..."
                } else {
                    "  (ffmpeg not installed for video thumbnail extraction)"
                },
                Style::default().fg(Color::DarkGray),
            )));
        }

        let p = Paragraph::new(lines);
        frame.render_widget(p, area);
        return;
    }

    // 3. Audio preview
    if let Some(aud) = &full_data.audio {
        let lines = vec![
            Line::from(vec![
                Span::styled(
                    "  Title:    ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(aud.title.as_deref().unwrap_or("Unknown Title")),
            ]),
            Line::from(vec![
                Span::styled("  Artist:   ", Style::default().fg(theme.accent)),
                Span::raw(aud.artist.as_deref().unwrap_or("Unknown Artist")),
            ]),
            Line::from(vec![
                Span::styled("  Album:    ", Style::default().fg(theme.accent)),
                Span::raw(aud.album.as_deref().unwrap_or("Unknown Album")),
            ]),
            Line::raw(""),
            Line::from(vec![
                Span::styled("  Codec:    ", Style::default().fg(Color::Gray)),
                Span::raw(format!(
                    "{}  |  Sample Rate: {} Hz  |  Channels: {}",
                    aud.codec.as_deref().unwrap_or("Unknown"),
                    aud.sample_rate_hz.unwrap_or(0),
                    aud.channels.unwrap_or(0)
                )),
            ]),
            Line::from(vec![
                Span::styled("  Duration: ", Style::default().fg(Color::Gray)),
                Span::raw(format!(
                    "{:.1} seconds",
                    aud.duration_seconds.unwrap_or(0.0)
                )),
            ]),
            Line::raw(""),
            Line::from(Span::styled(
                match aud.player_available {
                    Some(p) => format!("  Player: {} detected (Use Actions tab to play)", p),
                    None => {
                        "  (No player detected: install mpv or ffplay for playback)".to_string()
                    }
                },
                Style::default().fg(theme.warning),
            )),
        ];

        let p = Paragraph::new(lines);
        frame.render_widget(p, area);
        return;
    }

    // 4. PDF preview
    if let Some(pdf) = &full_data.pdf {
        let mut lines = vec![
            Line::from(vec![
                Span::styled(
                    "  PDF Document: ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(&pdf.pdf_version),
                Span::styled("  •  Pages: ", Style::default().fg(theme.accent)),
                Span::raw(format!("{}", pdf.page_count.unwrap_or(1))),
            ]),
            Line::from(vec![
                Span::styled("  Title: ", Style::default().fg(theme.accent)),
                Span::raw(pdf.title.as_deref().unwrap_or("Untitled")),
            ]),
            Line::from(vec![
                Span::styled("  Navigation: ", Style::default().fg(theme.accent)),
                Span::styled(
                    "[n] Next Page  •  [p] Prev Page  •  [+] Zoom In  •  [-] Zoom Out",
                    Style::default().fg(theme.status_fg),
                ),
            ]),
            Line::raw(""),
        ];

        if !pdf.extracted_text.is_empty() {
            lines.push(Line::from(Span::styled(
                "  ── Page Text Stream ──",
                Style::default().fg(Color::DarkGray),
            )));
            for l in pdf
                .extracted_text
                .iter()
                .skip(offset)
                .take(visible_height.saturating_sub(5))
            {
                lines.push(Line::from(format!("  {}", l)));
            }
        } else {
            lines.push(Line::from(Span::styled(
                if pdf.pdftotext_available {
                    "  (No extractable text)"
                } else {
                    "  (Install poppler-utils / pdftotext for text stream preview)"
                },
                Style::default().fg(Color::DarkGray),
            )));
        }

        let p = Paragraph::new(lines);
        frame.render_widget(p, area);
        return;
    }

    // 5. Office document preview
    if let Some(off) = &full_data.office {
        let mut lines = vec![
            Line::from(vec![
                Span::styled(
                    "  Document: ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(off.document_type),
            ]),
            Line::raw(""),
        ];

        if !off.table_rows.is_empty() {
            lines.push(Line::from(Span::styled(
                "  ── Sheet Data ──",
                Style::default().fg(theme.accent),
            )));
            for row in off
                .table_rows
                .iter()
                .skip(offset)
                .take(visible_height.saturating_sub(4))
            {
                let formatted_row = row.join("  │  ");
                lines.push(Line::from(format!("  │ {}", formatted_row)));
            }
        } else if !off.extracted_paragraphs.is_empty() {
            lines.push(Line::from(Span::styled(
                "  ── Document Content ──",
                Style::default().fg(Color::DarkGray),
            )));
            for p in off
                .extracted_paragraphs
                .iter()
                .skip(offset)
                .take(visible_height.saturating_sub(4))
            {
                lines.push(Line::from(format!("  {}", p)));
            }
        }

        let p = Paragraph::new(lines);
        frame.render_widget(p, area);
        return;
    }

    // 6. Data preview (CSV, TSV, JSON, JSONL)
    if let Some(data) = &full_data.data {
        let mut lines = vec![
            Line::from(Span::styled(
                format!("  Format: {}", data.format_title),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::raw(""),
        ];

        match &data.kind {
            DataPreviewKind::Table {
                headers,
                rows,
                total_rows,
                ..
            } => {
                let header_line = headers.join(" │ ");
                lines.push(Line::from(Span::styled(
                    format!("  {} (Total rows: {})", header_line, total_rows),
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(Span::styled(
                    format!("  {}", "─".repeat(header_line.len().max(20))),
                    Style::default().fg(Color::DarkGray),
                )));
                for row in rows
                    .iter()
                    .skip(offset)
                    .take(visible_height.saturating_sub(5))
                {
                    lines.push(Line::from(format!("  {}", row.join(" │ "))));
                }
            }
            DataPreviewKind::Json {
                pretty_lines,
                is_valid,
                error_message,
                ..
            } => {
                if !is_valid {
                    if let Some(err) = error_message {
                        lines.push(Line::from(Span::styled(
                            format!("  {}", err),
                            Style::default().fg(theme.error),
                        )));
                    }
                }
                for l in pretty_lines
                    .iter()
                    .skip(offset)
                    .take(visible_height.saturating_sub(3))
                {
                    lines.push(Line::from(format!("  {}", l)));
                }
            }
            DataPreviewKind::Yaml {
                pretty_lines,
                is_valid,
                error_message,
                ..
            } => {
                if !is_valid {
                    if let Some(err) = error_message {
                        lines.push(Line::from(Span::styled(
                            format!("  {}", err),
                            Style::default().fg(theme.error),
                        )));
                    }
                }
                for l in pretty_lines
                    .iter()
                    .skip(offset)
                    .take(visible_height.saturating_sub(3))
                {
                    lines.push(Line::from(format!("  {}", l)));
                }
            }
            DataPreviewKind::JsonLines {
                records,
                total_records,
            } => {
                lines.push(Line::from(Span::styled(
                    format!("  Records: {}", total_records),
                    Style::default().fg(theme.accent),
                )));
                for (line_no, valid, txt) in records
                    .iter()
                    .skip(offset)
                    .take(visible_height.saturating_sub(4))
                {
                    let indicator = if *valid { "✓" } else { "✗" };
                    lines.push(Line::from(format!(
                        "  [{:>3}] {} {}",
                        line_no, indicator, txt
                    )));
                }
            }
        }

        let p = Paragraph::new(lines);
        frame.render_widget(p, area);
        return;
    }

    // 7. Archive preview
    if let Some(arch) = &full_data.archive {
        let mut lines = vec![
            Line::from(vec![
                Span::styled(
                    format!("  Archive: {}", arch.archive_type),
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(
                    "  (Entries: {}, Total: {})",
                    arch.total_entries,
                    ByteSize(arch.total_uncompressed_bytes)
                )),
            ]),
            Line::raw(""),
            Line::from(Span::styled(
                "  Name                          Size",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "  ──────────────────────────────────────────",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        for entry in arch
            .entries
            .iter()
            .skip(offset)
            .take(visible_height.saturating_sub(5))
        {
            let icon = if entry.is_dir { "📁 " } else { "📄 " };
            let size_str = if entry.is_dir {
                "--".to_string()
            } else {
                ByteSize(entry.uncompressed_size).to_string()
            };
            lines.push(Line::from(format!(
                "  {}{:<28} {:>10}",
                icon, entry.name, size_str
            )));
        }

        let p = Paragraph::new(lines);
        frame.render_widget(p, area);
        return;
    }

    // 8. SQLite database preview
    if let Some(db) = &full_data.database {
        let mut lines = vec![
            Line::from(Span::styled(
                format!("  SQLite 3 Database (Page size: {} bytes)", db.page_size),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::raw(""),
            Line::from(Span::styled(
                "  ── Tables ──",
                Style::default().fg(theme.accent),
            )),
        ];

        for t in &db.tables {
            lines.push(Line::from(format!("  • {} ─ {}", t.name, t.sql)));
        }

        if !db.sample_rows.is_empty() {
            lines.push(Line::raw(""));
            lines.push(Line::from(Span::styled(
                "  ── Sample Rows ──",
                Style::default().fg(theme.accent),
            )));
            for row in &db.sample_rows {
                lines.push(Line::from(format!("  {}", row)));
            }
        }

        let p = Paragraph::new(lines);
        frame.render_widget(p, area);
        return;
    }

    // 9. Markdown preview
    if let Some(md) = &full_data.markdown {
        let display_lines: Vec<Line> = md
            .formatted_lines
            .iter()
            .skip(offset)
            .take(visible_height)
            .map(|l| Line::from(vec![Span::raw("  "), Span::raw(l.as_str())]))
            .collect();
        let p = Paragraph::new(display_lines).wrap(ratatui::widgets::Wrap { trim: false });
        frame.render_widget(p, area);
        return;
    }

    // 10. Source Code and Configuration preview (with syntax highlighting and line numbers)
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
                        text.as_str(),
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
                    Span::raw(text.as_str()),
                ])
            })
            .collect()
    };

    let p = Paragraph::new(display_lines).scroll((0, preview.horizontal_offset as u16));
    frame.render_widget(p, area);
}

fn render_info_tab(
    frame: &mut Frame,
    area: Rect,
    full_data: &crate::preview::FullPreviewData,
    theme: &Theme,
) {
    let Some(meta) = &full_data.metadata else {
        let p =
            Paragraph::new("\n  No metadata available").style(Style::default().fg(Color::DarkGray));
        frame.render_widget(p, area);
        return;
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(
                "  File Name:    ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(&meta.file_name),
        ]),
        Line::from(vec![
            Span::styled("  Full Path:    ", Style::default().fg(theme.accent)),
            Span::raw(&meta.path),
        ]),
        Line::from(vec![
            Span::styled("  File Size:    ", Style::default().fg(theme.accent)),
            Span::raw(format!(
                "{} ({} bytes)",
                meta.size_formatted, meta.size_bytes
            )),
        ]),
        Line::from(vec![
            Span::styled("  MIME Type:    ", Style::default().fg(theme.accent)),
            Span::raw(&meta.mime_type),
        ]),
        Line::from(vec![
            Span::styled("  Category:     ", Style::default().fg(theme.accent)),
            Span::raw(full_data.format_info.description),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  Permissions:  ", Style::default().fg(theme.accent)),
            Span::raw(format!(
                "{} ({})",
                meta.permissions_str, meta.permissions_octal
            )),
        ]),
        Line::from(vec![
            Span::styled("  Ownership:    ", Style::default().fg(theme.accent)),
            Span::raw(format!("UID: {}  |  GID: {}", meta.uid, meta.gid)),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  Modified:     ", Style::default().fg(theme.accent)),
            Span::raw(&meta.modified_time),
        ]),
        Line::from(vec![
            Span::styled("  Accessed:     ", Style::default().fg(theme.accent)),
            Span::raw(&meta.accessed_time),
        ]),
        Line::from(vec![
            Span::styled("  Created:      ", Style::default().fg(theme.accent)),
            Span::raw(&meta.created_time),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  Symlink:      ", Style::default().fg(theme.accent)),
            Span::raw(if meta.is_symlink {
                format!(
                    "Yes -> {}",
                    meta.symlink_target.as_deref().unwrap_or("broken")
                )
            } else {
                "No".to_string()
            }),
        ]),
    ];

    let p = Paragraph::new(lines);
    frame.render_widget(p, area);
}

fn render_hex_tab(
    frame: &mut Frame,
    area: Rect,
    full_data: &crate::preview::FullPreviewData,
    offset: usize,
    visible_height: usize,
    theme: &Theme,
) {
    let Some(hex) = &full_data.hex else {
        let p = Paragraph::new("\n  Hex preview unavailable")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(p, area);
        return;
    };

    let mut lines = vec![
        Line::from(Span::styled(
            "  OFFSET    00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F   ASCII",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  ──────────────────────────────────────────────────────────────────────────",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    for row in hex
        .rows
        .iter()
        .skip(offset)
        .take(visible_height.saturating_sub(3))
    {
        let line_str = format!(
            "  {:08X}  {} {} │{}│",
            row.offset, row.hex_left, row.hex_right, row.ascii
        );
        lines.push(Line::from(line_str));
    }

    let p = Paragraph::new(lines);
    frame.render_widget(p, area);
}

fn render_actions_tab(
    frame: &mut Frame,
    area: Rect,
    full_data: &crate::preview::FullPreviewData,
    offset: usize,
    theme: &Theme,
) {
    let mut lines = vec![
        Line::from(Span::styled(
            "  Available File Actions (Press 1-9 to execute)",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::raw(""),
    ];

    for (idx, action) in full_data.actions.iter().skip(offset).enumerate() {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  [{}] ", idx + 1),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                action.label,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  ─  "),
            Span::styled(action.description, Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::raw(""));
    }

    let p = Paragraph::new(lines);
    frame.render_widget(p, area);
}

fn render_directory_preview(
    frame: &mut Frame,
    area: Rect,
    path: &std::path::Path,
    theme: &Theme,
    cached_data: Option<&crate::state::DirectoryPreviewData>,
) {
    let mut lines = Vec::new();

    let (items, total_count, dir_count, file_count, total_bytes) = if let Some(d) = cached_data {
        (
            &d.items,
            d.total_count,
            d.dir_count,
            d.file_count,
            d.total_bytes,
        )
    } else {
        let p = Paragraph::new("\n  󰉓 Loading directory metadata...")
            .style(Style::default().fg(theme.size_color));
        frame.render_widget(p, area);
        return;
    };

    let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
    lines.push(Line::from(vec![
        Span::styled(
            "  󰉓 ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{}/", dir_name),
            Style::default()
                .fg(theme.header_fg)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled("  Items:     ", Style::default().fg(theme.accent)),
        Span::styled(
            format!("{} entries", total_count),
            Style::default()
                .fg(theme.header_fg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(
            " ({} folders, {} files, {})",
            dir_count,
            file_count,
            ByteSize(total_bytes)
        )),
    ]));

    lines.push(Line::from(vec![
        Span::styled("  Actions:   ", Style::default().fg(theme.accent)),
        Span::styled("[Enter] Open ", Style::default().fg(theme.status_fg)),
        Span::styled("[F2] Rename ", Style::default().fg(theme.status_fg)),
        Span::styled("[a] New File ", Style::default().fg(theme.status_fg)),
        Span::styled("[A] New Dir", Style::default().fg(theme.status_fg)),
    ]));

    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        "  Contents (Recent / Top Entries):",
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )));

    let max_display = (area.height as usize).saturating_sub(7).min(items.len());
    for (i, (name, is_dir, size)) in items.iter().take(max_display).enumerate() {
        let is_last = i + 1 == max_display && items.len() <= max_display;
        let branch = if is_last {
            "    └── "
        } else {
            "    ├── "
        };
        if *is_dir {
            lines.push(Line::from(vec![
                Span::styled(branch, Style::default().fg(theme.hidden_color)),
                Span::styled("📁 ", Style::default().fg(theme.dir_color)),
                Span::styled(
                    format!("{}/", name),
                    Style::default()
                        .fg(theme.dir_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        } else {
            let size_pill = format!(" ({})", ByteSize(*size));
            lines.push(Line::from(vec![
                Span::styled(branch, Style::default().fg(theme.hidden_color)),
                Span::styled("📄 ", Style::default().fg(theme.file_color)),
                Span::raw(name.as_str()),
                Span::styled(size_pill, Style::default().fg(theme.size_color)),
            ]));
        }
    }

    if items.len() > max_display {
        let remaining = items.len() - max_display;
        lines.push(Line::from(Span::styled(
            format!("    ... and {} more items", remaining),
            Style::default().fg(Color::DarkGray),
        )));
    }

    let p = Paragraph::new(lines);
    frame.render_widget(p, area);
}
