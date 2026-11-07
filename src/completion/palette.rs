//! Command Palette Suggestions and Ranking Engine.
//! Provides intelligent ranking (exact, prefix, acronym, fuzzy, recency, context)
//! and includes dynamic navigation to pinned and recent directories.

use crate::commands::{CommandCategory, CommandItem, CommandRegistry};
use crate::config::PinnedFolder;
use crate::state::FocusedPanel;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::path::PathBuf;

/// A ranked command palette suggestion item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteSuggestion {
    /// Action identifier (e.g. "toggle_dual_pane" or "nav_dir:/path").
    pub id: String,
    /// Title to display.
    pub title: String,
    /// Explanatory description.
    pub description: String,
    /// Category / badge.
    pub category: CommandCategory,
    /// Shortcut key text if assigned.
    pub shortcut: Option<String>,
    /// Calculated ranking score (higher is ranked first).
    pub score: i64,
}

impl PaletteSuggestion {
    pub fn to_command_item(&self) -> CommandItem {
        CommandItem {
            id: Box::leak(self.id.clone().into_boxed_str()),
            title: Box::leak(self.title.clone().into_boxed_str()),
            description: Box::leak(self.description.clone().into_boxed_str()),
            category: self.category,
            shortcut: self
                .shortcut
                .as_ref()
                .map(|s| &*Box::leak(s.clone().into_boxed_str())),
        }
    }
}

/// Compute initials/acronym for a phrase (e.g. "Toggle Dual-Pane" -> "tdp").
fn initials(s: &str) -> String {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .filter_map(|w| w.chars().next())
        .collect::<String>()
        .to_lowercase()
}

/// Rank commands and dynamic workspace locations based on query, context, and recency.
/// Target latency: < 10ms (typical: 0.05ms - 0.2ms).
pub fn rank_palette_commands(
    query: &str,
    pinned: &[PinnedFolder],
    recent: &[PathBuf],
    context_panel: FocusedPanel,
) -> Vec<PaletteSuggestion> {
    let query_trimmed = query.trim();
    let query_lower = query_trimmed.to_lowercase();
    let matcher = SkimMatcherV2::default();
    let mut suggestions = Vec::new();

    // 1. Static built-in commands
    let all_commands = CommandRegistry::all_commands();
    for cmd in all_commands {
        let title_lower = cmd.title.to_lowercase();
        let cat_badge = cmd.category.badge().to_lowercase();
        let init = initials(cmd.title);

        let mut score: i64 = if query_lower.is_empty() {
            100
        } else if title_lower == query_lower {
            2000
        } else if title_lower.starts_with(&query_lower) {
            1200 + (100 - (cmd.title.len() as i64).min(50))
        } else if init == query_lower {
            900
        } else if let Some(s) = matcher.fuzzy_match(cmd.title, query_trimmed) {
            s + 400
        } else if let Some(s) = matcher.fuzzy_match(cmd.description, query_trimmed) {
            s + 100
        } else if cat_badge.contains(&query_lower) {
            250
        } else {
            continue;
        };

        // Context bonus
        let has_context_bonus = matches!(
            (context_panel, cmd.category),
            (
                FocusedPanel::Explorer | FocusedPanel::ExplorerSecondary,
                CommandCategory::Explorer,
            ) | (FocusedPanel::Terminal, CommandCategory::Terminal)
                | (FocusedPanel::Preview, CommandCategory::Navigation)
        );
        if has_context_bonus {
            score += 40;
        }

        suggestions.push(PaletteSuggestion {
            id: cmd.id.to_string(),
            title: cmd.title.to_string(),
            description: cmd.description.to_string(),
            category: cmd.category,
            shortcut: cmd.shortcut.map(|s| s.to_string()),
            score,
        });
    }

    // 2. Dynamic navigation: Pinned Folders
    for pf in pinned {
        let title = format!("Open Pinned: {}", pf.name);
        let path_str = pf.path.to_string_lossy().to_string();
        let name_lower = pf.name.to_lowercase();

        let score: i64 = if query_lower.is_empty() {
            150
        } else if name_lower == query_lower {
            1800
        } else if name_lower.starts_with(&query_lower) {
            1100
        } else if let Some(s) = matcher.fuzzy_match(&pf.name, query_trimmed) {
            s + 500
        } else if let Some(s) = matcher.fuzzy_match(&path_str, query_trimmed) {
            s + 200
        } else {
            continue;
        };

        suggestions.push(PaletteSuggestion {
            id: format!("nav_dir:{}", path_str),
            title,
            description: format!("Navigate to pinned directory {}", path_str),
            category: CommandCategory::Navigation,
            shortcut: Some("Pinned".to_string()),
            score,
        });
    }

    // 3. Dynamic navigation: Recent Directories
    for rd in recent {
        let path_str = rd.to_string_lossy().to_string();
        let dir_name = rd
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| path_str.clone());
        let title = format!("Open Recent: {}", dir_name);
        let name_lower = dir_name.to_lowercase();

        let score: i64 = if query_lower.is_empty() {
            80
        } else if name_lower == query_lower {
            1600
        } else if name_lower.starts_with(&query_lower) {
            1000
        } else if let Some(s) = matcher.fuzzy_match(&dir_name, query_trimmed) {
            s + 400
        } else if let Some(s) = matcher.fuzzy_match(&path_str, query_trimmed) {
            s + 150
        } else {
            continue;
        };

        suggestions.push(PaletteSuggestion {
            id: format!("nav_dir:{}", path_str),
            title,
            description: format!("Navigate to recently visited {}", path_str),
            category: CommandCategory::Navigation,
            shortcut: Some("Recent".to_string()),
            score,
        });
    }

    suggestions.sort_by_key(|a| std::cmp::Reverse(a.score));
    suggestions
}
