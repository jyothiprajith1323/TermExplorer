//! Git Dashboard and Version Control panel — developer workstation Git client.

use crate::git::GitRepoStatus;
use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub fn render_git_panel(
    frame: &mut Frame,
    area: Rect,
    git_status: &GitRepoStatus,
    selected_idx: usize,
    theme: &Theme,
) {
    let width = (area.width * 8 / 10).clamp(65, 104);
    let height = 25u16.min(area.height.saturating_sub(2));
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
    let title = if width >= 80 {
        format!(
            " 󰊢 GIT WORKBENCH  ─  Branch: {} ({})  │  [s] Stage  [u] Unstage  [c] Commit  [Esc] Close ",
            git_status.branch,
            branch_status
        )
    } else {
        format!(" 󰊢 GIT  ─  {} ({}) ", git_status.branch, branch_status)
    };

    let block = Block::default()
        .title(Span::styled(title, theme.header_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(theme.border_focused_style());

    let inner = block.inner(dlg_area);
    frame.render_widget(block, dlg_area);

    if inner.height < 8 {
        return;
    }

    if !git_status.is_repo {
        let p = Paragraph::new("\n  Current directory is not a Git repository.\n  Run `git init` to initialize version control.")
            .style(Style::default().fg(theme.hidden_color));
        frame.render_widget(p, inner);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // Staged files
            Constraint::Length(8), // Unstaged & Untracked
            Constraint::Min(4),    // Recent commits
        ])
        .split(inner);

    // ── 1. Staged files ──────────────────────────────────────────────────────
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

    frame.render_widget(List::new(staged_items).block(staged_block), chunks[0]);

    // ── 2. Unstaged & Untracked files ────────────────────────────────────────
    let mut unstaged_items: Vec<ListItem> = Vec::new();
    for f in &git_status.unstaged_files {
        unstaged_items.push(ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "[M] ",
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(&f.path),
        ])));
    }
    for f in &git_status.untracked_files {
        unstaged_items.push(ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled("[?] ", Style::default().fg(theme.error)),
            Span::raw(&f.path),
        ])));
    }
    if unstaged_items.is_empty() {
        unstaged_items.push(ListItem::new(Line::from(Span::styled(
            "  (Working tree clean)",
            Style::default().fg(theme.hidden_color),
        ))));
    }

    let unstaged_count = git_status.unstaged_files.len() + git_status.untracked_files.len();
    let unstaged_block = Block::default()
        .title(Span::styled(
            format!(" 󱉶 Modified & Untracked ({}) ", unstaged_count),
            theme.header_style(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_style());

    frame.render_widget(List::new(unstaged_items).block(unstaged_block), chunks[1]);

    // ── 3. Recent commits ────────────────────────────────────────────────────
    let commit_items: Vec<ListItem> = if git_status.recent_commits.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "  (No commit history found)",
            Style::default().fg(theme.hidden_color),
        )))]
    } else {
        git_status
            .recent_commits
            .iter()
            .take(chunks[2].height.saturating_sub(2) as usize)
            .map(|c| {
                ListItem::new(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        format!("[{}] ", c.hash),
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(&c.message, Style::default().fg(theme.file_color)),
                ]))
            })
            .collect()
    };

    let commits_block = Block::default()
        .title(Span::styled(" 󰜘 Recent Commits ", theme.header_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_style());

    frame.render_widget(List::new(commit_items).block(commits_block), chunks[2]);
}
