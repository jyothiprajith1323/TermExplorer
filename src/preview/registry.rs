//! Capability registry for preview handlers and external helper tools.

use std::process::Command;

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

impl CapabilityRegistry {
    /// Check whether a command exists in $PATH.
    pub fn is_tool_available(name: &str) -> bool {
        Command::new("which")
            .arg(name)
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }

    /// Resolve the full path of a tool if available.
    pub fn tool_path(name: &str) -> Option<String> {
        let out = Command::new("which").arg(name).output().ok()?;
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                return Some(s);
            }
        }
        None
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
