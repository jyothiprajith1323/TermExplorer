//! Keyboard shortcuts reference modal — workstation quick reference matrix.

use crate::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

const HELP_SECTIONS: &[(&str, &[(&str, &str)])] = &[
    (
        "󰞓 NAVIGATION",
        &[
            ("↑ / ↓ (k / j)", "Move up / down in directory or list"),
            ("→ / Enter (l)", "Navigate into directory / view file"),
            ("← / Backspace (h)", "Navigate out to parent folder"),
            ("Tab / Shift+Tab", "Cycle focus between active panels"),
        ],
    ),
    (
        "󰈔 FILE OPERATIONS",
        &[
            ("F4 / a", "Create new empty file"),
            ("F7 / A", "Create new subdirectory"),
            ("F8 / d", "Delete selected item (trash / prompt)"),
            ("r", "Rename selected file or folder"),
            ("y / c", "Copy selected entry to clipboard"),
            ("x", "Cut selected entry to clipboard"),
            ("p / v", "Paste entry into active directory"),
            (".", "Toggle hidden files visibility"),
            ("s", "Cycle sort order (Name, Size, Modified)"),
        ],
    ),
    (
        "󰍉 PANELS & WORKSTATION",
        &[
            ("F1 / ?", "Toggle this keyboard shortcuts reference"),
            ("F2", "Open selected file in built-in code editor"),
            ("F5", "Refresh filesystem directory & Git state"),
            ("F6", "Toggle system resource telemetry monitor"),
            ("F7", "Toggle dual-pane file explorer view"),
            ("F10", "Open full Settings Studio"),
            ("Ctrl+Shift+P", "Open global fuzzy Command Palette"),
            ("g", "Open Git Dashboard & commit log"),
            ("Ctrl+`", "Toggle integrated terminal drawer"),
            ("Ctrl+B", "Toggle left workspace navigation sidebar"),
            ("Alt+P", "Pin current directory to sidebar"),
        ],
    ),
    (
        "󰍉 SEARCH & FILTERING",
        &[
            ("Ctrl+F", "Search in current directory (modal)"),
            ("/", "Instant inline fuzzy filter for file list"),
            ("Esc", "Clear filter / dismiss modal / return focus"),
        ],
    ),
    (
        "󰈚 EDITOR (WHEN ACTIVE)",
        &[
            ("Ctrl+S", "Save active editor buffer to disk"),
            ("Ctrl+Z", "Undo last edit action"),
            ("Ctrl+Y", "Redo previously undone edit action"),
            ("Esc", "Exit editor and return to explorer"),
            ("Ctrl+Q", "Quit TermExplorer"),
        ],
    ),
];

pub fn render_help(frame: &mut Frame, area: Rect, theme: &Theme) {
    let popup_w = area.width.clamp(60, 80);
    let popup_h = area.height.clamp(20, 38).min(area.height);
    let popup = Rect {
        x: area.x + (area.width.saturating_sub(popup_w)) / 2,
        y: area.y + (area.height.saturating_sub(popup_h)) / 2,
        width: popup_w,
        height: popup_h,
    };

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(Span::styled(
            " 󰋖 KEYBOARD SHORTCUTS MATRIX  ─  [Esc / F1] Dismiss ",
            theme.header_style(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(theme.border_focused_style());

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let mut lines = Vec::new();
    for (sec_title, bindings) in HELP_SECTIONS {
        lines.push(Line::from(vec![Span::styled(
            format!("  {} ", sec_title),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )]));
        for (key, desc) in *bindings {
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(
                    format!("{:<20}", key),
                    Style::default()
                        .fg(Color::Rgb(255, 190, 70))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(*desc, Style::default().fg(theme.file_color)),
            ]));
        }
        lines.push(Line::raw(""));
    }

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(para, inner);
}
