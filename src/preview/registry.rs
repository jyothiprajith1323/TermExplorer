//! Capability registry for preview handlers and external helper tools.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandlerCapability {
    /// Handler is fully built-in using pure Rust.
    Builtin,
    /// Handler utilizes an optional external CLI tool when available, with a built-in fallback.
    OptionalTool {
        tool_name: &'static str,
        installed: bool,
        purpose: &'static str,
    },
    /// Format requires an external viewer or application.
    ExternalOnly {
        app_name: &'static str,
        installed: bool,
    },
}

#[derive(Debug, Clone)]
pub struct ToolStatus {
    pub name: &'static str,
    pub installed: bool,
    pub path: Option<String>,
    pub purpose: &'static str,
}

pub struct CapabilityRegistry;

static TOOL_CACHE: OnceLock<RwLock<HashMap<String, Option<String>>>> = OnceLock::new();

impl CapabilityRegistry {
    /// Check whether a command exists in $PATH using cached in-memory resolution.
    pub fn is_tool_available(name: &str) -> bool {
        Self::tool_path(name).is_some()
    }

    /// Resolve the full path of a tool without spawning slow subprocesses.
    pub fn tool_path(name: &str) -> Option<String> {
        let cache = TOOL_CACHE.get_or_init(|| RwLock::new(HashMap::new()));
        if let Ok(guard) = cache.read() {
            if let Some(hit) = guard.get(name) {
                return hit.clone();
            }
        }

        // Fast in-process PATH lookup — zero subprocess overhead
        let resolved = if let Ok(path_var) = std::env::var("PATH") {
            std::env::split_paths(&path_var).find_map(|dir| {
                let candidate = dir.join(name);
                if candidate.is_file() {
                    Some(candidate.to_string_lossy().to_string())
                } else {
                    None
                }
            })
        } else {
            None
        };

        if let Ok(mut guard) = cache.write() {
            guard.insert(name.to_string(), resolved.clone());
        }

        resolved
    }

    /// List status of all known optional integrations.
    pub fn list_integrations() -> Vec<ToolStatus> {
        let tools: &[(&'static str, &'static str)] = &[
            ("git", "Git version control integration"),
            ("gio", "FreeDesktop Trash integration via gio trash"),
            ("trash-put", "Command-line trash utility"),
            ("ffmpeg", "Video frame extraction and thumbnail rendering"),
            (
                "ffprobe",
                "Detailed video and audio container stream analysis",
            ),
            ("mpv", "Lightweight multimedia audio and video player"),
            ("pdftoppm", "Poppler PDF rasterization to page images"),
            ("pdftotext", "Poppler PDF text stream extraction"),
            ("sqlite3", "SQLite database CLI tool"),
            (
                "rust-analyzer",
                "Rust Language Server Protocol (LSP) server",
            ),
            ("pyright", "Python LSP server"),
            ("clangd", "C/C++ LSP server"),
            ("libreoffice", "Headless office document conversion"),
            ("pandoc", "Universal document converter"),
            ("xdg-open", "Standard Linux desktop application launcher"),
        ];

        tools
            .iter()
            .map(|&(name, purpose)| {
                let path = Self::tool_path(name);
                let installed = path.is_some();
                ToolStatus {
                    name,
                    installed,
                    path,
                    purpose,
                }
            })
            .collect()
    }
}
