//! Terminal environment and capabilities detection.
//!
//! Inspects terminal dimensions, color fidelity, shell availability, PTY subsystem,
//! and development integrations with thread-safe caching.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

static CACHED_CAPABILITIES: RwLock<Option<TerminalCapabilities>> = RwLock::new(None);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShellInfo {
    pub name: String,
    pub path: PathBuf,
    pub is_default: bool,
    pub is_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolIntegration {
    pub name: String,
    pub purpose: String,
    pub path: Option<PathBuf>,
    pub is_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalCapabilities {
    pub term: String,
    pub colorterm: String,
    pub term_program: Option<String>,
    pub columns: u16,
    pub rows: u16,
    pub truecolor_supported: bool,
    pub pty_available: bool,
    pub nerd_font_detected: bool,
    pub available_shells: Vec<ShellInfo>,
    pub default_shell: ShellInfo,
    pub detected_integrations: Vec<ToolIntegration>,
    pub is_wsl2: bool,
}

impl Default for TerminalCapabilities {
    fn default() -> Self {
        Self::detect()
    }
}

impl TerminalCapabilities {
    /// Return the cached terminal capabilities, or detect if not yet cached.
    pub fn detect() -> Self {
        if let Ok(read) = CACHED_CAPABILITIES.read() {
            if let Some(caps) = &*read {
                return caps.clone();
            }
        }

        let detected = Self::detect_uncached();
        if let Ok(mut write) = CACHED_CAPABILITIES.write() {
            *write = Some(detected.clone());
        }
        detected
    }

    /// Force a fresh detection of terminal capabilities, invalidating the cache.
    pub fn redetect() -> Self {
        let detected = Self::detect_uncached();
        if let Ok(mut write) = CACHED_CAPABILITIES.write() {
            *write = Some(detected.clone());
        }
        detected
    }

    /// Perform uncached environment inspection.
    pub fn detect_uncached() -> Self {
        let term = std::env::var("TERM").unwrap_or_else(|_| "unknown".to_string());
        let colorterm = std::env::var("COLORTERM").unwrap_or_else(|_| "none".to_string());
        let term_program = std::env::var("TERM_PROGRAM").ok();

        // Terminal dimensions
        let (columns, rows) = crossterm::terminal::size().unwrap_or((120, 40));

        // Truecolor support
        let colorterm_lower = colorterm.to_lowercase();
        let truecolor_supported = colorterm_lower == "truecolor"
            || colorterm_lower == "24bit"
            || term.contains("direct")
            || term_program
                .as_deref()
                .map(|p| p == "WezTerm" || p == "iTerm.app" || p == "ghostty" || p == "alacritty")
                .unwrap_or(false);

        // PTY subsystem check
        #[cfg(unix)]
        let pty_available = Path::new("/dev/ptmx").exists() || nix_is_pty_available();
        #[cfg(not(unix))]
        let pty_available = false;

        // Nerd Font detection ($NERD_FONT or fontconfig)
        let nerd_font_env = std::env::var("NERD_FONT")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        let nerd_font_detected = nerd_font_env || {
            let font_status = crate::fonts::FontStatus::detect();
            font_status.jetbrains_nerd_font_installed
        };

        // Detect WSL2
        let is_wsl2 = detect_wsl2();

        // Detect available shells
        let default_shell_env = std::env::var("SHELL").ok();
        let standard_shells = [
            ("bash", "/bin/bash"),
            ("bash", "/usr/bin/bash"),
            ("zsh", "/usr/bin/zsh"),
            ("zsh", "/bin/zsh"),
            ("fish", "/usr/bin/fish"),
            ("fish", "/bin/fish"),
            ("sh", "/bin/sh"),
        ];

        let mut available_shells = Vec::new();
        let mut seen_paths = std::collections::HashSet::new();

        for (name, path_str) in standard_shells {
            let path = PathBuf::from(path_str);
            if path.exists() && seen_paths.insert(path.clone()) {
                let is_def = default_shell_env
                    .as_deref()
                    .map(|d| d == path_str)
                    .unwrap_or(false);
                available_shells.push(ShellInfo {
                    name: name.to_string(),
                    path,
                    is_default: is_def,
                    is_available: true,
                });
            }
        }

        // Determine default shell
        let default_shell = if let Some(env_sh) = &default_shell_env {
            let path = PathBuf::from(env_sh);
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("sh")
                .to_string();
            let is_avail = path.exists();
            ShellInfo {
                name,
                path,
                is_default: true,
                is_available: is_avail,
            }
        } else if let Some(first) = available_shells.first() {
            first.clone()
        } else {
            ShellInfo {
                name: "sh".to_string(),
                path: PathBuf::from("/bin/sh"),
                is_default: true,
                is_available: Path::new("/bin/sh").exists(),
            }
        };

        // Detect CLI integrations
        let detected_integrations = vec![
            check_tool("git", "Version Control & Branch Workbench"),
            check_tool("ffmpeg", "Audio/Video Keyframes & Waveforms"),
            check_tool("pdftoppm", "Vector PDF Document Rasterization"),
            check_tool("sqlite3", "Structured Database CLI Inspector"),
            check_tool("rust-analyzer", "Rust Language Server Protocol (LSP)"),
        ];

        Self {
            term,
            colorterm,
            term_program,
            columns,
            rows,
            truecolor_supported,
            pty_available,
            nerd_font_detected,
            available_shells,
            default_shell,
            detected_integrations,
            is_wsl2,
        }
    }

    /// Return a human-readable diagnostic status summary.
    pub fn diagnostic_summary(&self) -> String {
        format!(
            "TERM={} | COLORTERM={} | TrueColor={} | PTY={} | Shell={}",
            self.term,
            self.colorterm,
            if self.truecolor_supported {
                "Yes"
            } else {
                "No"
            },
            if self.pty_available {
                "Available"
            } else {
                "Unavailable"
            },
            self.default_shell.path.display()
        )
    }
}

fn check_tool(name: &str, purpose: &str) -> ToolIntegration {
    let path = which_in_path(name);
    let is_available = path.is_some();
    ToolIntegration {
        name: name.to_string(),
        purpose: purpose.to_string(),
        path,
        is_available,
    }
}

fn which_in_path(executable: &str) -> Option<PathBuf> {
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(executable);
            if candidate.is_file() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(meta) = candidate.metadata() {
                        if meta.permissions().mode() & 0o111 != 0 {
                            return Some(candidate);
                        }
                    }
                }
                #[cfg(not(unix))]
                return Some(candidate);
            }
        }
    }
    None
}

fn detect_wsl2() -> bool {
    if let Ok(version) = std::fs::read_to_string("/proc/version") {
        version.to_lowercase().contains("microsoft") || version.to_lowercase().contains("wsl")
    } else {
        false
    }
}

#[cfg(unix)]
fn nix_is_pty_available() -> bool {
    // Attempting to open /dev/ptmx or /dev/pts/ptmx
    Path::new("/dev/ptmx").exists() || Path::new("/dev/pts/ptmx").exists()
}
