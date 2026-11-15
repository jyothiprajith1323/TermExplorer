//! Git Dashboard and Version Control panel — developer workstation Git client.

use crate::git::{GitBranch, GitCommitInfo, GitRepoStatus};
use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

/// View state for the comprehensive Git Workbench modal.
pub struct GitWorkbenchView<'a> {
    pub active_tab: usize, // 0 = Status, 1 = Diff, 2 = Branches, 3 = History
    pub selected_idx: usize,
    pub diff_content: &'a [Line<'static>],
    pub diff_scroll: usize,
    pub branches: &'a [GitBranch],
    pub branch_selected: usize,
    pub history: &'a [GitCommitInfo],
    pub history_selected: usize,
}

/// Backward-compatible entry point for tests and basic callers.
pub fn render_git_panel(
    frame: &mut Frame,
    area: Rect,
    git_status: &GitRepoStatus,
    selected_idx: usize,
    theme: &Theme,
) {
    let view = GitWorkbenchView {
        active_tab: 0,
        selected_idx,
        diff_content: &[],
        diff_scroll: 0,
        branches: &[],
        branch_selected: 0,
        history: &git_status.recent_commits,
        history_selected: 0,
    };
    render_git_workbench(frame, area, git_status, &view, theme);
}

/// Render the multi-tab Git Workbench modal.
pub fn render_git_workbench(
    frame: &mut Frame,
    area: Rect,
    git_status: &GitRepoStatus,
    view: &GitWorkbenchView,
    theme: &Theme,
) {
    let width = (area.width * 85 / 100).clamp(70, 110);
    let height = 28u16.min(area.height.saturating_sub(2));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;

    let dlg_area = Rect {
        x,
        y,
        width,
        height,
    };

    frame.render_widget(Clear, dlg_area);

    let branch_status = if git_status.is_dirty {
        "● Dirty"
    } else {
        "✓ Clean"
    };

    let ahead_behind = if git_status.ahead > 0 || git_status.behind > 0 {
        format!(" [+{}/-{}]", git_status.ahead, git_status.behind)
    } else {
        String::new()
    };

    let title = if width >= 80 {
        format!(
            " 󰊢 GIT WORKBENCH  ─  Branch: {}{}{}  │  [Tab] Switch Tab  [Esc] Close ",
            git_status.branch,
            ahead_behind,
            if git_status.is_dirty {
                " (Dirty)"
            } else {
                " (Clean)"
            }
        )
    } else {
        format!(
            " 󰊢 GIT WORKBENCH ─ {} ({}) ",
            git_status.branch, branch_status
        )
    };

    let block = Block::default()
        .title(Span::styled(title, theme.header_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(theme.border_focused_style());

    let inner = block.inner(dlg_area);
    frame.render_widget(block, dlg_area);

    if inner.height < 6 {
        return;
    }

    if !git_status.is_repo {
        let p = Paragraph::new("\n  Current directory is not a Git repository.\n  Run `git init` or navigate into a Git project.")
            .style(Style::default().fg(theme.hidden_color));
        frame.render_widget(p, inner);
        return;
    }

    // Split inner area into: Tab Header (1 row), Main Content (remaining - 1), Footer (1 row)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tab row
            Constraint::Min(4),    // Tab content
            Constraint::Length(1), // Action footer
        ])
        .split(inner);

    // ── 1. Tab Selector Bar ──────────────────────────────────────────────────
    let tabs = ["1: Status", "2: Diff", "3: Branches", "4: History"];
    let mut tab_spans = vec![Span::raw(" ")];
    for (idx, tab_name) in tabs.iter().enumerate() {
        if idx == view.active_tab {
            tab_spans.push(Span::styled(
                format!(" [{}] ", tab_name),
                Style::default()
                    .fg(Color::Black)
                    .bg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            tab_spans.push(Span::styled(
                format!("  {}  ", tab_name),
                Style::default().fg(theme.header_fg),
            ));
        }
        tab_spans.push(Span::raw(" "));
    }
    frame.render_widget(Paragraph::new(Line::from(tab_spans)), chunks[0]);

    // ── 2. Tab Content Dispatch ──────────────────────────────────────────────
    match view.active_tab {
        0 => render_status_tab(frame, chunks[1], git_status, view.selected_idx, theme),
        1 => render_diff_tab(frame, chunks[1], view.diff_content, view.diff_scroll, theme),
        2 => render_branches_tab(frame, chunks[1], view.branches, view.branch_selected, theme),
        _ => render_history_tab(frame, chunks[1], view.history, view.history_selected, theme),
    }

    // ── 3. Footer Action Hints ───────────────────────────────────────────────
    let footer_text = match view.active_tab {
        0 => " [s] Stage  [u] Unstage  [a] Stage All  [A] Unstage All  [c] Commit  [d] Diff  [r] Refresh  [Esc] Close",
        1 => " [▲/▼ or j/k] Scroll Diff  [Tab] Next Tab  [Esc] Close",
        2 => " [b] Switch Branch  [n] New Branch  [d] Delete Branch  [r] Refresh  [Esc] Close",
        _ => " [d/Enter] View Commit Diff  [▲/▼] Navigate  [r] Refresh  [Esc] Close",
    };

    let footer = Paragraph::new(Line::from(Span::styled(
        footer_text,
        Style::default().fg(theme.size_color),
    )));
    frame.render_widget(footer, chunks[2]);
}

/// Render Tab 0: Status & Staging lists.
fn render_status_tab(
    frame: &mut Frame,
    area: Rect,
    git_status: &GitRepoStatus,
    selected_idx: usize,
    theme: &Theme,
) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(area.height / 2),
            Constraint::Min(area.height / 2),
        ])
        .split(area);

    // ── Staged files ─────────────────────────────────────────────────────────
    let staged_items: Vec<ListItem> = if git_status.staged_files.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "  (No changes staged for commit)",
            Style::default().fg(theme.hidden_color),
        )))]
    } else {
        git_status
            .staged_files
            .iter()
            .enumerate()
            .map(|(idx, f)| {
                let is_sel = idx == selected_idx;
                let style = if is_sel {
                    theme.selection_style()
                } else {
                    theme.normal_style()
                };
                let prefix = if is_sel { "▌ " } else { "  " };
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(
                        format!("[{}] ", f.raw_code.trim()),
                        Style::default()
                            .fg(theme.success)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(&f.path, style),
                ]))
            })
            .collect()
    };

    let staged_block = Block::default()
        .title(Span::styled(
            format!(" 󰄬 Staged Changes ({}) ", git_status.staged_files.len()),
            theme.header_style(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_style());

    frame.render_widget(List::new(staged_items).block(staged_block), sections[0]);

    // ── Unstaged & Untracked files ───────────────────────────────────────────
    let mut unstaged_items: Vec<ListItem> = Vec::new();
    let staged_count = git_status.staged_files.len();

    for (idx, f) in git_status.unstaged_files.iter().enumerate() {
        let global_idx = staged_count + idx;
        let is_sel = global_idx == selected_idx;
        let style = if is_sel {
            theme.selection_style()
        } else {
            theme.normal_style()
        };
        let prefix = if is_sel { "▌ " } else { "  " };
        unstaged_items.push(ListItem::new(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(
                "[M] ",
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&f.path, style),
        ])));
    }

    let unstaged_base = staged_count + git_status.unstaged_files.len();
    for (idx, f) in git_status.untracked_files.iter().enumerate() {
        let global_idx = unstaged_base + idx;
        let is_sel = global_idx == selected_idx;
        let style = if is_sel {
            theme.selection_style()
        } else {
            theme.normal_style()
        };
        let prefix = if is_sel { "▌ " } else { "  " };
        unstaged_items.push(ListItem::new(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled("[?] ", Style::default().fg(theme.error)),
            Span::styled(&f.path, style),
        ])));
    }

    for (idx, f) in git_status.conflicted_files.iter().enumerate() {
        let global_idx = unstaged_base + git_status.untracked_files.len() + idx;
        let is_sel = global_idx == selected_idx;
        let style = if is_sel {
            theme.selection_style()
        } else {
            theme.normal_style()
        };
        let prefix = if is_sel { "▌ " } else { "  " };
        unstaged_items.push(ListItem::new(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(
                "[!] ",
                Style::default()
                    .fg(theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&f.path, style),
        ])));
    }

    if unstaged_items.is_empty() {
        unstaged_items.push(ListItem::new(Line::from(Span::styled(
            "  (Working tree clean)",
            Style::default().fg(theme.hidden_color),
        ))));
    }

    let unstaged_total = git_status.unstaged_files.len()
        + git_status.untracked_files.len()
        + git_status.conflicted_files.len();

    let unstaged_block = Block::default()
        .title(Span::styled(
            format!(" 󱉶 Modified & Untracked ({}) ", unstaged_total),
            theme.header_style(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_style());

    frame.render_widget(List::new(unstaged_items).block(unstaged_block), sections[1]);
}

/// Render Tab 1: Unified Diff viewer.
fn render_diff_tab(
    frame: &mut Frame,
    area: Rect,
    diff_content: &[Line<'static>],
    diff_scroll: usize,
    theme: &Theme,
) {
    let block = Block::default()
        .title(Span::styled(" 󰊢 Unified Diff View ", theme.header_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_style());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if diff_content.is_empty() {
        let p = Paragraph::new("\n  No diff to display. Select a modified file and press [d].")
            .style(Style::default().fg(theme.hidden_color));
        frame.render_widget(p, inner);
        return;
    }

    let visible_lines: Vec<Line<'static>> = diff_content
        .iter()
        .skip(diff_scroll)
        .take(inner.height as usize)
        .cloned()
        .collect();

    frame.render_widget(Paragraph::new(visible_lines), inner);
}

/// Render Tab 2: Branches list.
fn render_branches_tab(
    frame: &mut Frame,
    area: Rect,
    branches: &[GitBranch],
    selected_idx: usize,
    theme: &Theme,
) {
    let block = Block::default()
        .title(Span::styled(
            format!(" 󰘬 Repository Branches ({}) ", branches.len()),
            theme.header_style(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_style());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if branches.is_empty() {
        let p = Paragraph::new("\n  No branches found. Press [r] to refresh.")
            .style(Style::default().fg(theme.hidden_color));
        frame.render_widget(p, inner);
        return;
    }

    let items: Vec<ListItem> = branches
        .iter()
        .enumerate()
        .map(|(idx, b)| {
            let is_sel = idx == selected_idx;
            let style = if is_sel {
                theme.selection_style()
            } else {
                theme.normal_style()
            };
            let prefix = if is_sel { "▌ " } else { "  " };

            let mut spans = vec![
                Span::styled(prefix, style),
                if b.is_current {
                    Span::styled(
                        "* ",
                        Style::default()
                            .fg(theme.success)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::raw("  ")
                },
                Span::styled(
                    &b.name,
                    if b.is_current {
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD)
                    } else if b.is_remote {
                        Style::default().fg(theme.size_color)
                    } else {
                        theme.normal_style()
                    },
                ),
            ];

            if let Some(ref up) = b.upstream {
                spans.push(Span::styled(
                    format!(" [{}]", up),
                    Style::default().fg(theme.hidden_color),
                ));
            }
            if b.ahead > 0 || b.behind > 0 {
                spans.push(Span::styled(
                    format!(" [+{}/-{}]", b.ahead, b.behind),
                    Style::default().fg(theme.warning),
                ));
            }
            if !b.commit_summary.is_empty() {
                spans.push(Span::styled(
                    format!(" - {}", b.commit_summary),
                    Style::default().fg(theme.hidden_color),
                ));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    frame.render_widget(List::new(items), inner);
}

/// Render Tab 3: Commit History list.
fn render_history_tab(
    frame: &mut Frame,
    area: Rect,
    history: &[GitCommitInfo],
    selected_idx: usize,
    theme: &Theme,
) {
    let block = Block::default()
        .title(Span::styled(
            format!(" 󰜘 Commit History ({}) ", history.len()),
            theme.header_style(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_style());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if history.is_empty() {
        let p = Paragraph::new("\n  No commit history found.")
            .style(Style::default().fg(theme.hidden_color));
        frame.render_widget(p, inner);
        return;
    }

    let items: Vec<ListItem> = history
        .iter()
        .enumerate()
        .map(|(idx, c)| {
            let is_sel = idx == selected_idx;
            let style = if is_sel {
                theme.selection_style()
            } else {
                theme.normal_style()
            };
            let prefix = if is_sel { "▌ " } else { "  " };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(
                    format!("[{}] ", c.short_hash),
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} ", c.date),
                    Style::default().fg(theme.size_color),
                ),
                Span::styled(
                    format!("{} ", c.author),
                    Style::default().fg(theme.header_fg),
                ),
                Span::styled(&c.message, Style::default().fg(theme.fg)),
            ]))
        })
        .collect();

    frame.render_widget(List::new(items), inner);
}
