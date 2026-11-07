//! Workstation Command Bar — top navigation and workspace header.

use crate::monitor::MonitorSnapshot;
use crate::state::AppState;
use crate::theme::Theme;
use crate::ui::hit_test::{AppAction, HitRegistry, WidgetId};
use bytesize::ByteSize;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render_header(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    monitor_snap: Option<&MonitorSnapshot>,
    theme: &Theme,
    hit_registry: &mut HitRegistry,
) {
    if area.height < 1 || area.width < 10 {
        return;
    }

    let header_bg = theme.header_bg.unwrap_or(Color::Rgb(20, 24, 32));
    // Clear the full header line with consistent background
    frame.render_widget(
        Paragraph::new(" ".repeat(area.width as usize)).style(Style::default().bg(header_bg)),
        area,
    );

    let width = area.width;

    // ── 1. Left Section: Brand & Workspace ────────────────────────────────────
    let brand_w = if width >= 90 {
        36u16.min(area.width)
    } else {
        18u16.min(area.width)
    };
    let brand_rect = Rect {
        x: area.x,
        y: area.y,
        width: brand_w,
        height: 1,
    };
    hit_registry.register_button(
        WidgetId::HeaderBrand,
        Rect {
            x: area.x,
            y: area.y,
            width: 17u16.min(area.width),
            height: 1,
        },
        AppAction::OpenHelp,
        10,
    );

    let mut left_spans = vec![
        Span::styled(
            " ◆ TERMEXPLORER ",
            Style::default()
                .fg(Color::Black)
                .bg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ];

    if width >= 90 {
        left_spans.push(Span::styled(
            format!("󰲋 {} ", state.workspace.name),
            Style::default()
                .fg(theme.header_fg)
                .add_modifier(Modifier::BOLD),
        ));
        left_spans.push(Span::styled(
            format!("[{}] ", state.workspace.project_kind.badge()),
            Style::default().fg(theme.accent),
        ));
        left_spans.push(Span::styled("│ ", Style::default().fg(theme.border)));
    }
    frame.render_widget(
        Paragraph::new(Line::from(left_spans)).style(Style::default().bg(header_bg)),
        brand_rect,
    );

    // ── 2. Right Section Buttons: Settings, Plugins, Search ───────────────────
    let (search_txt, search_w, plugins_txt, plugins_w, settings_txt, settings_w) = if width >= 90 {
        (
            "[ 󰍉 Search ]",
            14u16,
            "[ ◇ Plugins ]",
            14u16,
            "[ ⚙ Settings ]",
            15u16,
        )
    } else if width >= 55 {
        ("[Search]", 8u16, "[Plugins]", 9u16, "[Settings]", 10u16)
    } else {
        ("[󰍉]", 4u16, "[◇]", 4u16, "[⚙]", 4u16)
    };

    let settings_start = if width >= 90 {
        area.x + area.width.saturating_sub(16)
    } else if width >= 55 {
        area.x + area.width.saturating_sub(12)
    } else {
        area.x + area.width.saturating_sub(5)
    };
    let settings_rect = Rect {
        x: settings_start,
        y: area.y,
        width: settings_w.min(
            area.width
                .saturating_sub(settings_start.saturating_sub(area.x)),
        ),
        height: 1,
    };
    hit_registry.register_button(
        WidgetId::HeaderSettings,
        settings_rect,
        AppAction::OpenSettings,
        10,
    );

    let plugins_start = if width >= 90 {
        area.x + area.width.saturating_sub(32)
    } else if width >= 55 {
        area.x + area.width.saturating_sub(23)
    } else {
        area.x + area.width.saturating_sub(10)
    };
    let plugins_rect = Rect {
        x: plugins_start,
        y: area.y,
        width: plugins_w.min(settings_start.saturating_sub(plugins_start)),
        height: 1,
    };
    hit_registry.register_button(
        WidgetId::HeaderPlugins,
        plugins_rect,
        AppAction::OpenPluginCenter,
        10,
    );

    let search_start = if width >= 90 {
        area.x + area.width.saturating_sub(48)
    } else if width >= 55 {
        area.x + area.width.saturating_sub(33)
    } else {
        area.x + area.width.saturating_sub(15)
    };
    let search_rect = Rect {
        x: search_start,
        y: area.y,
        width: search_w.min(plugins_start.saturating_sub(search_start)),
        height: 1,
    };
    hit_registry.register_button(
        WidgetId::HeaderSearch,
        search_rect,
        AppAction::OpenSearch,
        10,
    );

    frame.render_widget(
        Paragraph::new(settings_txt).style(
            Style::default()
                .fg(theme.header_fg)
                .bg(header_bg)
                .add_modifier(Modifier::BOLD),
        ),
        settings_rect,
    );
    frame.render_widget(
        Paragraph::new(plugins_txt).style(
            Style::default()
                .fg(theme.success)
                .bg(header_bg)
                .add_modifier(Modifier::BOLD),
        ),
        plugins_rect,
    );
    frame.render_widget(
        Paragraph::new(search_txt).style(
            Style::default()
                .fg(theme.accent)
                .bg(header_bg)
                .add_modifier(Modifier::BOLD),
        ),
        search_rect,
    );

    // ── 3. Telemetry Pill (when wide enough) ──────────────────────────────────
    let mut center_end_x = search_start;
    if width >= 120 {
        if let Some(snap) = monitor_snap {
            let telem_txt = format!(
                "󰻠 {:.0}% 󰍛 {}",
                snap.cpu.global_usage,
                ByteSize(snap.mem.used)
            );
            let telem_w = (telem_txt.chars().count() + 4) as u16;
            let telem_x = search_start.saturating_sub(telem_w);
            if telem_x > brand_rect.x + brand_rect.width + 10 {
                let telem_rect = Rect {
                    x: telem_x,
                    y: area.y,
                    width: telem_w,
                    height: 1,
                };
                hit_registry.register_button(
                    WidgetId::HeaderTelemetry,
                    telem_rect,
                    AppAction::OpenHelp,
                    5,
                );
                let telem_line = Line::from(vec![
                    Span::styled(" │ ", Style::default().fg(theme.border)),
                    Span::styled(telem_txt, Style::default().fg(theme.status_fg)),
                    Span::styled(" ", Style::default().fg(theme.border)),
                ]);
                frame.render_widget(
                    Paragraph::new(telem_line).style(Style::default().bg(header_bg)),
                    telem_rect,
                );
                center_end_x = telem_x;
            }
        }
    }

    // ── 4. Center Section: Breadcrumbs & Git ──────────────────────────────────
    let center_x = area.x + brand_w;
    if center_end_x > center_x + 5 {
        let center_w = center_end_x.saturating_sub(center_x);
        let center_rect = Rect {
            x: center_x,
            y: area.y,
            width: center_w,
            height: 1,
        };

        let mut center_spans = Vec::new();
        let current_dir = &state.explorer.current_dir;
        let home_dir = dirs::home_dir();

        let display_path = if let Some(home) = &home_dir {
            if let Ok(rel) = current_dir.strip_prefix(home) {
                format!("~/{}", rel.display())
            } else {
                current_dir.display().to_string()
            }
        } else {
            current_dir.display().to_string()
        };

        let parts: Vec<&str> = display_path.split('/').filter(|s| !s.is_empty()).collect();
        if width >= 110 && !parts.is_empty() {
            for (i, part) in parts.iter().enumerate() {
                if i > 0 {
                    center_spans.push(Span::styled(" › ", Style::default().fg(theme.hidden_color)));
                }
                let is_last = i + 1 == parts.len();
                if is_last {
                    center_spans.push(Span::styled(
                        *part,
                        Style::default()
                            .fg(theme.header_fg)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    center_spans.push(Span::styled(*part, Style::default().fg(theme.size_color)));
                }
            }
        } else {
            let folder_name = current_dir
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "/".to_string());
            center_spans.push(Span::styled(
                format!("󰝰 {} ", folder_name),
                Style::default()
                    .fg(theme.header_fg)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        // Git branch indicator in header
        if state.git_status.is_repo {
            center_spans.push(Span::styled(" │ ", Style::default().fg(theme.border)));
            let git_color = if state.git_status.is_dirty {
                theme.warning
            } else {
                theme.success
            };
            let dirty_suffix = if state.git_status.is_dirty {
                let total_dirty = state.git_status.staged_files.len()
                    + state.git_status.unstaged_files.len()
                    + state.git_status.untracked_files.len();
                format!(" ● {}", total_dirty)
            } else {
                " ✓".to_string()
            };
            center_spans.push(Span::styled(
                format!("󰊢 {}{}", state.git_status.branch, dirty_suffix),
                Style::default().fg(git_color).add_modifier(Modifier::BOLD),
            ));
            hit_registry.register_button(
                WidgetId::HeaderGit,
                center_rect,
                AppAction::OpenGitDashboard,
                5,
            );
        }

        frame.render_widget(
            Paragraph::new(Line::from(center_spans)).style(Style::default().bg(header_bg)),
            center_rect,
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderAction {
    Brand,
    Search,
    Plugins,
    Settings,
}

pub fn header_click_action(x: u16, width: u16) -> Option<HeaderAction> {
    if x < 18 {
        return Some(HeaderAction::Brand);
    }
    if width >= 90 {
        let settings_start = width.saturating_sub(16);
        let plugins_start = width.saturating_sub(32);
        let search_start = width.saturating_sub(48);
        if x >= settings_start {
            Some(HeaderAction::Settings)
        } else if x >= plugins_start {
            Some(HeaderAction::Plugins)
        } else if x >= search_start {
            Some(HeaderAction::Search)
        } else {
            None
        }
    } else if width >= 55 {
        let settings_start = width.saturating_sub(12);
        let plugins_start = width.saturating_sub(23);
        let search_start = width.saturating_sub(33);
        if x >= settings_start {
            Some(HeaderAction::Settings)
        } else if x >= plugins_start {
            Some(HeaderAction::Plugins)
        } else if x >= search_start {
            Some(HeaderAction::Search)
        } else {
            None
        }
    } else {
        let settings_start = width.saturating_sub(5);
        let plugins_start = width.saturating_sub(10);
        let search_start = width.saturating_sub(15);
        if x >= settings_start {
            Some(HeaderAction::Settings)
        } else if x >= plugins_start {
            Some(HeaderAction::Plugins)
        } else if x >= search_start {
            Some(HeaderAction::Search)
        } else {
            None
        }
    }
}
