//! Sidebar rendering — pinned folders and built-in shortcuts.

use crate::state::{AppState, FocusedPanel, SidebarSection};
use crate::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
    Frame,
};
use std::path::PathBuf;

/// Fixed OS shortcuts shown in the sidebar.
pub fn os_shortcuts() -> Vec<(&'static str, PathBuf)> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/root"));
    let mut v = vec![
        ("  Home", home.clone()),
        ("  Documents", home.join("Documents")),
        ("  Downloads", home.join("Downloads")),
        ("  Desktop", home.join("Desktop")),
        ("  Pictures", home.join("Pictures")),
        ("  Videos", home.join("Videos")),
        ("  Music", home.join("Music")),
        (" /", PathBuf::from("/")),
        (" /etc", PathBuf::from("/etc")),
        (" /var", PathBuf::from("/var")),
        (" /tmp", PathBuf::from("/tmp")),
        (" /usr", PathBuf::from("/usr")),
        (" /opt", PathBuf::from("/opt")),
        (" /mnt", PathBuf::from("/mnt")),
        (" /media", PathBuf::from("/media")),
    ];
    // Keep only paths that exist.
    v.retain(|(_, p)| p.exists());
    v
}

pub fn render_sidebar(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let focused = state.focused == FocusedPanel::Sidebar;
    let border_style = if focused {
        theme.border_focused_style()
    } else {
        theme.border_style()
    };

    let block = Block::default()
        .title(Span::styled(" 󰊢 SIDEBAR ", theme.header_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 2 {
        return;
    }

    // ── Sections ──────────────────────────────────────────────────────────
    use ratatui::layout::{Constraint, Direction, Layout};

    let pinned_count = state.config.pinned_folders.len().max(1); // at least header
    let shortcuts = os_shortcuts();
    let recent = &state.recent_dirs.dirs;

    // Rough height split.
    let pinned_h = (pinned_count as u16 + 3).min(inner.height / 2);
    let recent_h = if !recent.is_empty() {
        (recent.len() as u16 + 2).min(5)
    } else {
        0
    };
    let shortcut_h = inner.height.saturating_sub(pinned_h + recent_h);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(pinned_h),
            Constraint::Length(recent_h),
            Constraint::Length(shortcut_h),
        ])
        .split(inner);

    // ── Pinned folders ─────────────────────────────────────────────────────
    let pinned_items: Vec<ListItem> = {
        let mut items = vec![ListItem::new(Line::from(Span::styled(
            "── Pinned ──────",
            theme.accent_style(),
        )))];
        if state.config.pinned_folders.is_empty() {
            items.push(ListItem::new(Line::from(Span::styled(
                "  (none — Alt+P to pin)",
                Style::default().fg(Color::DarkGray),
            ))));
        } else {
            for pf in &state.config.pinned_folders {
                let exists = pf.path.exists();
                let style = if exists {
                    theme.normal_style()
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                items.push(ListItem::new(Line::from(vec![
                    Span::raw("  📌 "),
                    Span::styled(&pf.name, style),
                ])));
            }
        }
        items
    };
    let mut pinned_state = ListState::default();
    if focused
        && state.sidebar.section == SidebarSection::Pinned
        && !state.config.pinned_folders.is_empty()
    {
        pinned_state.select(Some(state.sidebar.selected_index + 1));
    }
    let pinned_list = List::new(pinned_items)
        .highlight_style(theme.selection_style())
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(pinned_list, sections[0], &mut pinned_state);

    // ── Recent dirs ────────────────────────────────────────────────────────
    if recent_h > 0 {
        let recent_items: Vec<ListItem> = {
            let mut items = vec![ListItem::new(Line::from(Span::styled(
                "── Recent ───────",
                theme.accent_style(),
            )))];
            for dir in recent.iter().take(4) {
                let name = dir
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| dir.to_string_lossy().to_string());
                items.push(ListItem::new(Line::from(vec![
                    Span::raw("  🕐 "),
                    Span::styled(name, theme.normal_style()),
                ])));
            }
            items
        };
        frame.render_widget(List::new(recent_items), sections[1]);
    }

    // ── Shortcuts ──────────────────────────────────────────────────────────
    let shortcut_items: Vec<ListItem> = {
        let mut items = vec![ListItem::new(Line::from(Span::styled(
            "── Shortcuts ────",
            theme.accent_style(),
        )))];
        for (name, path) in &shortcuts {
            let exists = path.exists();
            let style = if exists {
                theme.normal_style()
            } else {
                Style::default().fg(Color::DarkGray)
            };
            items.push(ListItem::new(Line::from(Span::styled(*name, style))));
        }
        items
    };
    // Highlight selected shortcut if sidebar is focused.
    let mut list_state = ListState::default();
    if focused && state.sidebar.section == SidebarSection::Shortcuts && !shortcuts.is_empty() {
        list_state.select(Some(state.sidebar.selected_index + 1));
    }
    let list = List::new(shortcut_items)
        .highlight_style(theme.selection_style())
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(list, sections[2], &mut list_state);
}
