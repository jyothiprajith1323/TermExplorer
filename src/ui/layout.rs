//! Layout — computes the rects for every panel.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct LayoutAreas {
    pub header: Rect,
    pub sidebar: Option<Rect>,
    pub explorer: Rect,
    pub preview: Option<Rect>,
    pub bottom_panel: Option<Rect>, // terminal OR monitor
    pub status_bar: Rect,
}

pub struct LayoutConfig {
    pub show_sidebar: bool,
    pub show_preview: bool,
    pub show_bottom: bool,
    pub sidebar_pct: u16,
    pub preview_pct: u16,
    pub bottom_height: u16,
}

pub fn compute_layout(area: Rect, cfg: &LayoutConfig) -> LayoutAreas {
    // ── Vertical slices: header / content / status_bar ───────────────────
    let bottom_h = if cfg.show_bottom {
        cfg.bottom_height
    } else {
        0
    };

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),        // header
            Constraint::Min(1),           // content
            Constraint::Length(bottom_h), // terminal/monitor
            Constraint::Length(1),        // status bar
        ])
        .split(area);

    let header = vertical[0];
    let content_area = vertical[1];
    let bottom_panel = if cfg.show_bottom {
        Some(vertical[2])
    } else {
        None
    };
    let status_bar = vertical[3];

    // ── Horizontal slices: sidebar / explorer / preview ──────────────────
    let mut h_constraints = Vec::new();
    if cfg.show_sidebar {
        h_constraints.push(Constraint::Percentage(cfg.sidebar_pct));
    }
    // Explorer gets whatever is left.
    if cfg.show_preview {
        h_constraints.push(Constraint::Min(1));
        h_constraints.push(Constraint::Percentage(cfg.preview_pct));
    } else {
        h_constraints.push(Constraint::Min(1));
    }

    let h_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(h_constraints)
        .split(content_area);

    let mut idx = 0usize;
    let sidebar = if cfg.show_sidebar {
        let r = Some(h_areas[idx]);
        idx += 1;
        r
    } else {
        None
    };
    let explorer = h_areas[idx];
    idx += 1;
    let preview = if cfg.show_preview {
        Some(h_areas[idx])
    } else {
        None
    };

    LayoutAreas {
        header,
        sidebar,
        explorer,
        preview,
        bottom_panel,
        status_bar,
    }
}
