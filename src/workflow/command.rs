//! Centralized Application Command Registry & Fuzzy Search Palette.
//!
//! Provides a single authoritative registry for all actions, keyboard shortcuts,
//! context-aware availability predicates, and fuzzy search rankings.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::ui::hit_test::AppAction;

/// Categories of commands available in TermExplorer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandCategory {
    File,
    Navigation,
    Editor,
    Preview,
    Terminal,
    Git,
    Tasks,
    Settings,
    Help,
}

impl CommandCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::File => "File",
            Self::Navigation => "Navigation",
            Self::Editor => "Editor",
            Self::Preview => "Preview",
            Self::Terminal => "Terminal",
            Self::Git => "Git",
            Self::Tasks => "Tasks",
            Self::Settings => "Settings",
            Self::Help => "Help",
        }
    }
}

/// Dynamic contextual state of TermExplorer used to evaluate command availability.
#[derive(Debug, Clone, Default)]
pub struct CommandContext {
    pub has_git: bool,
    pub is_editor_mode: bool,
    pub is_preview_mode: bool,
    pub has_terminal_session: bool,
    pub has_project_tasks: bool,
    pub has_selection: bool,
    pub is_dirty: bool,
    pub selected_path: Option<PathBuf>,
    pub current_dir: PathBuf,
}

pub type CustomActionFn = Arc<dyn Fn(&CommandContext) -> Option<AppAction> + Send + Sync>;

/// Action to dispatch when a command is executed.
#[derive(Clone)]
pub enum CommandAction {
    App(AppAction),
    RunTask(String),
    Custom(CustomActionFn),
}

impl std::fmt::Debug for CommandAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::App(act) => write!(f, "App({:?})", act),
            Self::RunTask(id) => write!(f, "RunTask({})", id),
            Self::Custom(_) => write!(f, "Custom(<fn>)"),
        }
    }
}

/// Declarative specification of a single application command.
#[derive(Clone)]
pub struct CommandDefinition {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub category: CommandCategory,
    pub keywords: Vec<String>,
    pub default_shortcut: Option<String>,
    pub availability: Arc<dyn Fn(&CommandContext) -> bool + Send + Sync>,
    pub action: CommandAction,
}

impl std::fmt::Debug for CommandDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandDefinition")
            .field("id", &self.id)
            .field("display_name", &self.display_name)
            .field("description", &self.description)
            .field("category", &self.category)
            .field("keywords", &self.keywords)
            .field("default_shortcut", &self.default_shortcut)
            .finish()
    }
}

/// Fuzzy search match result for display in the Command Palette.
#[derive(Debug, Clone)]
pub struct CommandSearchResult {
    pub command: CommandDefinition,
    pub score: i64,
    pub is_available: bool,
    pub is_recent: bool,
}

/// Bounded history of recently executed command IDs.
#[derive(Debug, Clone, Default)]
pub struct CommandHistory {
    entries: VecDeque<String>,
    max_entries: usize,
}

impl CommandHistory {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_entries),
            max_entries,
        }
    }

    pub fn record(&mut self, command_id: impl Into<String>) {
        let id = command_id.into();
        self.entries.retain(|existing| existing != &id);
        self.entries.push_front(id);
        if self.entries.len() > self.max_entries {
            self.entries.pop_back();
        }
    }

    pub fn contains(&self, command_id: &str) -> bool {
        self.entries.iter().any(|id| id == command_id)
    }

    pub fn entries(&self) -> &VecDeque<String> {
        &self.entries
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// Authoritative central registry of application commands.
pub struct CommandRegistry {
    commands: Vec<CommandDefinition>,
    matcher: SkimMatcherV2,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut reg = Self {
            commands: Vec::with_capacity(64),
            matcher: SkimMatcherV2::default(),
        };
        reg.register_default_commands();
        reg
    }

    pub fn register(&mut self, command: CommandDefinition) {
        // Enforce uniqueness by replacing duplicate command IDs
        self.commands.retain(|c| c.id != command.id);
        self.commands.push(command);
    }

    pub fn find(&self, id: &str) -> Option<&CommandDefinition> {
        self.commands.iter().find(|c| c.id == id)
    }

    pub fn commands(&self) -> &[CommandDefinition] {
        &self.commands
    }

    /// Searches commands by matching against name, description, category and keywords,
    /// ranking recent commands appropriately.
    pub fn search(
        &self,
        query: &str,
        ctx: &CommandContext,
        history: &CommandHistory,
    ) -> Vec<CommandSearchResult> {
        let trimmed = query.trim();

        if trimmed.is_empty() {
            // Empty query: Show recent commands first, then all available commands grouped by category
            let mut results = Vec::new();

            // First: Recent commands
            for rec_id in history.entries() {
                if let Some(cmd) = self.find(rec_id) {
                    let available = (cmd.availability)(ctx);
                    results.push(CommandSearchResult {
                        command: cmd.clone(),
                        score: 1000,
                        is_available: available,
                        is_recent: true,
                    });
                }
            }

            // Then: Other commands
            for cmd in &self.commands {
                if !history.contains(&cmd.id) {
                    let available = (cmd.availability)(ctx);
                    results.push(CommandSearchResult {
                        command: cmd.clone(),
                        score: 0,
                        is_available: available,
                        is_recent: false,
                    });
                }
            }

            return results;
        }

        let mut scored_results = Vec::new();

        for cmd in &self.commands {
            let available = (cmd.availability)(ctx);

            // Compute fuzzy scores across name, category, description, and keywords
            let name_score = self
                .matcher
                .fuzzy_match(&cmd.display_name, trimmed)
                .unwrap_or(0);
            let cat_score = self
                .matcher
                .fuzzy_match(cmd.category.label(), trimmed)
                .unwrap_or(0);
            let desc_score = self
                .matcher
                .fuzzy_match(&cmd.description, trimmed)
                .unwrap_or(0);

            let mut keyword_score = 0;
            for kw in &cmd.keywords {
                let s = self.matcher.fuzzy_match(kw, trimmed).unwrap_or(0);
                if s > keyword_score {
                    keyword_score = s;
                }
            }

            // Best match among fields
            let best_score = name_score.max(cat_score).max(desc_score).max(keyword_score);

            if best_score > 0 {
                let is_recent = history.contains(&cmd.id);
                // Bonus for recency and exact name prefix matches
                let mut total_score = best_score;
                if is_recent {
                    total_score += 15;
                }
                if cmd
                    .display_name
                    .to_lowercase()
                    .starts_with(&trimmed.to_lowercase())
                {
                    total_score += 50;
                }
                if !available {
                    total_score -= 100;
                }

                scored_results.push(CommandSearchResult {
                    command: cmd.clone(),
                    score: total_score,
                    is_available: available,
                    is_recent,
                });
            }
        }

        // Sort descending by score
        scored_results.sort_by_key(|a| std::cmp::Reverse(a.score));
        scored_results
    }

    /// Registers built-in commands covering all TermExplorer capabilities.
    fn register_default_commands(&mut self) {
        // ── Navigation & Explorer ─────────────────────────────────────────────
        self.register(CommandDefinition {
            id: "explorer.focus".to_string(),
            display_name: "Focus File Explorer".to_string(),
            description: "Shift focus to the 25% File Explorer pane".to_string(),
            category: CommandCategory::Navigation,
            keywords: vec!["file".into(), "tree".into(), "browse".into()],
            default_shortcut: Some("Alt+1".into()),
            availability: Arc::new(|_| true),
            action: CommandAction::App(AppAction::FocusExplorer),
        });

        self.register(CommandDefinition {
            id: "preview.focus".to_string(),
            display_name: "Focus Preview / Editor".to_string(),
            description: "Shift focus to the 75% Inspector or Editor pane".to_string(),
            category: CommandCategory::Navigation,
            keywords: vec!["inspect".into(), "viewer".into(), "code".into()],
            default_shortcut: Some("Alt+2".into()),
            availability: Arc::new(|_| true),
            action: CommandAction::App(AppAction::FocusPreview),
        });

        self.register(CommandDefinition {
            id: "search.open".to_string(),
            display_name: "Search Workspace".to_string(),
            description: "Open unified fuzzy file and content search".to_string(),
            category: CommandCategory::File,
            keywords: vec!["find".into(), "grep".into(), "query".into()],
            default_shortcut: Some("Ctrl+F".into()),
            availability: Arc::new(|_| true),
            action: CommandAction::App(AppAction::OpenSearch),
        });

        // ── Terminal Management ───────────────────────────────────────────────
        self.register(CommandDefinition {
            id: "terminal.toggle".to_string(),
            display_name: "Toggle Terminal Drawer".to_string(),
            description: "Open or close the 15% interactive terminal drawer".to_string(),
            category: CommandCategory::Terminal,
            keywords: vec!["shell".into(), "bash".into(), "pty".into()],
            default_shortcut: Some("Ctrl+\\".into()),
            availability: Arc::new(|_| true),
            action: CommandAction::App(AppAction::ToggleTerminal),
        });

        self.register(CommandDefinition {
            id: "terminal.new_session".to_string(),
            display_name: "Terminal: New Session".to_string(),
            description: "Spawn a new interactive PTY shell tab".to_string(),
            category: CommandCategory::Terminal,
            keywords: vec!["new tab".into(), "shell".into(), "add".into()],
            default_shortcut: Some("Ctrl+Shift+T".into()),
            availability: Arc::new(|ctx| ctx.has_terminal_session),
            action: CommandAction::App(AppAction::FocusTerminal),
        });

        self.register(CommandDefinition {
            id: "terminal.close_session".to_string(),
            display_name: "Terminal: Close Current Session".to_string(),
            description: "Terminate the active PTY session".to_string(),
            category: CommandCategory::Terminal,
            keywords: vec!["kill".into(), "exit".into(), "tab".into()],
            default_shortcut: Some("Ctrl+Shift+W".into()),
            availability: Arc::new(|ctx| ctx.has_terminal_session),
            action: CommandAction::App(AppAction::CloseTerminalSession),
        });

        self.register(CommandDefinition {
            id: "terminal.restart".to_string(),
            display_name: "Terminal: Restart Session".to_string(),
            description: "Restart the active terminal session process".to_string(),
            category: CommandCategory::Terminal,
            keywords: vec!["reset".into(), "reboot".into(), "respawn".into()],
            default_shortcut: None,
            availability: Arc::new(|ctx| ctx.has_terminal_session),
            action: CommandAction::App(AppAction::RestartTerminal),
        });

        self.register(CommandDefinition {
            id: "terminal.maximize".to_string(),
            display_name: "Terminal: Maximize Drawer".to_string(),
            description: "Expand the terminal drawer to fill the screen".to_string(),
            category: CommandCategory::Terminal,
            keywords: vec!["fullscreen".into(), "grow".into(), "expand".into()],
            default_shortcut: Some("Alt+F6".into()),
            availability: Arc::new(|ctx| ctx.has_terminal_session),
            action: CommandAction::App(AppAction::MaximizeTerminal),
        });

        // ── Git Workflows ─────────────────────────────────────────────────────
        self.register(CommandDefinition {
            id: "git.dashboard".to_string(),
            display_name: "Git: Open Dashboard".to_string(),
            description: "Open the 4-tab Git workbench (Status, Diff, Branches, History)"
                .to_string(),
            category: CommandCategory::Git,
            keywords: vec![
                "vcs".into(),
                "branch".into(),
                "commit".into(),
                "status".into(),
            ],
            default_shortcut: Some("Ctrl+G".into()),
            availability: Arc::new(|ctx| ctx.has_git),
            action: CommandAction::App(AppAction::OpenGitDashboard),
        });

        self.register(CommandDefinition {
            id: "git.stage_file".to_string(),
            display_name: "Git: Stage Selected File".to_string(),
            description: "Stage the currently selected file to the Git index".to_string(),
            category: CommandCategory::Git,
            keywords: vec!["add".into(), "index".into()],
            default_shortcut: Some("S".into()),
            availability: Arc::new(|ctx| ctx.has_git && ctx.has_selection),
            action: CommandAction::App(AppAction::OpenGitDashboard),
        });

        self.register(CommandDefinition {
            id: "git.commit".to_string(),
            display_name: "Git: Create Commit".to_string(),
            description: "Open commit message dialog to commit staged changes".to_string(),
            category: CommandCategory::Git,
            keywords: vec!["commit".into(), "checkin".into(), "save".into()],
            default_shortcut: Some("C".into()),
            availability: Arc::new(|ctx| ctx.has_git),
            action: CommandAction::App(AppAction::OpenGitDashboard),
        });

        // ── Developer Tasks & Workflows ───────────────────────────────────────
        self.register(CommandDefinition {
            id: "task.run_project".to_string(),
            display_name: "Tasks: Run Project".to_string(),
            description: "Run the project's primary execution task".to_string(),
            category: CommandCategory::Tasks,
            keywords: vec!["start".into(), "execute".into(), "run".into()],
            default_shortcut: Some("F5".into()),
            availability: Arc::new(|ctx| ctx.has_project_tasks),
            action: CommandAction::RunTask("run".to_string()),
        });

        self.register(CommandDefinition {
            id: "task.run_tests".to_string(),
            display_name: "Tasks: Run Tests".to_string(),
            description: "Run project unit and integration test suite".to_string(),
            category: CommandCategory::Tasks,
            keywords: vec!["cargo test".into(), "pytest".into(), "npm test".into()],
            default_shortcut: Some("F6".into()),
            availability: Arc::new(|ctx| ctx.has_project_tasks),
            action: CommandAction::RunTask("test".to_string()),
        });

        self.register(CommandDefinition {
            id: "task.build".to_string(),
            display_name: "Tasks: Build Project".to_string(),
            description: "Compile and build project targets".to_string(),
            category: CommandCategory::Tasks,
            keywords: vec!["cargo build".into(), "compile".into(), "make".into()],
            default_shortcut: Some("F7".into()),
            availability: Arc::new(|ctx| ctx.has_project_tasks),
            action: CommandAction::RunTask("build".to_string()),
        });

        self.register(CommandDefinition {
            id: "task.lint".to_string(),
            display_name: "Tasks: Run Linter / Check".to_string(),
            description: "Run static analysis checks (e.g. clippy, eslint, flake8)".to_string(),
            category: CommandCategory::Tasks,
            keywords: vec!["clippy".into(), "check".into(), "lint".into()],
            default_shortcut: Some("F8".into()),
            availability: Arc::new(|ctx| ctx.has_project_tasks),
            action: CommandAction::RunTask("lint".to_string()),
        });

        // ── Settings & Diagnostics ────────────────────────────────────────────
        self.register(CommandDefinition {
            id: "settings.open".to_string(),
            display_name: "Open Settings".to_string(),
            description: "Configure themes, keybindings, and terminal preferences".to_string(),
            category: CommandCategory::Settings,
            keywords: vec!["config".into(), "preferences".into(), "theme".into()],
            default_shortcut: Some("Ctrl+,".into()),
            availability: Arc::new(|_| true),
            action: CommandAction::App(AppAction::OpenSettings),
        });

        self.register(CommandDefinition {
            id: "plugins.open".to_string(),
            display_name: "Open Plugin Center".to_string(),
            description: "Inspect WebAssembly plugins and extensions".to_string(),
            category: CommandCategory::Settings,
            keywords: vec!["wasm".into(), "addons".into(), "extensions".into()],
            default_shortcut: None,
            availability: Arc::new(|_| true),
            action: CommandAction::App(AppAction::OpenPluginCenter),
        });

        self.register(CommandDefinition {
            id: "help.open".to_string(),
            display_name: "Open Help & Keybindings".to_string(),
            description: "View complete keyboard and mouse reference".to_string(),
            category: CommandCategory::Help,
            keywords: vec!["docs".into(), "shortcuts".into(), "keys".into()],
            default_shortcut: Some("?".into()),
            availability: Arc::new(|_| true),
            action: CommandAction::App(AppAction::OpenHelp),
        });

        self.register(CommandDefinition {
            id: "system.diagnostics".to_string(),
            display_name: "Toggle System Telemetry".to_string(),
            description: "Show CPU, RAM, disk I/O and process telemetry".to_string(),
            category: CommandCategory::Settings,
            keywords: vec!["perf".into(), "cpu".into(), "ram".into(), "monitor".into()],
            default_shortcut: Some("F12".into()),
            availability: Arc::new(|_| true),
            action: CommandAction::App(AppAction::ToggleDiagnostics),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_registry_default_population() {
        let registry = CommandRegistry::new();
        assert!(registry.commands().len() >= 12);
        assert!(registry.find("git.dashboard").is_some());
        assert!(registry.find("terminal.toggle").is_some());
        assert!(registry.find("task.run_tests").is_some());
    }

    #[test]
    fn test_command_registry_fuzzy_search() {
        let registry = CommandRegistry::new();
        let history = CommandHistory::new(10);
        let ctx = CommandContext {
            has_git: true,
            has_project_tasks: true,
            has_terminal_session: true,
            ..Default::default()
        };

        let results = registry.search("git", &ctx, &history);
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.command.id == "git.dashboard"));

        let terminal_results = registry.search("term", &ctx, &history);
        assert!(!terminal_results.is_empty());
        assert!(terminal_results
            .iter()
            .any(|r| r.command.id == "terminal.toggle"));
    }

    #[test]
    fn test_command_availability_predicate() {
        let registry = CommandRegistry::new();
        let history = CommandHistory::new(10);

        // When has_git is false
        let ctx_no_git = CommandContext {
            has_git: false,
            ..Default::default()
        };
        let results = registry.search("git.dashboard", &ctx_no_git, &history);
        if let Some(r) = results.iter().find(|r| r.command.id == "git.dashboard") {
            assert!(!r.is_available);
        }

        // When has_git is true
        let ctx_git = CommandContext {
            has_git: true,
            ..Default::default()
        };
        let results = registry.search("git.dashboard", &ctx_git, &history);
        if let Some(r) = results.iter().find(|r| r.command.id == "git.dashboard") {
            assert!(r.is_available);
        }
    }

    #[test]
    fn test_command_history_recency_ordering() {
        let mut history = CommandHistory::new(5);
        history.record("terminal.toggle");
        history.record("git.dashboard");

        assert_eq!(history.entries().len(), 2);
        assert_eq!(history.entries()[0], "git.dashboard");
        assert_eq!(history.entries()[1], "terminal.toggle");

        history.record("terminal.toggle");
        assert_eq!(history.entries()[0], "terminal.toggle");
        assert_eq!(history.entries()[1], "git.dashboard");
    }
}
