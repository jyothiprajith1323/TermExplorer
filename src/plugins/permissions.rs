//! Plugin permission model and capability boundaries.

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginPermission {
    Commands,
    ContextActions,
    WorkspaceActions,
    FileMetadata,
    PreviewProvider,
    StatusText,
}

impl PluginPermission {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Commands => "commands",
            Self::ContextActions => "context_actions",
            Self::WorkspaceActions => "workspace_actions",
            Self::FileMetadata => "file_metadata",
            Self::PreviewProvider => "preview_provider",
            Self::StatusText => "status_text",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Commands => "Register custom actions and keybindings",
            Self::ContextActions => "Add items to file explorer right-click / context menu",
            Self::WorkspaceActions => "Inspect and trigger workspace hooks",
            Self::FileMetadata => "Read file attributes, extension, and path",
            Self::PreviewProvider => "Provide custom preview rendering for files",
            Self::StatusText => "Display telemetry and custom text in the status bar",
        }
    }
}

impl fmt::Display for PluginPermission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
