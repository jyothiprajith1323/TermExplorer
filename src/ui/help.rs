//! Keyboard shortcuts reference modal — workstation quick reference matrix.

use crate::theme::Theme;
use crate::ui::hit_test::{AppAction, HitRegistry, WidgetId};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub struct HelpCategory {
    pub id: usize,
    pub key_hint: &'static str,
    pub title: &'static str,
    pub icon: &'static str,
    pub description: &'static str,
}

pub type ShortcutItem = (&'static str, &'static str, &'static str);
pub type SectionGroup = (&'static str, &'static [ShortcutItem]);

struct CategoryDetail {
    title: &'static str,
    icon: &'static str,
    description: &'static str,
    shortcuts: &'static [ShortcutItem],
    pro_tips: &'static [&'static str],
}

pub const HELP_CATEGORIES: &[HelpCategory] = &[
    HelpCategory {
        id: 0,
        key_hint: "1",
        title: "ALL",
        icon: "󰈹",
        description: "Complete keyboard shortcuts matrix across all workstation subsystems",
    },
    HelpCategory {
        id: 1,
        key_hint: "2",
        title: "WORKSTATION",
        icon: "󰕰",
        description: "Layout architecture, panel resizing, dual-pane explorer & telemetry",
    },
    HelpCategory {
        id: 2,
        key_hint: "3",
        title: "MENUS",
        icon: "󰘳",
        description: "Command palette, Settings Studio, Plugin Center & Git Workbench",
    },
    HelpCategory {
        id: 3,
        key_hint: "4",
        title: "NAVIGATION",
        icon: "󰞓",
        description: "Directory tree traversal, cursor control, panel focus & fuzzy filter",
    },
    HelpCategory {
        id: 4,
        key_hint: "5",
        title: "TERMINAL",
        icon: "󰞷",
        description: "Integrated Linux PTY terminal drawer, sizing, maximize & scrolling",
    },
    HelpCategory {
        id: 5,
        key_hint: "6",
        title: "EDITOR & FILES",
        icon: "󰈚",
        description: "Built-in code buffer, autocomplete, undo/redo & file system operations",
    },
];

const SHORTCUTS_WORKSTATION: &[(&str, &str, &str)] = &[
    (
        "F7",
        "Dual-Pane Explorer",
        "Toggle side-by-side secondary file manager",
    ),
    (
        "F6 / Ctrl+M",
        "Telemetry Monitor",
        "Toggle live CPU, RAM, Swap, Disk & Net monitor",
    ),
    (
        "F11 / z",
        "Fullscreen Preview",
        "Expand preview panel to 100% width",
    ),
    (
        "Ctrl+Right / Alt+]",
        "Expand Preview",
        "Increase preview panel width (+5%, up to 85%)",
    ),
    (
        "Ctrl+Left / Alt+[",
        "Shrink Preview",
        "Decrease preview panel width (-5%, down to 15%)",
    ),
    (
        "Ctrl+B",
        "Toggle Sidebar",
        "Show or hide workspace folders sidebar",
    ),
    (
        "F12",
        "System Diagnostics",
        "Toggle frame latency and hit-testing overlay",
    ),
];

const SHORTCUTS_MENUS: &[(&str, &str, &str)] = &[
    (
        "Ctrl+P / Ctrl+Shift+P",
        "Command Palette",
        "Fuzzy search actions & quick folder navigation",
    ),
    (
        "Ctrl+, / F10",
        "Settings Studio",
        "Interactive settings with 14 customizable categories",
    ),
    (
        "Ctrl+Shift+X",
        "Plugin Marketplace",
        "Discover, install and configure Wasm extensions",
    ),
    (
        "g / Ctrl+G",
        "Git Workbench",
        "Inspect repository status, staged diffs and commit log",
    ),
    (
        "F1 / ?",
        "Shortcuts Matrix",
        "Toggle this workstation keyboard reference modal",
    ),
    (
        "Ctrl+Q",
        "Quit TermExplorer",
        "Cleanly terminate all sessions and exit to shell",
    ),
];

const SHORTCUTS_NAVIGATION: &[(&str, &str, &str)] = &[
    (
        "↑ / ↓ (k / j)",
        "Cursor Move",
        "Navigate up and down in file list or tables",
    ),
    (
        "→ / Enter (l)",
        "Open / Descend",
        "Open folder into tree or inspect file in preview",
    ),
    (
        "← / Backspace (h)",
        "Ascend Directory",
        "Navigate upward to parent directory",
    ),
    (
        "Space",
        "Focus Preview",
        "Jump active focus directly into preview/inspector",
    ),
    (
        "Tab / Shift+Tab",
        "Cycle Focus / Tabs",
        "Switch focus between panels or cycle preview tabs",
    ),
    (
        "Home / End",
        "First / Last Entry",
        "Instantly jump to top or bottom of active directory",
    ),
    (
        "PageUp / PageDn",
        "Scroll Page",
        "Quickly scroll active list by one screen",
    ),
    (
        "/",
        "Inline Fuzzy Filter",
        "Instantly filter files in current directory by name",
    ),
];

const SHORTCUTS_TERMINAL: &[(&str, &str, &str)] = &[
    (
        "Ctrl+T / Ctrl+`",
        "Toggle Terminal",
        "Show or hide integrated bottom Linux PTY drawer",
    ),
    (
        "Alt+F6 / Ctrl+Shift+T",
        "Maximize Terminal",
        "Expand terminal drawer to ~80% workstation height",
    ),
    (
        "Ctrl+Up / Alt+}",
        "Increase Height",
        "Grow terminal height by 2 rows",
    ),
    (
        "Ctrl+Down / Alt+{",
        "Decrease Height",
        "Shrink terminal height by 2 rows",
    ),
    (
        "Alt+Up / Alt+k",
        "Focus to Explorer",
        "Return focus from terminal back to file explorer",
    ),
    (
        "Click Terminal",
        "Focus PTY Shell",
        "Click inside terminal area for interactive typing",
    ),
];

const SHORTCUTS_EDITOR: &[(&str, &str, &str)] = &[
    (
        "F2 / Enter",
        "Open Editor",
        "Open highlighted file in built-in code editor",
    ),
    (
        "Ctrl+S",
        "Save Buffer",
        "Write active editor buffer changes safely to disk",
    ),
    (
        "Tab",
        "Accept Completion",
        "Accept instant word or path autocomplete popup",
    ),
    (
        "Ctrl+Z / Ctrl+Y",
        "Undo / Redo",
        "Revert or restore text buffer editing operations",
    ),
    (
        "Ctrl+C / Ctrl+X / Ctrl+V",
        "Clipboard Line",
        "Copy, cut, or paste entire lines in buffer",
    ),
    (
        "Esc",
        "Close Editor",
        "Exit editor buffer and return to file explorer",
    ),
];

const SHORTCUTS_FILEOPS: &[(&str, &str, &str)] = &[
    (
        "a / F4",
        "New File",
        "Create empty file with immediate name prompt",
    ),
    (
        "A / F7",
        "New Directory",
        "Create new subdirectory in current folder",
    ),
    ("r", "Rename", "Rename selected file or folder in place"),
    (
        "d / F8 / Del",
        "Delete Entry",
        "Delete selected entry (with trash / confirmation)",
    ),
    (
        "c / y",
        "Copy Entry",
        "Copy selected file or folder to clipboard",
    ),
    ("x", "Cut Entry", "Cut selected file or folder for moving"),
    (
        "v / p",
        "Paste Entry",
        "Paste clipboard file or folder into current directory",
    ),
    (
        ".",
        "Toggle Hidden",
        "Show or hide dotfiles (.git, .config, etc.)",
    ),
    (
        "s",
        "Cycle Sort",
        "Sort by Name, Size, Date Modified, or Extension",
    ),
    (
        "F5",
        "Refresh",
        "Re-read directory tree and refresh Git repository state",
    ),
];

pub fn render_help(
    frame: &mut Frame,
    area: Rect,
    active_category: usize,
    scroll: usize,
    theme: &Theme,
    hit_registry: &mut HitRegistry,
) {
    let popup_w = area.width.clamp(76, 114).min(area.width.saturating_sub(2));
    let popup_h = area.height.clamp(24, 38).min(area.height.saturating_sub(2));
    let popup_x = area.x + (area.width.saturating_sub(popup_w)) / 2;
    let popup_y = area.y + (area.height.saturating_sub(popup_h)) / 2;

    let popup = Rect {
        x: popup_x,
        y: popup_y,
        width: popup_w,
        height: popup_h,
    };

    frame.render_widget(Clear, popup);

    // Register close hit region at top right
    let close_btn_w = 14u16;
    let close_btn_rect = Rect {
        x: popup.right().saturating_sub(close_btn_w + 2),
        y: popup.y,
        width: close_btn_w,
        height: 1,
    };
    hit_registry.register_button(
        WidgetId::HelpClose,
        close_btn_rect,
        AppAction::CloseActiveView,
        12,
    );

    let title_line = Line::from(vec![
        Span::styled(
            " 󰋖 KEYBOARD SHORTCUTS MATRIX ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("│ ", Style::default().fg(Color::Rgb(88, 91, 112))),
        Span::styled(
            "The Developer Workstation ",
            Style::default().fg(Color::Rgb(166, 173, 200)),
        ),
        Span::styled(
            "[✕ Esc / F1] ",
            Style::default()
                .fg(Color::Rgb(243, 139, 168))
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let block = Block::default()
        .title(title_line)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent));

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    if inner.width < 50 || inner.height < 10 {
        return;
    }

    // Vertical layout:
    // 0: Category tabs bar (height 2)
    // 1: Content body (Min 6)
    // 2: Bottom status / hint bar (height 1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(6),
            Constraint::Length(1),
        ])
        .split(inner);

    let tabs_area = chunks[0];
    let body_area = chunks[1];
    let footer_area = chunks[2];

    // ── Render Category Tabs ──────────────────────────────────────────────────
    render_category_tabs(frame, tabs_area, active_category, theme, hit_registry);

    // ── Render Content Body ───────────────────────────────────────────────────
    if active_category == 0 {
        render_overview_matrix(frame, body_area, scroll, theme);
    } else {
        render_detailed_category(frame, body_area, active_category, scroll, theme);
    }

    // ── Render Footer ─────────────────────────────────────────────────────────
    render_footer(frame, footer_area, theme);
}

fn render_category_tabs(
    frame: &mut Frame,
    area: Rect,
    active_category: usize,
    theme: &Theme,
    hit_registry: &mut HitRegistry,
) {
    let mut tab_spans = Vec::new();
    let mut cur_col = area.x;

    for (idx, cat) in HELP_CATEGORIES.iter().enumerate() {
        let is_active = idx == active_category;
        let label = format!(" {} [{}] {} ", cat.icon, cat.key_hint, cat.title);
        let tab_w = label.chars().count() as u16;

        let style = if is_active {
            Style::default()
                .bg(theme.accent)
                .fg(Color::Rgb(17, 17, 27))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .bg(Color::Rgb(30, 32, 48))
                .fg(Color::Rgb(186, 194, 222))
        };

        if cur_col + tab_w <= area.right() {
            let tab_rect = Rect {
                x: cur_col,
                y: area.y,
                width: tab_w,
                height: 1,
            };
            hit_registry.register_button(
                WidgetId::HelpTab(idx),
                tab_rect,
                AppAction::SelectHelpCategory(idx),
                11,
            );
        }

        tab_spans.push(Span::styled(label, style));
        tab_spans.push(Span::raw(" "));
        cur_col = cur_col.saturating_add(tab_w + 1);
    }

    // Divider line under tabs
    let divider = Line::from(vec![Span::styled(
        "─".repeat(area.width as usize),
        Style::default().fg(Color::Rgb(49, 50, 68)),
    )]);

    let tabs_paragraph = Paragraph::new(vec![Line::from(tab_spans), divider]);
    frame.render_widget(tabs_paragraph, area);
}

fn render_overview_matrix(frame: &mut Frame, area: Rect, scroll: usize, theme: &Theme) {
    if area.width >= 82 {
        // Two-column layout
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let left_lines = build_column_lines(
            &[
                ("󰕰 WORKSTATION & WINDOWS", SHORTCUTS_WORKSTATION),
                ("󰘳 MENUS & CENTERS", SHORTCUTS_MENUS),
            ],
            scroll,
            cols[0].height as usize,
            theme,
        );

        let right_lines = build_column_lines(
            &[
                ("󰞓 NAVIGATION & CURSOR", SHORTCUTS_NAVIGATION),
                ("󰞷 EMBEDDED TERMINAL", SHORTCUTS_TERMINAL),
                ("󰈚 EDITOR & FILE OPS", SHORTCUTS_EDITOR),
            ],
            scroll,
            cols[1].height as usize,
            theme,
        );

        frame.render_widget(
            Paragraph::new(left_lines).wrap(Wrap { trim: false }),
            cols[0],
        );
        frame.render_widget(
            Paragraph::new(right_lines).wrap(Wrap { trim: false }),
            cols[1],
        );
    } else {
        // Single column layout
        let all_lines = build_column_lines(
            &[
                ("󰕰 WORKSTATION & WINDOWS", SHORTCUTS_WORKSTATION),
                ("󰘳 MENUS & CENTERS", SHORTCUTS_MENUS),
                ("󰞓 NAVIGATION & CURSOR", SHORTCUTS_NAVIGATION),
                ("󰞷 EMBEDDED TERMINAL", SHORTCUTS_TERMINAL),
                ("󰈚 EDITOR", SHORTCUTS_EDITOR),
                ("󰈔 FILE OPERATIONS", SHORTCUTS_FILEOPS),
            ],
            scroll,
            area.height as usize,
            theme,
        );
        frame.render_widget(Paragraph::new(all_lines).wrap(Wrap { trim: false }), area);
    }
}

fn build_column_lines(
    sections: &[SectionGroup],
    scroll: usize,
    _max_height: usize,
    theme: &Theme,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    for (sec_title, bindings) in sections {
        // Section Header with styled banner
        lines.push(Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled(
                format!(" {} ", sec_title),
                Style::default()
                    .fg(theme.accent)
                    .bg(Color::Rgb(30, 32, 48))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " ───────────────────────────────────",
                Style::default().fg(Color::Rgb(49, 50, 68)),
            ),
        ]));

        for (key, title, _desc) in *bindings {
            lines.push(Line::from(vec![
                Span::raw("  "),
                // Pill badge for the shortcut
                Span::styled(
                    format!(" {:<18} ", key),
                    Style::default()
                        .fg(Color::Rgb(249, 226, 175)) // Catppuccin Yellow / Gold
                        .bg(Color::Rgb(40, 44, 60))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    title.to_string(),
                    Style::default()
                        .fg(Color::Rgb(205, 214, 244))
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }
        lines.push(Line::raw(""));
    }

    if scroll > 0 && scroll < lines.len() {
        lines.drain(0..scroll);
    }

    lines
}

fn render_detailed_category(
    frame: &mut Frame,
    area: Rect,
    category_idx: usize,
    scroll: usize,
    theme: &Theme,
) {
    let detail = match category_idx {
        1 => CategoryDetail {
            title: "WORKSTATION & WINDOW MANAGEMENT",
            icon: "󰕰",
            description: "Standardized 25% File Explorer / 75% Contextual Preview with live resizing and dual-pane",
            shortcuts: SHORTCUTS_WORKSTATION,
            pro_tips: &[
                "Use `[` and `]` to instantly expand or shrink the explorer width on the fly.",
                "Press `F7` to toggle dual-pane explorer for side-by-side file copy/move operations.",
                "Press `F11` or `z` to maximize the preview pane to 100% fullscreen for image inspection.",
                "Press `F6` to inspect system CPU, RAM, swap, disk I/O, and network activity.",
                "Press `F12` to open the real-time Diagnostics overlay with frame latency metrics.",
            ],
        },
        2 => CategoryDetail {
            title: "MENUS, CENTERS & WORKBENCHES",
            icon: "󰘳",
            description: "Central command surfaces: Command Palette, Settings Studio, Plugin Manager & Git",
            shortcuts: SHORTCUTS_MENUS,
            pro_tips: &[
                "Press `Ctrl+P` to open the Command Palette with acronym search (e.g., 'git', 'st').",
                "Press `Ctrl+,` or `F10` to enter the Interactive Settings Studio with 14 categories.",
                "Press `Ctrl+Shift+X` to browse, install, and configure sandboxed Wasm extensions.",
                "Press `g` or `Ctrl+G` to enter the full Git Workbench with commit history and staging.",
                "All menus are keyboard navigable with `Tab`, `Enter`, and `Esc` to return.",
            ],
        },
        3 => CategoryDetail {
            title: "NAVIGATION, CURSOR & SEARCH",
            icon: "󰞓",
            description: "Blazing fast directory navigation with Vim bindings, fuzzy filtering and panel cycling",
            shortcuts: SHORTCUTS_NAVIGATION,
            pro_tips: &[
                "Vim keys `h`, `j`, `k`, `l` work seamlessly alongside standard arrow keys.",
                "Press `/` anytime in the explorer for instant inline fuzzy filtering of file entries.",
                "Press `Tab` / `Shift+Tab` to quickly switch focus between Explorer, Preview and Terminal.",
                "Press `Home` / `End` to jump directly to the top or bottom of massive directories.",
                "Press `Backspace` to swiftly navigate up to the parent directory hierarchy.",
            ],
        },
        4 => CategoryDetail {
            title: "EMBEDDED LINUX PTY TERMINAL",
            icon: "󰞷",
            description: "Compact integrated terminal drawer running your native shell with full PTY emulation",
            shortcuts: SHORTCUTS_TERMINAL,
            pro_tips: &[
                "The embedded terminal starts open by default (~20-22% height) for instant developer flow.",
                "Press `Ctrl+T` to toggle the terminal drawer open or closed anytime.",
                "Press `Alt+F6` or `Ctrl+Shift+T` to maximize the terminal to ~80% of workstation height.",
                "Press `}` / `Ctrl+Up` to grow terminal height and `{` / `Ctrl+Down` to shrink it.",
                "Press `Alt+Up` or `Alt+k` to return keyboard focus back to the file explorer.",
            ],
        },
        _ => CategoryDetail {
            title: "CODE EDITOR & FILE OPERATIONS",
            icon: "󰈚",
            description: "Built-in code editor buffer, sub-millisecond autocomplete, and file system management",
            shortcuts: SHORTCUTS_EDITOR,
            pro_tips: &[
                "Press `Enter` or `F2` on any file to open it in the lightweight built-in editor.",
                "Word & path autocomplete triggers automatically with `< 0.2ms` measured latency.",
                "Press `Tab` to accept autocomplete suggestions; press `Esc` to dismiss.",
                "Press `a` / `F4` to create a new file; press `A` to create a new directory.",
                "Press `d` / `F8` to delete; press `y` to copy; press `p` to paste.",
            ],
        },
    };

    let mut lines = Vec::new();

    // Category Hero Banner
    lines.push(Line::from(vec![
        Span::styled(
            format!(" {} ", detail.icon),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            detail.title,
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![Span::styled(
        format!("  {}", detail.description),
        Style::default().fg(Color::Rgb(147, 153, 178)),
    )]));
    lines.push(Line::raw(""));

    // Shortcuts Section
    lines.push(Line::from(vec![Span::styled(
        "  KEYBOARD SHORTCUTS",
        Style::default()
            .fg(Color::Rgb(186, 194, 222))
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(vec![Span::styled(
        "  ─────────────────────────────────────────────────────────────────",
        Style::default().fg(Color::Rgb(49, 50, 68)),
    )]));

    for (key, title, desc) in detail.shortcuts {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!(" {:<20} ", key),
                Style::default()
                    .fg(Color::Rgb(249, 226, 175))
                    .bg(Color::Rgb(40, 44, 60))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                format!("{:<24}", title),
                Style::default()
                    .fg(Color::Rgb(205, 214, 244))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("─ {}", desc),
                Style::default().fg(Color::Rgb(147, 153, 178)),
            ),
        ]));
    }

    if category_idx == 5 {
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![Span::styled(
            "  FILE OPERATIONS & GIT SHORTCUTS",
            Style::default()
                .fg(Color::Rgb(186, 194, 222))
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "  ─────────────────────────────────────────────────────────────────",
            Style::default().fg(Color::Rgb(49, 50, 68)),
        )]));

        for (key, title, desc) in SHORTCUTS_FILEOPS {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!(" {:<20} ", key),
                    Style::default()
                        .fg(Color::Rgb(166, 227, 161)) // Green pill
                        .bg(Color::Rgb(40, 44, 60))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("{:<24}", title),
                    Style::default()
                        .fg(Color::Rgb(205, 214, 244))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("─ {}", desc),
                    Style::default().fg(Color::Rgb(147, 153, 178)),
                ),
            ]));
        }
    }

    // Pro Tips Section
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![Span::styled(
        "  💡 WORKSTATION PRO TIPS",
        Style::default()
            .fg(Color::Rgb(250, 179, 135))
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(vec![Span::styled(
        "  ─────────────────────────────────────────────────────────────────",
        Style::default().fg(Color::Rgb(49, 50, 68)),
    )]));

    for tip in detail.pro_tips {
        lines.push(Line::from(vec![
            Span::styled("  • ", Style::default().fg(theme.accent)),
            Span::styled(*tip, Style::default().fg(Color::Rgb(186, 194, 222))),
        ]));
    }

    if scroll > 0 && scroll < lines.len() {
        lines.drain(0..scroll);
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
}

fn render_footer(frame: &mut Frame, area: Rect, theme: &Theme) {
    let footer_line = Line::from(vec![
        Span::styled(
            " [1-6 / Tab] ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Switch Category  ",
            Style::default().fg(Color::Rgb(166, 173, 200)),
        ),
        Span::styled(
            "│  [↑/↓] ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Scroll  ", Style::default().fg(Color::Rgb(166, 173, 200))),
        Span::styled(
            "│  [Esc / F1 / q] ",
            Style::default()
                .fg(Color::Rgb(243, 139, 168))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Dismiss  ", Style::default().fg(Color::Rgb(166, 173, 200))),
        Span::styled(
            "│  TermExplorer v1.1.0",
            Style::default().fg(Color::Rgb(108, 112, 134)),
        ),
    ]);

    frame.render_widget(Paragraph::new(footer_line), area);
}
