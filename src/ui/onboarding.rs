//! First-launch interactive onboarding setup wizard and welcome experience.

use crate::shell::TerminalCapabilities;
use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnboardingStep {
    Welcome,
    Appearance,
    KeyboardInteraction,
    TerminalEnvironment,
    Complete,
}

impl OnboardingStep {
    pub const ALL: &'static [OnboardingStep] = &[
        OnboardingStep::Welcome,
        OnboardingStep::Appearance,
        OnboardingStep::KeyboardInteraction,
        OnboardingStep::TerminalEnvironment,
        OnboardingStep::Complete,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            Self::Welcome => "Welcome to TermExplorer",
            Self::Appearance => "Step 1: Color Theme & Appearance",
            Self::KeyboardInteraction => "Step 2: Keyboard Interaction & Keymaps",
            Self::TerminalEnvironment => "Step 3: Terminal & Linux Environment",
            Self::Complete => "Step 4: Setup Complete",
        }
    }
}

pub fn render_onboarding(
    frame: &mut Frame,
    area: Rect,
    step: OnboardingStep,
    selected_option: usize,
    theme: &Theme,
) {
    let width = (area.width * 8 / 10).clamp(70, 100);
    let height = 24u16.min(area.height.saturating_sub(2));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;

    let dlg_area = Rect {
        x,
        y,
        width,
        height,
    };

    frame.render_widget(Clear, dlg_area);

    let title = format!(" 󰒓 TERMEXPLORER  ─  {} ", step.title());

    let outer = Block::default()
        .title(Span::styled(title, theme.header_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(theme.border_focused_style());

    let inner = outer.inner(dlg_area);
    frame.render_widget(outer, dlg_area);

    if inner.height < 8 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),    // Main content
            Constraint::Length(3), // Footer actions
        ])
        .split(inner);

    let content_area = chunks[0];
    let footer_area = chunks[1];

    match step {
        OnboardingStep::Welcome => {
            let actions = [
                (
                    "1. Explore Current Directory",
                    "Launch immediately into the active project workspace",
                ),
                (
                    "2. Guided Setup Wizard",
                    "Customize theme, layout, and audit terminal capabilities (2 min)",
                ),
                (
                    "3. View Keyboard Shortcuts",
                    "Browse complete modal keyboard shortcut reference (F1)",
                ),
                (
                    "4. Open Settings Studio",
                    "Full-screen settings center with 14 categories (F10)",
                ),
                (
                    "5. Skip Onboarding",
                    "Accept sensible defaults and enter TermExplorer",
                ),
            ];

            let mut lines = vec![
                Line::from(Span::styled(
                    "Welcome to TermExplorer — Inspection-First Terminal Developer Workstation",
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                )),
                Line::raw(""),
                Line::from("TermExplorer combines a 25% hierarchical explorer, 75% universal previewer,"),
                Line::from("embedded PTY shell drawer, Git branch workbench, and secure WebAssembly extensions."),
                Line::raw(""),
                Line::from(Span::styled("Choose an action to begin (use ↑/↓ or number keys 1-5):", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))),
                Line::raw(""),
            ];

            for (idx, (label, desc)) in actions.iter().enumerate() {
                let is_sel = idx == selected_option;
                let style = if is_sel {
                    theme.selection_style()
                } else {
                    theme.normal_style()
                };
                let prefix = if is_sel { "▶ " } else { "  " };
                lines.push(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(format!("{:<32} ", label), style),
                    Span::styled(*desc, Style::default().fg(Color::DarkGray)),
                ]));
            }

            frame.render_widget(Paragraph::new(lines), content_area);
        }

        OnboardingStep::Appearance => {
            let themes = Theme::list_themes();
            let mut lines = vec![
                Line::from("Select your preferred visual theme (updates live across all panes):"),
                Line::raw(""),
            ];

            for (idx, &(name, desc)) in themes.iter().take(8).enumerate() {
                let is_sel = idx == selected_option;
                let style = if is_sel {
                    theme.selection_style()
                } else {
                    theme.normal_style()
                };
                let prefix = if is_sel { "▶ " } else { "  " };
                lines.push(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(format!("{:<24} ", name), style),
                    Span::styled(desc, Style::default().fg(Color::DarkGray)),
                ]));
            }

            frame.render_widget(Paragraph::new(lines), content_area);
        }

        OnboardingStep::KeyboardInteraction => {
            let lines = vec![
                Line::from(Span::styled(
                    "Central Keyboard Shortcuts & Interaction Architecture",
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                )),
                Line::raw(""),
                Line::from(vec![
                    Span::styled("  • Navigation:    ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::styled("j, k (Down/Up) │ h, l (Parent/Enter) │ gg, G (Top/Bottom)", Style::default().fg(Color::Gray)),
                ]),
                Line::from(vec![
                    Span::styled("  • Panes & Focus: ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::styled("Tab (Cycle Explorer/Preview) │ F7 (Dual Pane Split)", Style::default().fg(Color::Gray)),
                ]),
                Line::from(vec![
                    Span::styled("  • PTY Terminal:  ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::styled("` (Backtick) or Ctrl-` (Toggle 15% height drawer)", Style::default().fg(Color::Gray)),
                ]),
                Line::from(vec![
                    Span::styled("  • Developer:     ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::styled("Ctrl+Shift+P (Command Palette) │ g (Git Panel) │ F2 (Editor)", Style::default().fg(Color::Gray)),
                ]),
                Line::from(vec![
                    Span::styled("  • Workstation:   ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::styled("F10 (Settings Studio) │ ? or F1 (Help) │ q (Clean Exit)", Style::default().fg(Color::Gray)),
                ]),
                Line::raw(""),
                Line::from("All shortcuts can be remapped or customized at any time in Settings Studio (F10)."),
            ];

            frame.render_widget(Paragraph::new(lines), content_area);
        }

        OnboardingStep::TerminalEnvironment => {
            let caps = TerminalCapabilities::detect();
            let mut lines = vec![
                Line::from(Span::styled(
                    "Detected Host Terminal Environment & Linux Toolchain",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::raw(""),
                Line::from(vec![
                    Span::styled(
                        "  Terminal:     ",
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(
                            "$TERM={} │ $COLORTERM={} │ TrueColor={}",
                            caps.term,
                            caps.colorterm,
                            if caps.truecolor_supported {
                                "Active (24-bit RGB)"
                            } else {
                                "ANSI Fallback"
                            }
                        ),
                        Style::default().fg(Color::Cyan),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        "  PTY Subsystem:",
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        if caps.pty_available {
                            " Available (/dev/ptmx, Unix pseudo-terminals)"
                        } else {
                            " Unavailable"
                        },
                        Style::default().fg(if caps.pty_available {
                            theme.success
                        } else {
                            Color::Red
                        }),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        "  Default Shell:",
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" {}", caps.default_shell.path.display()),
                        Style::default().fg(Color::Yellow),
                    ),
                ]),
                Line::raw(""),
                Line::from(Span::styled(
                    "  Optional Media & Developer Integrations:",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )),
            ];

            for tool in caps.detected_integrations.iter().take(5) {
                let status_span = if tool.is_available {
                    Span::styled(
                        "   [✓ Found] ",
                        Style::default()
                            .fg(theme.success)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::styled("   [✗ Missing] ", Style::default().fg(Color::DarkGray))
                };
                lines.push(Line::from(vec![
                    status_span,
                    Span::styled(
                        format!("{:<15} ", tool.name),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(&tool.purpose, Style::default().fg(Color::Gray)),
                ]));
            }

            frame.render_widget(Paragraph::new(lines), content_area);
        }

        OnboardingStep::Complete => {
            let text = vec![
                Line::from(Span::styled(
                    "Setup Complete! Your preferences are persisted to ~/.config/termexplorer/config.toml",
                    Style::default().fg(theme.success).add_modifier(Modifier::BOLD),
                )),
                Line::raw(""),
                Line::from("TermExplorer is ready for high-performance terminal development."),
                Line::raw(""),
                Line::from("Quick Navigation Tips:"),
                Line::from("  • Press [F10] to open Settings Studio and adjust any setting"),
                Line::from("  • Press [Ctrl+Shift+P] to open the Command Palette"),
                Line::from("  • Press [ ` ] (Backtick) to launch the embedded PTY shell drawer"),
                Line::from("  • Press [F1] or [?] to view the keyboard cheat-sheet overlay"),
                Line::raw(""),
                Line::from(Span::styled(
                    "Press [Enter] or [Space] to enter the workstation!",
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                )),
            ];
            frame.render_widget(Paragraph::new(text), content_area);
        }
    }

    // Footer actions
    let footer_text = match step {
        OnboardingStep::Welcome => Line::from(vec![
            Span::styled(
                " [1-5] Choose Action   ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" [Enter] Select   ", Style::default().fg(Color::White)),
            Span::styled(" [Esc] Skip Setup   ", Style::default().fg(Color::DarkGray)),
        ]),
        OnboardingStep::Complete => Line::from(vec![Span::styled(
            " [Enter / Space] Launch Workstation ",
            Style::default()
                .fg(theme.success)
                .add_modifier(Modifier::BOLD),
        )]),
        _ => Line::from(vec![
            Span::styled(
                " [Enter] Next Step   ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" [↑/↓] Change Option   ", Style::default().fg(Color::White)),
            Span::styled(
                " [Esc] Skip Wizard   ",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    };

    frame.render_widget(Paragraph::new(footer_text), footer_area);
}
