//! Font detection on Linux using fontconfig (`fc-list` and `fc-match`).

use std::path::PathBuf;
use std::process::Command;
use std::sync::RwLock;

static CACHED_STATUS: RwLock<Option<FontStatus>> = RwLock::new(None);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontStatus {
    pub jetbrains_mono_installed: bool,
    pub jetbrains_nerd_font_installed: bool,
    pub detected_family: Option<String>,
    pub detected_file: Option<PathBuf>,
    pub matching_entry: Option<String>,
}

impl Default for FontStatus {
    fn default() -> Self {
        Self::detect()
    }
}

impl FontStatus {
    /// Detect presence of JetBrains Mono and Nerd Font variants on the system (cached).
    pub fn detect() -> Self {
        if let Ok(read) = CACHED_STATUS.read() {
            if let Some(status) = &*read {
                return status.clone();
            }
        }

        let detected = Self::detect_uncached();
        if let Ok(mut write) = CACHED_STATUS.write() {
            *write = Some(detected.clone());
        }
        detected
    }

    /// Explicitly refresh font detection, invalidating cached state.
    pub fn redetect() -> Self {
        let detected = Self::detect_uncached();
        if let Ok(mut write) = CACHED_STATUS.write() {
            *write = Some(detected.clone());
        }
        detected
    }

    /// Uncached font detection running fc-list and fc-match.
    pub fn detect_uncached() -> Self {
        let (mono, nerd, family, file) = detect_via_fc();

        Self {
            jetbrains_mono_installed: mono,
            jetbrains_nerd_font_installed: nerd,
            detected_family: family,
            detected_file: file,
            matching_entry: check_fc_match(),
        }
    }

    /// Summary description for display in Settings or Command Palette.
    pub fn status_summary(&self) -> String {
        if self.jetbrains_nerd_font_installed {
            "JetBrainsMono Nerd Font installed & available".to_string()
        } else if self.jetbrains_mono_installed {
            "JetBrains Mono detected (standard, non-Nerd Font)".to_string()
        } else {
            "JetBrains Mono not detected on host system".to_string()
        }
    }
}

fn detect_via_fc() -> (bool, bool, Option<String>, Option<PathBuf>) {
    let output = match Command::new("fc-list")
        .args([":", "family", "file"])
        .output()
    {
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
        Err(_) => return (false, false, None, None),
    };

    let mut mono = false;
    let mut nerd = false;
    let mut family = None;
    let mut file = None;

    for line in output.lines() {
        let lower = line.to_lowercase();
        if lower.contains("jetbrains") {
            mono = true;
            if lower.contains("nerd") {
                nerd = true;
            }
            if family.is_none() {
                // fc-list outputs format: /path/to/font.ttf: Family Name
                let parts: Vec<&str> = line.split(':').collect();
                if let Some(f) = parts.first() {
                    file = Some(PathBuf::from(f.trim()));
                }
                if parts.len() > 1 {
                    family = Some(parts[1].trim().to_string());
                }
            }
        }
    }

    (mono, nerd, family, file)
}

fn check_fc_match() -> Option<String> {
    let output = Command::new("fc-match")
        .arg("JetBrains Mono")
        .output()
        .ok()?;
    let res = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if res.is_empty() {
        None
    } else {
        Some(res)
    }
}
