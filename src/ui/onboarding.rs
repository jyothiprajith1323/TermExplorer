//! First-launch interactive onboarding setup wizard.

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
    ThemeSelection,
    LayoutSelection,
    IntegrationsScan,
    Complete,
}

impl OnboardingStep {
    pub const ALL: &'static [OnboardingStep] = &[
        OnboardingStep::Welcome,
        OnboardingStep::ThemeSelection,
        OnboardingStep::LayoutSelection,
        OnboardingStep::IntegrationsScan,
        OnboardingStep::Complete,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            Self::Welcome => "Welcome to TermExplorer",
            Self::ThemeSelection => "Step 1: Choose Your Color Theme",
            Self::LayoutSelection => "Step 2: Choose Workspace Layout",
            Self::IntegrationsScan => "Step 3: Linux Toolchain & Integrations",
            Self::Complete => "Setup Complete!",
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
    let width = (area.width * 7 / 10).clamp(60, 95);
    let height = 22u16.min(area.height.saturating_sub(2));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;

    let dlg_area = Rect {
        x,
        y,
        width,
        height,
    };

    frame.render_widget(Clear, dlg_area);

    let title = format!(" 󰒓 TERMEXPLORER ONBOARDING  ─  {} ", step.title());

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
            Constraint::Min(6),    // Main content
            Constraint::Length(3), // Footer buttons
        ])
        .split(inner);

    let content_area = chunks[0];
    let footer_area = chunks[1];

    match step {
        OnboardingStep::Welcome => {
            let text = vec![
                Line::from(Span::styled(
                    "Welcome to TermExplorer — Your terminal. Your workspace. Your rules.",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::raw(""),
                Line::from(
                    "TermExplorer combines a modern file manager, syntax-aware code editor,",
                ),
                Line::from(
                    "real pseudo-terminal emulator, system monitor, and universal file viewer",
                ),
                Line::from(
                    "into a single native keyboard-driven developer environment on Ubuntu Linux.",
                ),
                Line::raw(""),
                Line::from("This quick setup wizard will help configure your preferences."),
                Line::from("You can change any setting at any time later by pressing [F10]."),
            ];
            frame.render_widget(Paragraph::new(text), content_area);
        }
        OnboardingStep::ThemeSelection => {
            let themes = Theme::list_themes();
            let mut lines = vec![
                Line::from("Select your preferred color theme (press ↑/↓ to choose):"),
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
                    Span::styled(format!("{:<22} ", name), style),
                    Span::styled(desc, Style::default().fg(Color::DarkGray)),
                ]));
            }
            frame.render_widget(Paragraph::new(lines), content_area);
        }
        OnboardingStep::LayoutSelection => {
            let profiles = crate::settings::ProfileKind::ALL;
            let mut lines = vec![
                Line::from("Choose your default layout arrangement:"),
                Line::raw(""),
            ];

            for (idx, p) in profiles.iter().enumerate() {
                let is_sel = idx == selected_option;
                let style = if is_sel {
                    theme.selection_style()
                } else {
                    theme.normal_style()
                };
                let prefix = if is_sel { "▶ " } else { "  " };
                lines.push(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(format!("{:<30} ", p.name()), style),
                    Span::styled(p.description(), Style::default().fg(Color::DarkGray)),
                ]));
            }
            frame.render_widget(Paragraph::new(lines), content_area);
        }
        OnboardingStep::IntegrationsScan => {
            let tools = crate::preview::CapabilityRegistry::list_integrations();
            let mut lines = vec![
                Line::from("Detected Linux development utilities in your $PATH:"),
                Line::raw(""),
            ];

            for t in tools.iter().take(8) {
                let status_span = if t.installed {
                    Span::styled(
                        " [✓ Detected] ",
                        Style::default()
                            .fg(theme.success)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::styled(" [✗ Not Found]", Style::default().fg(Color::DarkGray))
                };
                lines.push(Line::from(vec![
                    status_span,
                    Span::styled(
                        format!(" {:<14}", t.name),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(t.purpose, Style::default().fg(Color::Gray)),
                ]));
            }
            frame.render_widget(Paragraph::new(lines), content_area);
        }
        OnboardingStep::Complete => {
            let text = vec![
                Line::from(Span::styled(
                    "Configuration complete and saved to ~/.config/termexplorer/config.toml!",
                    Style::default()
                        .fg(theme.success)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::raw(""),
                Line::from("Key Shortcuts to remember:"),
                Line::from("  • F1  ─ Show complete keyboard cheat-sheet"),
                Line::from("  • F2  ─ Open selected file in code editor"),
                Line::from("  • F6  ─ Toggle embedded terminal & system monitor"),
                Line::from("  • F7  ─ Toggle side-by-side Dual Pane Explorer"),
                Line::from("  • F10 ─ Interactive Settings Studio"),
                Line::from("  • Ctrl+Shift+P ─ Central Command Palette"),
                Line::raw(""),
                Line::from("Press [Enter] or [Space] to launch TermExplorer!"),
            ];
            frame.render_widget(Paragraph::new(text), content_area);
        }
    }

    // Footer actions
    let footer_text = Line::from(vec![
        Span::styled(
            " [Enter] Next Step   ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " [Esc] Skip Wizard   ",
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(" [↑/↓] Change Option ", Style::default().fg(Color::White)),
    ]);
    frame.render_widget(Paragraph::new(footer_text), footer_area);
}
