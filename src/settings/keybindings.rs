//! Keybinding management with scope separation and conflict detection.

use crossterm::event::{KeyCode, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingScope {
    Global,
    Explorer,
    Editor,
    Terminal,
    Preview,
    Git,
}

#[derive(Debug, Clone)]
pub struct KeyActionBinding {
    pub id: &'static str,
    pub name: &'static str,
    pub scope: BindingScope,
    pub primary_key: String,
    pub description: &'static str,
}

#[derive(Debug, Clone)]
pub struct ConflictReport {
    pub has_conflicts: bool,
    pub details: Vec<String>,
}

pub struct KeybindingManager;

impl KeybindingManager {
    /// Return all default keybindings categorized by scope.
    pub fn default_bindings() -> Vec<KeyActionBinding> {
        vec![
            // Global
            KeyActionBinding {
                id: "help",
                name: "Show Help",
                scope: BindingScope::Global,
                primary_key: "F1 / ?".into(),
                description: "Display keyboard shortcuts overlay",
            },
            KeyActionBinding {
                id: "edit",
                name: "Open in Editor",
                scope: BindingScope::Global,
                primary_key: "F2".into(),
                description: "Open selected file in lightweight text editor",
            },
            KeyActionBinding {
                id: "preview",
                name: "Toggle Preview Zoom",
                scope: BindingScope::Global,
                primary_key: "F3".into(),
                description: "Maximize or toggle right preview panel",
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
                description: "Show/hide bottom system resource monitor",
            },
            KeyActionBinding {
                id: "dual_pane",
                name: "Toggle Dual-Pane",
                scope: BindingScope::Global,
                primary_key: "F7".into(),
                description: "Toggle side-by-side dual explorer panes",
            },
            KeyActionBinding {
                id: "settings",
                name: "Settings Studio",
                scope: BindingScope::Global,
                primary_key: "F10".into(),
                description: "Open interactive settings application",
            },
            KeyActionBinding {
                id: "cmd_palette",
                name: "Command Palette",
                scope: BindingScope::Global,
                primary_key: "Ctrl+Shift+P".into(),
                description: "Fuzzy-search commands, themes, and files",
            },
            KeyActionBinding {
                id: "quit",
                name: "Quit Application",
                scope: BindingScope::Global,
                primary_key: "Ctrl+Q / q".into(),
                description: "Cleanly restore terminal and exit",
            },
            // Explorer
            KeyActionBinding {
                id: "nav_down",
                name: "Move Down",
                scope: BindingScope::Explorer,
                primary_key: "j / Down".into(),
                description: "Highlight next item",
            },
            KeyActionBinding {
                id: "nav_up",
                name: "Move Up",
                scope: BindingScope::Explorer,
                primary_key: "k / Up".into(),
                description: "Highlight previous item",
            },
            KeyActionBinding {
                id: "open",
                name: "Enter Directory",
                scope: BindingScope::Explorer,
                primary_key: "Enter / l".into(),
                description: "Open selected directory or preview file",
            },
            KeyActionBinding {
                id: "parent",
                name: "Parent Directory",
                scope: BindingScope::Explorer,
                primary_key: "Backspace / h".into(),
                description: "Navigate to parent folder",
            },
            KeyActionBinding {
                id: "mark",
                name: "Mark / Select File",
                scope: BindingScope::Explorer,
                primary_key: "Space".into(),
                description: "Toggle visual selection for batch operations",
            },
            KeyActionBinding {
                id: "new_file",
                name: "New File",
                scope: BindingScope::Explorer,
                primary_key: "a".into(),
                description: "Create new empty file",
            },
            KeyActionBinding {
                id: "new_dir",
                name: "New Directory",
                scope: BindingScope::Explorer,
                primary_key: "A".into(),
                description: "Create new subdirectory",
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
                name: "Delete (Trash / Safe)",
                scope: BindingScope::Explorer,
                primary_key: "d / Delete".into(),
                description: "Send to trash or delete with prompt",
            },
            KeyActionBinding {
                id: "copy",
                name: "Copy",
                scope: BindingScope::Explorer,
                primary_key: "y".into(),
                description: "Copy selected entry to clipboard",
            },
            KeyActionBinding {
                id: "cut",
                name: "Cut",
                scope: BindingScope::Explorer,
                primary_key: "x".into(),
                description: "Cut selected entry for moving",
            },
            KeyActionBinding {
                id: "paste",
                name: "Paste",
                scope: BindingScope::Explorer,
                primary_key: "p".into(),
                description: "Paste entry into current directory",
            },
            KeyActionBinding {
                id: "search",
                name: "Fuzzy Filter",
                scope: BindingScope::Explorer,
                primary_key: "/".into(),
                description: "Filter files in current directory",
            },
            KeyActionBinding {
                id: "hidden",
                name: "Toggle Hidden Files",
                scope: BindingScope::Explorer,
                primary_key: ".".into(),
                description: "Show or hide dotfiles",
            },
            KeyActionBinding {
                id: "sort",
                name: "Cycle Sort Order",
                scope: BindingScope::Explorer,
                primary_key: "s".into(),
                description: "Sort by Name, Size, Date, or Ext",
            },
            // Editor
            KeyActionBinding {
                id: "save",
                name: "Atomic Save",
                scope: BindingScope::Editor,
                primary_key: "Ctrl+S".into(),
                description: "Atomically persist buffer to disk",
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
                description: "Insert selected word completion",
            },
            KeyActionBinding {
                id: "close_editor",
                name: "Close Editor",
                scope: BindingScope::Editor,
                primary_key: "Esc".into(),
                description: "Return to file explorer view",
            },
            // Terminal
            KeyActionBinding {
                id: "term_toggle",
                name: "Toggle Terminal",
                scope: BindingScope::Terminal,
                primary_key: "Ctrl+T".into(),
                description: "Show or hide embedded PTY terminal",
            },
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
                description: "Send end-of-file to shell",
            },
            // Preview
            KeyActionBinding {
                id: "prev_tab",
                name: "Previous Preview Tab",
                scope: BindingScope::Preview,
                primary_key: "[ / Left".into(),
                description: "Switch to previous tab (Preview/Info/Hex/Actions)",
            },
            KeyActionBinding {
                id: "next_tab",
                name: "Next Preview Tab",
                scope: BindingScope::Preview,
                primary_key: "] / Right".into(),
                description: "Switch to next tab",
            },
        ]
    }

    /// Detect duplicate shortcuts and scope conflicts.
    pub fn check_conflicts(bindings: &[KeyActionBinding]) -> ConflictReport {
        let mut details = Vec::new();
        for (i, a) in bindings.iter().enumerate() {
            for b in bindings.iter().skip(i + 1) {
                // If same primary key in same or overlapping scope
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
