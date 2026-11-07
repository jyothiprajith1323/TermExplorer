//! Plugin Marketplace & Extension Management UI.
//!
//! Provides a discoverable 3-tab extension studio:
//! 1. Marketplace: Browse and install curated & community plugins.
//! 2. Installed: Enable, disable, uninstall, inspect details, view logs and reload.
//! 3. Permissions: Review granted capabilities and toggle/revoke permissions.

use crate::plugins::permissions::PluginPermission;
use crate::plugins::PluginState;
use crate::state::AppState;
use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

#[derive(Debug, Clone)]
pub struct MarketplacePlugin {
    pub id: &'static str,
    pub name: &'static str,
    pub version: &'static str,
    pub author: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub icon: &'static str,
    pub permissions: &'static [PluginPermission],
    pub compatibility: &'static str,
    pub sha256: &'static str,
}

pub const MARKETPLACE_CATALOG: &[MarketplacePlugin] = &[
    MarketplacePlugin {
        id: "workspace-tools",
        name: "Workspace Tools",
        version: "0.1.0",
        author: "TermExplorer Core Team",
        description: "Workspace management actions, project metrics, and status bar telemetry",
        category: "Developer Tools",
        icon: "󰲋",
        permissions: &[
            PluginPermission::Commands,
            PluginPermission::WorkspaceActions,
            PluginPermission::StatusText,
        ],
        compatibility: ">= 1.0.0",
        sha256: "9016b629c2e145b4ce8f64ee478966b2c011800b42e39d0fe7d481ce65631af9",
    },
    MarketplacePlugin {
        id: "file-inspector",
        name: "File Inspector",
        version: "0.1.0",
        author: "TermExplorer Core Team",
        description: "Fast file size, extension, and binary heuristic telemetry inspector",
        category: "Inspection",
        icon: "󰈈",
        permissions: &[
            PluginPermission::FileMetadata,
            PluginPermission::StatusText,
            PluginPermission::Commands,
        ],
        compatibility: ">= 1.0.0",
        sha256: "9a89d0c9588b9a9c7ec18cac26a8cc9e65818e709ce6b45a99050821a8abebcd",
    },
    MarketplacePlugin {
        id: "git-supercharged",
        name: "Git Supercharged",
        version: "1.2.0",
        author: "TermExplorer Community",
        description: "Visual branch overview, uncommitted changes diffs, and staging actions",
        category: "Developer Tools",
        icon: "󰊢",
        permissions: &[
            PluginPermission::Commands,
            PluginPermission::StatusText,
            PluginPermission::WorkspaceActions,
        ],
        compatibility: ">= 1.0.0",
        sha256: "4b2e817a26f8d32b5ca51187d995c613ecf734a9b2075b630e9227181fcae912",
    },
    MarketplacePlugin {
        id: "hex-color-decorator",
        name: "Hex Color Decorator",
        version: "0.9.4",
        author: "Catppuccin Palette Team",
        description: "Automatic ANSI / CSS hex color badges and palette suggestions in preview",
        category: "UI & Themes",
        icon: "󰏘",
        permissions: &[
            PluginPermission::FileMetadata,
            PluginPermission::PreviewProvider,
        ],
        compatibility: ">= 1.0.0",
        sha256: "6d3c09e32a685fb12de88a7c64098ec9e530686940d9081a95b058c42289c07a",
    },
    MarketplacePlugin {
        id: "markdown-preview-pro",
        name: "Markdown Preview Pro",
        version: "1.1.0",
        author: "Rust Text Systems",
        description: "Extended GitHub flavor table generator, task lists, and diagram formatting",
        category: "Productivity",
        icon: "󰍔",
        permissions: &[
            PluginPermission::PreviewProvider,
            PluginPermission::FileMetadata,
        ],
        compatibility: ">= 1.0.0",
        sha256: "8e78a631bf3dc15cf398bb2c2b32f91572d478ff03714578051838d728518be4",
    },
    MarketplacePlugin {
        id: "sqlite-lens",
        name: "SQLite Lens",
        version: "0.8.2",
        author: "Data Tools Lab",
        description: "Tabular schema explorer and query stats preview for SQLite .db files",
        category: "Developer Tools",
        icon: "󰆦",
        permissions: &[
            PluginPermission::PreviewProvider,
            PluginPermission::FileMetadata,
        ],
        compatibility: ">= 1.0.0",
        sha256: "3ca5871fa997dc688f219f72782161bcf77b94d1f2b260d8293774889c168233",
    },
    MarketplacePlugin {
        id: "system-telemetry-plus",
        name: "System Telemetry Plus",
        version: "1.0.1",
        author: "SysMon Project",
        description: "Fine-grained per-core CPU and cgroup memory stats in bottom bar",
        category: "Telemetry",
        icon: "󰻠",
        permissions: &[PluginPermission::StatusText, PluginPermission::Commands],
        compatibility: ">= 1.0.0",
        sha256: "2d1b7470fcf214b7e9b04873c52a0a19fa628c68aa238ff0b852ce79018ca1ef",
    },
    MarketplacePlugin {
        id: "json-schema-validator",
        name: "JSON Schema Validator",
        version: "0.5.0",
        author: "Schema Tools",
        description: "Real-time JSON schema validation and syntax linting diagnostics",
        category: "Productivity",
        icon: "󰘦",
        permissions: &[
            PluginPermission::FileMetadata,
            PluginPermission::PreviewProvider,
        ],
        compatibility: ">= 1.0.0",
        sha256: "fa4940251c110ee8f15d9cf28135805565576a94cbcdfa56b710078872e4d8e8",
    },
];

pub const CATEGORIES: &[&str] = &[
    "All",
    "Developer Tools",
    "Inspection",
    "Productivity",
    "Telemetry",
    "UI & Themes",
];

#[derive(Debug, Clone, Copy, Default)]
pub struct PluginCenterView<'a> {
    pub tab: usize,
    pub selected_idx: usize,
    pub query: &'a str,
    pub category_idx: usize,
    pub confirm_install: Option<&'a str>,
    pub inspecting_logs: bool,
}

pub fn render_plugin_center(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: &Theme,
    view: &PluginCenterView,
) {
    let tab = view.tab;
    let selected_idx = view.selected_idx;
    let query = view.query;
    let category_idx = view.category_idx;
    let confirm_install = view.confirm_install;
    frame.render_widget(Clear, area);

    let installed_count = state.plugin_manager.plugins.len();
    let title = Line::from(vec![
        Span::styled(" ◇ PLUGIN CENTER & MARKETPLACE ", theme.header_style()),
        Span::styled(
            "─ [1/2/3 or Tab] Tabs  │  [↑/↓] Select  │  [i/Enter] Action  │  [Esc] Exit ",
            Style::default().fg(theme.size_color),
        ),
    ]);

    let outer_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(theme.border_focused_style());

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    if inner.width < 40 || inner.height < 10 {
        return;
    }

    // Vertical layout: Tab header (3 lines), Content (remaining)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8)])
        .split(inner);

    let tab_area = chunks[0];
    let content_area = chunks[1];

    // ── Tab Bar ───────────────────────────────────────────────────────────────
    let tabs = [
        format!("󰄬 Marketplace ({})", MARKETPLACE_CATALOG.len()),
        format!("󰏗 Installed ({})", installed_count),
        "󰒃 Permissions".to_string(),
    ];

    let mut tab_spans = vec![Span::raw(" ")];
    for (i, t_name) in tabs.iter().enumerate() {
        if i == tab {
            tab_spans.push(Span::styled(
                format!(" [{}] ", t_name),
                Style::default()
                    .fg(Color::Black)
                    .bg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            tab_spans.push(Span::styled(
                format!("  {}  ", t_name),
                Style::default().fg(theme.size_color),
            ));
        }
        tab_spans.push(Span::raw("  "));
    }

    let tab_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(theme.border_style());
    let tab_para = Paragraph::new(Line::from(tab_spans)).block(tab_block);
    frame.render_widget(tab_para, tab_area);

    match tab {
        0 => render_marketplace_tab(
            frame,
            content_area,
            state,
            theme,
            selected_idx,
            query,
            category_idx,
        ),
        1 => render_installed_tab(frame, content_area, state, theme, selected_idx),
        2 => render_permissions_tab(frame, content_area, state, theme, selected_idx),
        _ => {}
    }

    // ── Confirmation Modal ────────────────────────────────────────────────────
    if let Some(target_id) = confirm_install {
        render_permission_confirmation_modal(frame, area, target_id, theme);
    }
}

fn render_marketplace_tab(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: &Theme,
    selected_idx: usize,
    query: &str,
    category_idx: usize,
) {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(6)])
        .split(area);

    let filter_area = vertical[0];
    let split_area = vertical[1];

    // Filter bar
    let sel_category = CATEGORIES[category_idx.min(CATEGORIES.len() - 1)];
    let search_label = if query.is_empty() {
        "Type to search..."
    } else {
        query
    };
    let filter_line = Line::from(vec![
        Span::styled(
            " 󰍉 Filter: ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("[{}] ", search_label),
            Style::default().fg(theme.header_fg),
        ),
        Span::styled(" │ Category: ", Style::default().fg(theme.border)),
        Span::styled(
            format!("< {} > (c to cycle)", sel_category),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(Paragraph::new(filter_line), filter_area);

    // Filter catalog items
    let filtered_plugins: Vec<&MarketplacePlugin> = MARKETPLACE_CATALOG
        .iter()
        .filter(|p| {
            let cat_match = sel_category == "All" || p.category == sel_category;
            let query_match = query.is_empty()
                || p.name.to_lowercase().contains(&query.to_lowercase())
                || p.description.to_lowercase().contains(&query.to_lowercase())
                || p.id.to_lowercase().contains(&query.to_lowercase());
            cat_match && query_match
        })
        .collect();

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(split_area);

    let list_area = horizontal[0];
    let detail_area = horizontal[1];

    // Render list
    let list_items: Vec<ListItem> = filtered_plugins
        .iter()
        .enumerate()
        .map(|(i, &p)| {
            let is_sel = i == selected_idx;
            let is_installed = state
                .plugin_manager
                .plugins
                .iter()
                .any(|lp| lp.manifest.id == p.id);

            let prefix = if is_sel { "▌ " } else { "  " };
            let style = if is_sel {
                theme.selection_style()
            } else {
                theme.normal_style()
            };

            let status_span = if is_installed {
                Span::styled(
                    " [Installed] ",
                    Style::default()
                        .fg(theme.success)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    " [Install] ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )
            };

            let line = Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(format!("{} ", p.icon), Style::default().fg(theme.accent)),
                Span::styled(
                    format!("{:<20} ", p.name),
                    style.add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("v{} ", p.version),
                    Style::default().fg(theme.size_color),
                ),
                status_span,
            ]);

            ListItem::new(line)
        })
        .collect();

    let list_block = Block::default()
        .title(Span::styled(" Available Extensions ", theme.header_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focused_style());

    frame.render_widget(List::new(list_items).block(list_block), list_area);

    // Render details
    if let Some(&selected) = filtered_plugins.get(selected_idx) {
        let is_installed = state
            .plugin_manager
            .plugins
            .iter()
            .any(|lp| lp.manifest.id == selected.id);

        let mut detail_lines = Vec::new();
        detail_lines.push(Line::from(vec![
            Span::styled(
                format!("{} ", selected.icon),
                Style::default().fg(theme.accent),
            ),
            Span::styled(
                selected.name,
                Style::default()
                    .fg(theme.header_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  v{}", selected.version),
                Style::default().fg(theme.size_color),
            ),
        ]));
        detail_lines.push(Line::from(vec![
            Span::styled("Author:      ", Style::default().fg(theme.accent)),
            Span::styled(selected.author, Style::default().fg(theme.status_fg)),
        ]));
        detail_lines.push(Line::from(vec![
            Span::styled("Category:    ", Style::default().fg(theme.accent)),
            Span::styled(selected.category, Style::default().fg(theme.header_fg)),
        ]));
        detail_lines.push(Line::from(vec![
            Span::styled("Compat:      ", Style::default().fg(theme.accent)),
            Span::styled(
                selected.compatibility,
                Style::default().fg(theme.size_color),
            ),
        ]));
        detail_lines.push(Line::raw(""));
        detail_lines.push(Line::from(Span::styled(
            selected.description,
            Style::default().fg(theme.header_fg),
        )));
        detail_lines.push(Line::raw(""));
        detail_lines.push(Line::from(Span::styled(
            "Capabilities & Permissions Requested:",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )));

        for perm in selected.permissions {
            detail_lines.push(Line::from(vec![
                Span::styled("  • ", Style::default().fg(theme.warning)),
                Span::styled(
                    format!("{:<18} ", perm.as_str()),
                    Style::default()
                        .fg(theme.warning)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("— {}", perm.description()),
                    Style::default().fg(theme.status_fg),
                ),
            ]));
        }

        detail_lines.push(Line::raw(""));
        detail_lines.push(Line::from(vec![
            Span::styled("SHA256: ", Style::default().fg(theme.hidden_color)),
            Span::styled(
                &selected.sha256[..16],
                Style::default().fg(theme.hidden_color),
            ),
            Span::styled("...", Style::default().fg(theme.hidden_color)),
        ]));

        detail_lines.push(Line::raw(""));
        if is_installed {
            detail_lines.push(Line::from(vec![Span::styled(
                "✓ Extension is installed. Switch to [2: Installed] to manage.",
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            )]));
        } else {
            detail_lines.push(Line::from(vec![Span::styled(
                "Press [Enter] or [i] to review permissions & install.",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )]));
        }

        let detail_block = Block::default()
            .title(Span::styled(" Extension Details ", theme.header_style()))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme.border_style());

        frame.render_widget(
            Paragraph::new(detail_lines).block(detail_block),
            detail_area,
        );
    } else {
        let empty_para = Paragraph::new("\n  No extensions match current search query.")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty_para, detail_area);
    }
}

fn render_installed_tab(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: &Theme,
    selected_idx: usize,
) {
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    let list_area = horizontal[0];
    let detail_area = horizontal[1];

    let installed = &state.plugin_manager.plugins;

    if installed.is_empty() {
        let msg = Paragraph::new(
            "\n  No plugins installed yet.\n\n  Press [1] to open the Marketplace and install extensions.",
        )
        .style(Style::default().fg(theme.status_fg));
        frame.render_widget(msg, area);
        return;
    }

    let list_items: Vec<ListItem> = installed
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let is_sel = i == selected_idx;
            let prefix = if is_sel { "▌ " } else { "  " };
            let style = if is_sel {
                theme.selection_style()
            } else {
                theme.normal_style()
            };

            let (state_icon, state_style) = match &p.state {
                PluginState::Active => ("● Active", Style::default().fg(theme.success)),
                PluginState::Disabled => ("○ Disabled", Style::default().fg(Color::DarkGray)),
                PluginState::Error(_) => ("✕ Error", Style::default().fg(theme.error)),
            };

            let line = Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(
                    format!("{:<22} ", p.manifest.name),
                    style.add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("v{:<6} ", p.manifest.version),
                    Style::default().fg(theme.size_color),
                ),
                Span::styled(
                    format!("[{}]", state_icon),
                    state_style.add_modifier(Modifier::BOLD),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list_block = Block::default()
        .title(Span::styled(" Installed Extensions ", theme.header_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focused_style());

    frame.render_widget(List::new(list_items).block(list_block), list_area);

    // Selected plugin detail
    if let Some(selected) = installed.get(selected_idx) {
        let mut detail_lines = Vec::new();
        detail_lines.push(Line::from(vec![
            Span::styled("󰏗 ", Style::default().fg(theme.accent)),
            Span::styled(
                &selected.manifest.name,
                Style::default()
                    .fg(theme.header_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  v{}", selected.manifest.version),
                Style::default().fg(theme.size_color),
            ),
        ]));

        detail_lines.push(Line::from(vec![
            Span::styled("State:        ", Style::default().fg(theme.accent)),
            Span::styled(
                selected.state.as_str(),
                Style::default().fg(theme.header_fg),
            ),
        ]));

        detail_lines.push(Line::from(vec![
            Span::styled("Path:         ", Style::default().fg(theme.accent)),
            Span::styled(
                selected.path.display().to_string(),
                Style::default().fg(theme.status_fg),
            ),
        ]));

        if let Some(desc) = &selected.manifest.description {
            detail_lines.push(Line::raw(""));
            detail_lines.push(Line::from(Span::styled(
                desc.clone(),
                Style::default().fg(theme.header_fg),
            )));
        }

        detail_lines.push(Line::raw(""));
        detail_lines.push(Line::from(Span::styled(
            "Granted Permissions:",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )));

        for perm in &selected.manifest.permissions {
            detail_lines.push(Line::from(vec![
                Span::styled("  ✔ ", Style::default().fg(theme.success)),
                Span::styled(
                    format!("{:<18} ", perm.as_str()),
                    Style::default()
                        .fg(theme.success)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("— {}", perm.description()),
                    Style::default().fg(theme.status_fg),
                ),
            ]));
        }

        detail_lines.push(Line::raw(""));
        detail_lines.push(Line::from(Span::styled(
            "Management Actions:",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )));
        detail_lines.push(Line::from(vec![
            Span::styled(
                "[Space] ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Toggle Active / Disabled  │  ",
                Style::default().fg(theme.status_fg),
            ),
            Span::styled(
                "[d] ",
                Style::default()
                    .fg(theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Uninstall  │  ", Style::default().fg(theme.status_fg)),
            Span::styled(
                "[r] ",
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Reload", Style::default().fg(theme.status_fg)),
        ]));

        let detail_block = Block::default()
            .title(Span::styled(" Extension Management ", theme.header_style()))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme.border_style());

        frame.render_widget(
            Paragraph::new(detail_lines).block(detail_block),
            detail_area,
        );
    }
}

fn render_permissions_tab(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: &Theme,
    _selected_idx: usize,
) {
    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        "Extension Security & Sandbox Capabilities Overview",
        theme.header_style(),
    )));
    lines.push(Line::from(Span::styled(
        "TermExplorer enforces strict WebAssembly sandbox boundaries. Plugins cannot access capabilities unless explicitly granted.",
        Style::default().fg(theme.status_fg),
    )));
    lines.push(Line::raw(""));

    let all_perms = [
        PluginPermission::Commands,
        PluginPermission::ContextActions,
        PluginPermission::WorkspaceActions,
        PluginPermission::FileMetadata,
        PluginPermission::PreviewProvider,
        PluginPermission::StatusText,
    ];

    for perm in all_perms {
        let granted_plugins: Vec<&str> = state
            .plugin_manager
            .plugins
            .iter()
            .filter(|p| p.manifest.permissions.contains(&perm))
            .map(|p| p.manifest.name.as_str())
            .collect();

        let count = granted_plugins.len();
        let plugin_names = if count > 0 {
            granted_plugins.join(", ")
        } else {
            "None".to_string()
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!("󰒃 {:<20} ", perm.as_str()),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("— {}", perm.description()),
                Style::default().fg(theme.header_fg),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("    Active Grants: ", Style::default().fg(theme.size_color)),
            Span::styled(
                plugin_names,
                if count > 0 {
                    Style::default().fg(theme.success)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
        ]));
        lines.push(Line::raw(""));
    }

    let block = Block::default()
        .title(Span::styled(" Capability Matrix ", theme.header_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focused_style());

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_permission_confirmation_modal(
    frame: &mut Frame,
    area: Rect,
    target_id: &str,
    theme: &Theme,
) {
    let plugin = MARKETPLACE_CATALOG.iter().find(|p| p.id == target_id);
    let name = plugin.map(|p| p.name).unwrap_or(target_id);
    let perms: Vec<PluginPermission> = plugin
        .map(|p| p.permissions.to_vec())
        .unwrap_or_else(|| vec![PluginPermission::Commands, PluginPermission::StatusText]);

    let modal_w = (area.width * 60 / 100).max(50).min(area.width);
    let modal_h = 16u16.min(area.height);
    let x = (area.width.saturating_sub(modal_w)) / 2;
    let y = (area.height.saturating_sub(modal_h)) / 2;
    let modal_area = Rect::new(x, y, modal_w, modal_h);

    frame.render_widget(Clear, modal_area);

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        format!("Install extension '{}'?", name),
        Style::default()
            .fg(theme.header_fg)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        "This extension requests access to the following security capabilities:",
        Style::default().fg(theme.warning),
    )));
    lines.push(Line::raw(""));

    for perm in perms {
        lines.push(Line::from(vec![
            Span::styled("  • ", Style::default().fg(theme.warning)),
            Span::styled(
                format!("{:<18} ", perm.as_str()),
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(": {}", perm.description()),
                Style::default().fg(theme.status_fg),
            ),
        ]));
    }

    lines.push(Line::raw(""));
    lines.push(Line::from(vec![Span::styled(
        "Press [y / Enter] to Grant & Install  │  [n / Esc] to Cancel",
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )]));

    let block = Block::default()
        .title(Span::styled(
            " 󰒃 PERMISSION GRANT CONFIRMATION ",
            theme.header_style(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(theme.border_focused_style());

    frame.render_widget(Paragraph::new(lines).block(block), modal_area);
}
