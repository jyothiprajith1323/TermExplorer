//! Keybinding management with scope separation, conflict detection, and search.

use crossterm::event::{KeyCode, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingScope {
    Global,
    Explorer,
    Editor,
    Terminal,
    Preview,
    Git,
    Search,
    Workflow,
}

impl BindingScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Global => "Global",
            Self::Explorer => "Explorer",
            Self::Editor => "Editor",
            Self::Terminal => "Terminal",
            Self::Preview => "Preview",
            Self::Git => "Git",
            Self::Search => "Search",
            Self::Workflow => "Workflow",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyActionBinding {
    pub id: &'static str,
    pub name: &'static str,
    pub scope: BindingScope,
    pub primary_key: String,
    pub description: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictReport {
    pub has_conflicts: bool,
    pub details: Vec<String>,
}

pub struct KeybindingManager;

impl KeybindingManager {
    /// Return all default keybindings categorized by scope.
    pub fn default_bindings() -> Vec<KeyActionBinding> {
        vec![
            // ── Global ──────────────────────────────────────────────────────────
            KeyActionBinding {
                id: "help",
                name: "Show Help Overlay",
                scope: BindingScope::Global,
                primary_key: "F1 / ?".into(),
                description: "Display keyboard shortcuts cheat-sheet",
            },
            KeyActionBinding {
                id: "edit",
                name: "Open in Editor",
                scope: BindingScope::Global,
                primary_key: "F2".into(),
                description: "Open selected file in integrated code editor",
            },
            KeyActionBinding {
                id: "preview_zoom",
                name: "Toggle Preview Zoom",
                scope: BindingScope::Global,
                primary_key: "F3".into(),
                description: "Maximize or restore right preview panel",
            },
            KeyActionBinding {
                id: "refresh",
                name: "Refresh View",
                scope: BindingScope::Global,
                primary_key: "F5".into(),
                description: "Reload active directory and system stats",
            },
            KeyActionBinding {
                id: "monitor",
                name: "Toggle System Monitor",
                scope: BindingScope::Global,
                primary_key: "F6".into(),
                description: "Show/hide bottom system telemetry monitor",
            },
            KeyActionBinding {
                id: "dual_pane",
                name: "Toggle Dual Pane Explorer",
                scope: BindingScope::Global,
                primary_key: "F7".into(),
                description: "Toggle side-by-side dual explorer panes",
            },
            KeyActionBinding {
                id: "settings",
                name: "Settings Studio",
                scope: BindingScope::Global,
                primary_key: "F10 / Ctrl+,".into(),
                description: "Open interactive settings application",
            },
            KeyActionBinding {
                id: "cmd_palette",
                name: "Command Palette",
                scope: BindingScope::Global,
                primary_key: "Ctrl+Shift+P".into(),
                description: "Fuzzy-search commands, tasks, git, and files",
            },
            KeyActionBinding {
                id: "term_drawer",
                name: "Toggle Terminal Drawer",
                scope: BindingScope::Global,
                primary_key: "` / Ctrl-`".into(),
                description: "Toggle embedded PTY terminal drawer (15% height)",
            },
            KeyActionBinding {
                id: "cycle_focus",
                name: "Cycle Focus",
                scope: BindingScope::Global,
                primary_key: "Tab / Shift-Tab".into(),
                description: "Cycle focus between Explorer, Preview, and Terminal",
            },
            KeyActionBinding {
                id: "quit",
                name: "Quit Application",
                scope: BindingScope::Global,
                primary_key: "Ctrl+Q / q".into(),
                description: "Cleanly restore terminal and exit",
            },
            // ── Explorer ────────────────────────────────────────────────────────
            KeyActionBinding {
                id: "nav_down",
                name: "Move Down",
                scope: BindingScope::Explorer,
                primary_key: "Down / j".into(),
                description: "Highlight next file or folder",
            },
            KeyActionBinding {
                id: "nav_up",
                name: "Move Up",
                scope: BindingScope::Explorer,
                primary_key: "Up / k".into(),
                description: "Highlight previous file or folder",
            },
            KeyActionBinding {
                id: "open",
                name: "Enter Directory",
                scope: BindingScope::Explorer,
                primary_key: "Right / Enter / l".into(),
                description: "Open selected directory or inspect file",
            },
            KeyActionBinding {
                id: "parent",
                name: "Parent Directory",
                scope: BindingScope::Explorer,
                primary_key: "Left / Backspace / h".into(),
                description: "Navigate to parent folder",
            },
            KeyActionBinding {
                id: "top_bottom",
                name: "Jump Top / Bottom",
                scope: BindingScope::Explorer,
                primary_key: "gg / G".into(),
                description: "Jump to first or last item in file list",
            },
            KeyActionBinding {
                id: "new_file",
                name: "New File",
                scope: BindingScope::Explorer,
                primary_key: "a".into(),
                description: "Create new empty file in current directory",
            },
            KeyActionBinding {
                id: "new_dir",
                name: "New Directory",
                scope: BindingScope::Explorer,
                primary_key: "A".into(),
                description: "Create new subdirectory in current directory",
            },
            KeyActionBinding {
                id: "rename",
                name: "Rename Item",
                scope: BindingScope::Explorer,
                primary_key: "r".into(),
                description: "Rename selected file or folder",
            },
            KeyActionBinding {
                id: "delete",
                name: "Delete Item",
                scope: BindingScope::Explorer,
                primary_key: "d / Delete".into(),
                description: "Send to FreeDesktop Trash or confirm delete",
            },
            KeyActionBinding {
                id: "search_filter",
                name: "Fuzzy Filter",
                scope: BindingScope::Explorer,
                primary_key: "/".into(),
                description: "Filter files in current directory",
            },
            KeyActionBinding {
                id: "hidden",
                name: "Toggle Hidden Files",
                scope: BindingScope::Explorer,
                primary_key: "zh / .".into(),
                description: "Show or hide dotfiles and hidden folders",
            },
            // ── Editor ──────────────────────────────────────────────────────────
            KeyActionBinding {
                id: "save",
                name: "Atomic Save",
                scope: BindingScope::Editor,
                primary_key: "Ctrl+S".into(),
                description: "Atomically persist rope buffer to disk",
            },
            KeyActionBinding {
                id: "undo",
                name: "Undo Edit",
                scope: BindingScope::Editor,
                primary_key: "Ctrl+Z".into(),
                description: "Revert last editing operation",
            },
            KeyActionBinding {
                id: "redo",
                name: "Redo Edit",
                scope: BindingScope::Editor,
                primary_key: "Ctrl+Y".into(),
                description: "Reapply reverted edit",
            },
            KeyActionBinding {
                id: "complete",
                name: "Accept Completion",
                scope: BindingScope::Editor,
                primary_key: "Tab / Enter".into(),
                description: "Insert selected word completion popup item",
            },
            KeyActionBinding {
                id: "close_editor",
                name: "Close Editor",
                scope: BindingScope::Editor,
                primary_key: "Esc".into(),
                description: "Return to file explorer view",
            },
            // ── Terminal ────────────────────────────────────────────────────────
            KeyActionBinding {
                id: "term_interrupt",
                name: "Send SIGINT",
                scope: BindingScope::Terminal,
                primary_key: "Ctrl+C".into(),
                description: "Interrupt foreground process in PTY",
            },
            KeyActionBinding {
                id: "term_eof",
                name: "Send EOF",
                scope: BindingScope::Terminal,
                primary_key: "Ctrl+D".into(),
                description: "Send end-of-file signal to active shell",
            },
            // ── Git & Developer Tools ───────────────────────────────────────────
            KeyActionBinding {
                id: "git_panel",
                name: "Git Workbench",
                scope: BindingScope::Git,
                primary_key: "g".into(),
                description: "Open Git status, diff viewer, and branch manager",
            },
            KeyActionBinding {
                id: "project_search",
                name: "Unified Search",
                scope: BindingScope::Search,
                primary_key: "Ctrl+Shift+F".into(),
                description: "Global ripgrep-style content search across project",
            },
            KeyActionBinding {
                id: "plugin_manager",
                name: "Plugin Center",
                scope: BindingScope::Workflow,
                primary_key: "Ctrl+Shift+X".into(),
                description: "Browse, inspect, and manage WebAssembly plugins",
            },
        ]
    }

    /// Filter bindings matching search query across action name, key, description, or scope.
    pub fn search_bindings(bindings: &[KeyActionBinding], query: &str) -> Vec<KeyActionBinding> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return bindings.to_vec();
        }

        bindings
            .iter()
            .filter(|b| {
                b.name.to_lowercase().contains(&q)
                    || b.primary_key.to_lowercase().contains(&q)
                    || b.description.to_lowercase().contains(&q)
                    || b.scope.as_str().to_lowercase().contains(&q)
            })
            .cloned()
            .collect()
    }

    /// Detect duplicate shortcuts and scope conflicts.
    pub fn check_conflicts(bindings: &[KeyActionBinding]) -> ConflictReport {
        let mut details = Vec::new();
        for (i, a) in bindings.iter().enumerate() {
            for b in bindings.iter().skip(i + 1) {
                // Check if identical key is bound in the same scope or overlapping global scope
                if a.primary_key == b.primary_key
                    && (a.scope == b.scope
                        || a.scope == BindingScope::Global
                        || b.scope == BindingScope::Global)
                {
                    details.push(format!(
                        "Conflict: '{}' is bound to both '{}' ({:?}) and '{}' ({:?})",
                        a.primary_key, a.name, a.scope, b.name, b.scope
                    ));
                }
            }
        }

        let has_conflicts = !details.is_empty();
        ConflictReport {
            has_conflicts,
            details,
        }
    }

    /// Format crossterm KeyEvent into a canonical string (e.g. "Ctrl+Shift+P").
    pub fn format_key_event(code: KeyCode, modifiers: KeyModifiers) -> String {
        let mut s = String::new();
        if modifiers.contains(KeyModifiers::CONTROL) {
            s.push_str("Ctrl+");
        }
        if modifiers.contains(KeyModifiers::ALT) {
            s.push_str("Alt+");
        }
        if modifiers.contains(KeyModifiers::SHIFT) {
            s.push_str("Shift+");
        }

        match code {
            KeyCode::Char(c) => s.push(c.to_ascii_uppercase()),
            KeyCode::F(n) => s.push_str(&format!("F{}", n)),
            KeyCode::Enter => s.push_str("Enter"),
            KeyCode::Esc => s.push_str("Esc"),
            KeyCode::Backspace => s.push_str("Backspace"),
            KeyCode::Tab => s.push_str("Tab"),
            KeyCode::Delete => s.push_str("Delete"),
            KeyCode::Left => s.push_str("Left"),
            KeyCode::Right => s.push_str("Right"),
            KeyCode::Up => s.push_str("Up"),
            KeyCode::Down => s.push_str("Down"),
            _ => s.push_str(&format!("{:?}", code)),
        }
        s
    }
}
