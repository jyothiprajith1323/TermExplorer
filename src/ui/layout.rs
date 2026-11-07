//! Layout — computes the rects for every panel.
//!
//! Default Main Workspace Architecture:
//! - 1-line Top Header (branding, breadcrumb, git, search, telemetry, settings)
//! - Two-Column Content Workspace:
//!   - LEFT: File Explorer (default 70% width)
//!   - RIGHT: Contextual Preview (default 30% width)
//! - Integrated Bottom Terminal (default ~25% height)
//! - 1-line Minimal Status Bar

use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct LayoutAreas {
    pub header: Rect,
    pub sidebar: Option<Rect>,
    pub explorer: Rect,
    pub explorer_secondary: Option<Rect>,
    pub preview: Option<Rect>,
    pub bottom_panel: Option<Rect>, // terminal OR monitor
    pub status_bar: Rect,
}

#[derive(Debug, Clone, Default)]
pub struct LayoutConfig {
    pub show_sidebar: bool,
    pub show_preview: bool,
    pub show_bottom: bool,
    pub dual_pane: bool,
    pub sidebar_pct: u16,
    pub preview_pct: u16,
    pub bottom_height: u16,
    pub preview_maximized: bool,
    pub terminal_maximized: bool,
}

pub fn compute_layout(area: Rect, cfg: &LayoutConfig) -> LayoutAreas {
    // ── Adaptive responsive breakpoints ──────────────────────────────────
    // Sidebar is NOT shown by default. If explicitly enabled by the user, only render on very wide displays.
    let show_sidebar = cfg.show_sidebar && area.width >= 110;
    // On narrow terminals (< 70 columns), collapse preview to preserve explorer readability.
    let show_preview = cfg.show_preview && area.width >= 70;

    // ── Vertical slices: header (1) / content (75%) / terminal (25%) / status_bar (1) ──
    let bottom_h = if cfg.show_bottom && area.height >= 12 {
        if cfg.terminal_maximized {
            // Maximized terminal takes ~80% height leaving 1 row header, 1 row status, and small context
            (area.height * 80 / 100)
                .min(area.height.saturating_sub(4))
                .max(8)
        } else {
            // Default height is ~25% of terminal height (min 5 rows, max height - 6)
            let calc_height = if cfg.bottom_height > 0 {
                cfg.bottom_height
            } else {
                (area.height * 25 / 100).max(6)
            };
            calc_height.min(area.height.saturating_sub(6)).max(5)
        }
    } else {
        0
    };

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),        // header
            Constraint::Min(4),           // content (explorer + preview)
            Constraint::Length(bottom_h), // bottom terminal/monitor (~25%)
            Constraint::Length(1),        // status bar
        ])
        .split(area);

    let header = vertical[0];
    let content_area = vertical[1];
    let bottom_panel = if cfg.show_bottom && bottom_h > 0 {
        Some(vertical[2])
    } else {
        None
    };
    let status_bar = vertical[3];

    // ── Fullscreen Preview Mode (100% width dedicated to preview) ────────
    if cfg.preview_maximized && show_preview {
        return LayoutAreas {
            header,
            sidebar: None,
            explorer: Rect::default(),
            explorer_secondary: None,
            preview: Some(content_area),
            bottom_panel,
            status_bar,
        };
    }

    // ── Horizontal slices: Two-Column Workspace (Explorer 70%, Preview 30%) ──
    let preview_pct = if cfg.preview_pct > 0 {
        cfg.preview_pct.clamp(15, 60)
    } else {
        30
    };

    let mut h_constraints = Vec::new();
    if show_sidebar {
        let sb_pct = if cfg.sidebar_pct > 0 {
            cfg.sidebar_pct.clamp(12, 30)
        } else {
            18
        };
        h_constraints.push(Constraint::Percentage(sb_pct));
    }

    if show_preview {
        let explorer_pct = if show_sidebar {
            100u16.saturating_sub(18 + preview_pct)
        } else {
            100u16.saturating_sub(preview_pct)
        };
        h_constraints.push(Constraint::Percentage(explorer_pct));
        h_constraints.push(Constraint::Percentage(preview_pct));
    } else {
        h_constraints.push(Constraint::Percentage(100));
    }

    let h_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(h_constraints)
        .split(content_area);

    let mut idx = 0usize;
    let sidebar = if show_sidebar {
        let r = Some(h_areas[idx]);
        idx += 1;
        r
    } else {
        None
    };

    let explorer_full = h_areas[idx];
    idx += 1;

    let preview = if show_preview {
        Some(h_areas[idx])
    } else {
        None
    };

    let (explorer, explorer_secondary) = if cfg.dual_pane {
        let dual = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(explorer_full);
        (dual[0], Some(dual[1]))
    } else {
        (explorer_full, None)
    };

    LayoutAreas {
        header,
        sidebar,
        explorer,
        explorer_secondary,
        preview,
        bottom_panel,
        status_bar,
    }
}
