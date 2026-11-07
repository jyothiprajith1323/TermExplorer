//! Minimal LSP client stub.
//!
//! Detects available language servers and provides a placeholder for
//! future LSP integration. Currently reports which servers are present.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LspAvailability {
    pub rust_analyzer: Option<String>,
    pub clangd: Option<String>,
    pub pyright: Option<String>,
    pub typescript_ls: Option<String>,
}

#[allow(dead_code)]
impl LspAvailability {
    /// Probe PATH for known language servers.
    pub fn detect() -> Self {
        Self {
            rust_analyzer: which("rust-analyzer"),
            clangd: which("clangd"),
            pyright: which("pyright-langserver").or_else(|| which("pyright")),
            typescript_ls: which("typescript-language-server"),
        }
    }

    /// Return the server path for a given file extension, if available.
    pub fn server_for_ext(&self, ext: &str) -> Option<&str> {
        match ext {
            "rs" => self.rust_analyzer.as_deref(),
            "c" | "cpp" | "h" | "hpp" | "cxx" => self.clangd.as_deref(),
            "py" => self.pyright.as_deref(),
            "js" | "ts" | "jsx" | "tsx" => self.typescript_ls.as_deref(),
            _ => None,
        }
    }
}

#[allow(dead_code)]
fn which(name: &str) -> Option<String> {
    std::process::Command::new("which")
        .arg(name)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            String::from_utf8(o.stdout)
                .ok()
                .map(|s| s.trim().to_string())
        })
        .filter(|s| !s.is_empty())
}
