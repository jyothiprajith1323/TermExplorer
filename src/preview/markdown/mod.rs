//! High-fidelity Markdown document parser and renderer using pulldown-cmark,
//! syntax-highlighted code blocks via Syntect, table formatting, and table of contents.

use crate::editor::syntax::highlight_file;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct TocItem {
    pub level: usize,
    pub title: String,
    pub line_number: usize,
}

#[derive(Debug, Clone)]
pub struct MarkdownPreview {
    pub rendered_lines: Vec<Line<'static>>,
    pub formatted_lines: Vec<String>, // Plain text compatibility
    pub toc: Vec<TocItem>,
    pub total_lines: usize,
    pub heading_count: usize,
}

impl MarkdownPreview {
    pub fn load(path: &Path, max_lines: usize) -> Self {
        let content = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                return Self {
                    rendered_lines: vec![Line::from(vec![Span::styled(
                        format!("[Error reading markdown: {e}]"),
                        Style::default().fg(Color::Red),
                    )])],
                    formatted_lines: vec![format!("[Error reading markdown: {e}]")],
                    toc: Vec::new(),
                    total_lines: 0,
                    heading_count: 0,
                };
            }
        };

        Self::parse_markdown(&content, max_lines)
    }

    pub fn parse_markdown(content: &str, max_lines: usize) -> Self {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

        let parser = Parser::new_ext(content, options);

        let mut rendered_lines: Vec<Line<'static>> = Vec::new();
        let mut current_spans: Vec<Span<'static>> = Vec::new();
        let mut toc: Vec<TocItem> = Vec::new();

        let mut bold = false;
        let mut italic = false;
        let mut in_code_block = false;
        let mut code_lang = String::new();
        let mut code_buffer = String::new();
        let mut in_heading = false;
        let mut heading_level = 1;
        let mut heading_text = String::new();
        let mut list_depth: usize = 0;
        let mut in_table = false;
        let mut table_headers: Vec<String> = Vec::new();
        let mut table_rows: Vec<Vec<String>> = Vec::new();
        let mut current_row: Vec<String> = Vec::new();
        let mut current_cell = String::new();
        let mut in_table_head = false;

        let mocha_mauve = Color::Rgb(203, 166, 247);
        let mocha_blue = Color::Rgb(137, 180, 250);
        let mocha_yellow = Color::Rgb(249, 226, 175);
        let mocha_green = Color::Rgb(166, 227, 161);
        let mocha_peach = Color::Rgb(250, 179, 135);
        let mocha_subtext = Color::Rgb(166, 173, 200);

        for event in parser {
            if rendered_lines.len() >= max_lines {
                break;
            }

            match event {
                Event::Start(tag) => match tag {
                    Tag::Heading { level, .. } => {
                        in_heading = true;
                        heading_level = match level {
                            HeadingLevel::H1 => 1,
                            HeadingLevel::H2 => 2,
                            HeadingLevel::H3 => 3,
                            HeadingLevel::H4 => 4,
                            HeadingLevel::H5 => 5,
                            HeadingLevel::H6 => 6,
                        };
                        heading_text.clear();
                        if !current_spans.is_empty() {
                            rendered_lines.push(Line::from(std::mem::take(&mut current_spans)));
                        }
                        rendered_lines.push(Line::default()); // Blank line before heading
                    }
                    Tag::Paragraph => {
                        if !current_spans.is_empty() {
                            rendered_lines.push(Line::from(std::mem::take(&mut current_spans)));
                        }
                    }
                    Tag::CodeBlock(kind) => {
                        in_code_block = true;
                        code_buffer.clear();
                        code_lang = match kind {
                            pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                            pulldown_cmark::CodeBlockKind::Indented => String::new(),
                        };
                        if !current_spans.is_empty() {
                            rendered_lines.push(Line::from(std::mem::take(&mut current_spans)));
                        }
                        let header_str = if code_lang.is_empty() {
                            "  ─── Code ───".to_string()
                        } else {
                            format!("  ─── Code [{code_lang}] ───")
                        };
                        rendered_lines.push(Line::from(vec![Span::styled(
                            header_str,
                            Style::default()
                                .fg(mocha_subtext)
                                .add_modifier(Modifier::DIM),
                        )]));
                    }
                    Tag::List(_) => {
                        list_depth += 1;
                    }
                    Tag::Item => {
                        if !current_spans.is_empty() {
                            rendered_lines.push(Line::from(std::mem::take(&mut current_spans)));
                        }
                        let indent = "  ".repeat(list_depth.saturating_sub(1));
                        current_spans.push(Span::styled(
                            format!("{indent}• "),
                            Style::default().fg(mocha_mauve),
                        ));
                    }
                    Tag::Table(_) => {
                        in_table = true;
                        table_headers.clear();
                        table_rows.clear();
                    }
                    Tag::TableHead => {
                        in_table_head = true;
                        current_row.clear();
                    }
                    Tag::TableRow => {
                        current_row.clear();
                    }
                    Tag::TableCell => {
                        current_cell.clear();
                    }
                    Tag::BlockQuote(_) => {
                        if !current_spans.is_empty() {
                            rendered_lines.push(Line::from(std::mem::take(&mut current_spans)));
                        }
                        current_spans.push(Span::styled(
                            "  ▎ ",
                            Style::default()
                                .fg(mocha_peach)
                                .add_modifier(Modifier::BOLD),
                        ));
                    }
                    Tag::Emphasis => italic = true,
                    Tag::Strong => bold = true,
                    _ => {}
                },
                Event::End(tag_end) => match tag_end {
                    TagEnd::Heading(_) => {
                        in_heading = false;
                        let line_num = rendered_lines.len() + 1;
                        toc.push(TocItem {
                            level: heading_level,
                            title: heading_text.clone(),
                            line_number: line_num,
                        });

                        let prefix = match heading_level {
                            1 => "# ",
                            2 => "## ",
                            3 => "### ",
                            4 => "#### ",
                            5 => "##### ",
                            _ => "###### ",
                        };
                        let style = match heading_level {
                            1 => Style::default()
                                .fg(mocha_mauve)
                                .add_modifier(Modifier::BOLD),
                            2 => Style::default().fg(mocha_blue).add_modifier(Modifier::BOLD),
                            3 => Style::default()
                                .fg(mocha_yellow)
                                .add_modifier(Modifier::BOLD),
                            _ => Style::default()
                                .fg(mocha_peach)
                                .add_modifier(Modifier::BOLD),
                        };

                        rendered_lines.push(Line::from(vec![
                            Span::styled(prefix, Style::default().fg(mocha_subtext)),
                            Span::styled(heading_text.clone(), style),
                        ]));

                        // Add decorative underline for H1 and H2
                        let bar_len = (heading_text.len() + prefix.len()).clamp(4, 70);
                        if heading_level == 1 {
                            rendered_lines.push(Line::from(vec![Span::styled(
                                "━".repeat(bar_len),
                                Style::default().fg(mocha_mauve).add_modifier(Modifier::DIM),
                            )]));
                        } else if heading_level == 2 {
                            rendered_lines.push(Line::from(vec![Span::styled(
                                "─".repeat(bar_len),
                                Style::default().fg(mocha_blue).add_modifier(Modifier::DIM),
                            )]));
                        }
                        rendered_lines.push(Line::default());
                    }
                    TagEnd::Paragraph => {
                        if !current_spans.is_empty() {
                            rendered_lines.push(Line::from(std::mem::take(&mut current_spans)));
                        }
                        rendered_lines.push(Line::default());
                    }
                    TagEnd::CodeBlock => {
                        in_code_block = false;
                        // Highlight buffer lines using Syntect via highlight_file
                        let dummy_ext = if code_lang.is_empty() {
                            "rs"
                        } else {
                            &code_lang
                        };
                        let dummy_path = std::path::Path::new("code").with_extension(dummy_ext);
                        let span_lines = highlight_file(&dummy_path, &code_buffer);
                        for line_spans in span_lines {
                            let mut ratatui_spans = vec![Span::raw("  │ ")];
                            for (r, g, b, text) in line_spans {
                                ratatui_spans.push(Span::styled(
                                    text,
                                    Style::default().fg(Color::Rgb(r, g, b)),
                                ));
                            }
                            rendered_lines.push(Line::from(ratatui_spans));
                        }
                        rendered_lines.push(Line::from(vec![Span::styled(
                            "  └───────────────",
                            Style::default()
                                .fg(mocha_subtext)
                                .add_modifier(Modifier::DIM),
                        )]));
                        rendered_lines.push(Line::default());
                    }
                    TagEnd::List(_) => {
                        list_depth = list_depth.saturating_sub(1);
                        if list_depth == 0 {
                            rendered_lines.push(Line::default());
                        }
                    }
                    TagEnd::Item => {
                        if !current_spans.is_empty() {
                            rendered_lines.push(Line::from(std::mem::take(&mut current_spans)));
                        }
                    }
                    TagEnd::TableCell => {
                        current_row.push(current_cell.clone());
                    }
                    TagEnd::TableHead => {
                        in_table_head = false;
                        table_headers = current_row.clone();
                    }
                    TagEnd::TableRow => {
                        if !in_table_head {
                            table_rows.push(current_row.clone());
                        }
                    }
                    TagEnd::Table => {
                        in_table = false;
                        // Format table into rendered lines
                        if !table_headers.is_empty() {
                            let mut col_widths = vec![4; table_headers.len()];
                            for (i, h) in table_headers.iter().enumerate() {
                                col_widths[i] = col_widths[i].max(h.len());
                            }
                            for row in &table_rows {
                                for (i, cell) in row.iter().enumerate() {
                                    if i < col_widths.len() {
                                        col_widths[i] = col_widths[i].max(cell.len());
                                    }
                                }
                            }

                            // Header line
                            let mut header_spans = vec![Span::raw("  ")];
                            for (i, h) in table_headers.iter().enumerate() {
                                let w = col_widths[i];
                                header_spans.push(Span::styled(
                                    format!(" {h:<w$} │"),
                                    Style::default().fg(mocha_blue).add_modifier(Modifier::BOLD),
                                ));
                            }
                            rendered_lines.push(Line::from(header_spans));

                            // Divider line
                            let mut div_spans = vec![Span::raw("  ")];
                            for &w in &col_widths {
                                div_spans.push(Span::styled(
                                    format!("─{}─┼", "─".repeat(w)),
                                    Style::default().fg(mocha_subtext),
                                ));
                            }
                            rendered_lines.push(Line::from(div_spans));

                            // Data rows
                            for r in &table_rows {
                                let mut row_spans = vec![Span::raw("  ")];
                                for (i, cell) in r.iter().enumerate() {
                                    let w = if i < col_widths.len() {
                                        col_widths[i]
                                    } else {
                                        10
                                    };
                                    row_spans.push(Span::raw(format!(" {cell:<w$} │")));
                                }
                                rendered_lines.push(Line::from(row_spans));
                            }
                            rendered_lines.push(Line::default());
                        }
                    }
                    TagEnd::BlockQuote(_) => {
                        if !current_spans.is_empty() {
                            rendered_lines.push(Line::from(std::mem::take(&mut current_spans)));
                        }
                        rendered_lines.push(Line::default());
                    }
                    TagEnd::Emphasis => italic = false,
                    TagEnd::Strong => bold = false,
                    _ => {}
                },
                Event::Text(t) => {
                    if in_code_block {
                        code_buffer.push_str(&t);
                    } else if in_heading {
                        heading_text.push_str(&t);
                    } else if in_table {
                        current_cell.push_str(&t);
                    } else {
                        let mut style = Style::default();
                        if bold {
                            style = style.add_modifier(Modifier::BOLD);
                        }
                        if italic {
                            style = style.add_modifier(Modifier::ITALIC);
                        }
                        current_spans.push(Span::styled(t.to_string(), style));
                    }
                }
                Event::Code(c) => {
                    if in_table {
                        current_cell.push_str(&format!("`{c}`"));
                    } else {
                        current_spans.push(Span::styled(
                            format!(" `{c}` "),
                            Style::default().fg(mocha_peach).bg(Color::Rgb(49, 50, 68)),
                        ));
                    }
                }
                Event::Rule => {
                    rendered_lines.push(Line::from(vec![Span::styled(
                        "  ────────────────────────────────────────────────────────",
                        Style::default()
                            .fg(mocha_subtext)
                            .add_modifier(Modifier::DIM),
                    )]));
                    rendered_lines.push(Line::default());
                }
                Event::TaskListMarker(checked) => {
                    let mark = if checked { "[x] " } else { "[ ] " };
                    let color = if checked { mocha_green } else { mocha_peach };
                    current_spans.push(Span::styled(
                        mark,
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ));
                }
                _ => {}
            }
        }

        if !current_spans.is_empty() {
            rendered_lines.push(Line::from(current_spans));
        }

        let formatted_lines: Vec<String> = rendered_lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .collect();

        let total_lines = rendered_lines.len();
        let heading_count = toc.len();

        Self {
            rendered_lines,
            formatted_lines,
            toc,
            total_lines,
            heading_count,
        }
    }
}
