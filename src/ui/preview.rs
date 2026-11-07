//! File preview panel rendering with tabs for Preview, Information, Hex, and Actions.

use crate::preview::data::DataPreviewKind;
use crate::preview::manager::PreviewTab;
use crate::state::{AppState, FocusedPanel};
use crate::theme::Theme;
use bytesize::ByteSize;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
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

    let tab_str = match preview.active_tab {
        PreviewTab::Preview => "[Preview]  Info   Hex   Actions",
        PreviewTab::Information => " Preview  [Info]  Hex   Actions",
        PreviewTab::Hex => " Preview   Info  [Hex]  Actions",
        PreviewTab::Actions => " Preview   Info   Hex  [Actions]",
    };

    let title = format!(" 󰍉 {}  ─  {} ", file_name, tab_str);

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

    let Some(path) = &preview.path else {
        let hint =
            Paragraph::new("\n  Select a file or folder to preview\n\n  Tabs: [ / ] or h / l switch tabs")
                .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(hint, inner);
        return;
    };

    if path.is_dir() {
        render_directory_preview(frame, inner, path, theme);
        return;
    }

    let full_data = match &preview.full_data {
        Some(d) => d,
        None => {
            let hint = Paragraph::new("\n  Select a file to preview")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(hint, inner);
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
            ]),
            Line::from(vec![
                Span::styled("  Format:     ", Style::default().fg(theme.accent)),
                Span::raw(format!("{}  |  {}", img.format_name, img.color_type)),
            ]),
            Line::raw(""),
        ];

        // Render halfblock image pixels if available (using static "▀" to avoid thousands of per-frame String allocations)
        for row in &img.halfblock_grid {
            let spans: Vec<Span> = row
                .iter()
                .map(|&(fg, bg, _ch)| Span::styled("▀", Style::default().fg(fg).bg(bg)))
                .collect();
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
            Line::raw(""),
        ];

        if !vid.thumbnail_grid.is_empty() {
            lines.push(Line::from(Span::styled(
                "  [Extracted First Frame]",
                Style::default().fg(theme.success),
            )));
            for row in &vid.thumbnail_grid {
                let spans: Vec<Span> = row
                    .iter()
                    .map(|&(fg, bg, _ch)| {
                        Span::styled("▀", Style::default().fg(fg).bg(bg))
                    })
                    .collect();
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
            ]),
            Line::from(vec![
                Span::styled("  Pages: ", Style::default().fg(theme.accent)),
                Span::raw(format!("{}", pdf.page_count.unwrap_or(1))),
                Span::raw("  |  Title: "),
                Span::raw(pdf.title.as_deref().unwrap_or("Untitled")),
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
                .take(visible_height.saturating_sub(4))
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
        let p = Paragraph::new(display_lines);
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

    let p = Paragraph::new(display_lines);
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
) {
    let mut lines = Vec::new();

    let (entries, total_count, dir_count, file_count) = match std::fs::read_dir(path) {
        Ok(read_dir) => {
            let mut items = Vec::new();
            let mut dirs = 0;
            let mut files = 0;
            for entry in read_dir.flatten() {
                let file_type = entry.file_type().ok();
                let is_dir = file_type.map(|t| t.is_dir()).unwrap_or(false);
                if is_dir {
                    dirs += 1;
                } else {
                    files += 1;
                }
                items.push((entry.file_name().to_string_lossy().to_string(), is_dir));
            }
            items.sort_by(|a, b| {
                match (a.1, b.1) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.0.to_lowercase().cmp(&b.0.to_lowercase()),
                }
            });
            let total = items.len();
            (items, total, dirs, files)
        }
        Err(e) => {
            let p = Paragraph::new(format!("\n  Cannot read directory: {}", e))
                .style(Style::default().fg(theme.error));
            frame.render_widget(p, area);
            return;
        }
    };

    let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
    lines.push(Line::from(vec![
        Span::styled(
            "  Directory: ",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{}/", dir_name),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Items:     ", Style::default().fg(theme.accent)),
        Span::raw(format!(
            "{} entries ({} folders, {} files)",
            total_count, dir_count, file_count
        )),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Action:    ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "Press [→] or [Enter] to enter directory",
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        "  Contents:",
        Style::default().fg(theme.accent),
    )));

    let max_display = (area.height as usize).saturating_sub(6).min(entries.len());
    for (name, is_dir) in entries.iter().take(max_display) {
        if *is_dir {
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled("📁 ", Style::default().fg(theme.dir_color)),
                Span::styled(
                    format!("{}/", name),
                    Style::default()
                        .fg(theme.dir_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled("📄 ", Style::default().fg(theme.file_color)),
                Span::raw(name.as_str()),
            ]));
        }
    }

    if entries.len() > max_display {
        let remaining = entries.len() - max_display;
        lines.push(Line::from(Span::styled(
            format!("    ... and {} more items", remaining),
            Style::default().fg(Color::DarkGray),
        )));
    }

    let p = Paragraph::new(lines);
    frame.render_widget(p, area);
}
