//! Filesystem browser — directory listing and metadata.

use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ── Sort order ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SortOrder {
    #[default]
    Name,
    NameDesc,
    Size,
    SizeDesc,
    Modified,
    ModifiedDesc,
    Kind,
}

impl SortOrder {
    pub fn cycle(self) -> Self {
        match self {
            SortOrder::Name => SortOrder::NameDesc,
            SortOrder::NameDesc => SortOrder::Size,
            SortOrder::Size => SortOrder::SizeDesc,
            SortOrder::SizeDesc => SortOrder::Modified,
            SortOrder::Modified => SortOrder::ModifiedDesc,
            SortOrder::ModifiedDesc => SortOrder::Kind,
            SortOrder::Kind => SortOrder::Name,
        }
    }

    pub fn next(self) -> Self {
        self.cycle()
    }

    pub fn label(self) -> &'static str {
        match self {
            SortOrder::Name => "Name ▲",
            SortOrder::NameDesc => "Name ▼",
            SortOrder::Size => "Size ▲",
            SortOrder::SizeDesc => "Size ▼",
            SortOrder::Modified => "Date ▲",
            SortOrder::ModifiedDesc => "Date ▼",
            SortOrder::Kind => "Type",
        }
    }

    #[allow(dead_code)]
    pub fn next_name(self) -> Self {
        match self {
            SortOrder::Name => SortOrder::NameDesc,
            SortOrder::NameDesc => SortOrder::Name,
            other => match other {
                SortOrder::Size | SortOrder::SizeDesc => SortOrder::Name,
                _ => SortOrder::Name,
            },
        }
    }
}

// ── Entry type ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryKind {
    Directory,
    File,
    Symlink {
        target: Option<PathBuf>,
        broken: bool,
    },
    Special, // block/char device, pipe, socket …
}

// ── Single directory entry ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DirEntry {
    /// File/directory name (not full path).
    pub name: String,
    pub kind: EntryKind,
    /// File size in bytes; 0 for directories.
    pub size: u64,
    pub modified: Option<DateTime<Local>>,
    pub is_hidden: bool,
    pub extension: Option<String>,
    pub permissions_str: String,
}

impl DirEntry {
    pub fn icon(&self) -> &'static str {
        match &self.kind {
            EntryKind::Directory => "📁",
            EntryKind::Symlink { broken: true, .. } => "🔗",
            EntryKind::Symlink { .. } => "🔗",
            EntryKind::Special => "⚙ ",
            EntryKind::File => file_icon(self.extension.as_deref()),
        }
    }

    /// Return icon string and color based on icon pack and Catppuccin palette.
    pub fn icon_styled(
        &self,
        pack: crate::icons::IconPack,
        palette: &crate::theme::CatppuccinPalette,
    ) -> (String, ratatui::style::Color) {
        match &self.kind {
            EntryKind::Directory => {
                crate::icons::mappings::resolve_folder_icon(&self.name, false, pack, palette)
            }
            EntryKind::Symlink { broken, .. } => {
                let symbol = match pack {
                    crate::icons::IconPack::Catppuccin | crate::icons::IconPack::NerdFont => {
                        crate::icons::glyphs::FILE_SYMLINK.nerd
                    }
                    crate::icons::IconPack::Unicode => crate::icons::glyphs::FILE_SYMLINK.unicode,
                    crate::icons::IconPack::Ascii => crate::icons::glyphs::FILE_SYMLINK.ascii,
                    crate::icons::IconPack::None => "",
                };
                let col = if *broken { palette.red } else { palette.teal };
                (format!("{} ", symbol.trim()), col)
            }
            EntryKind::Special => {
                let symbol = match pack {
                    crate::icons::IconPack::Catppuccin | crate::icons::IconPack::NerdFont => {
                        crate::icons::glyphs::FILE_SPECIAL.nerd
                    }
                    crate::icons::IconPack::Unicode => crate::icons::glyphs::FILE_SPECIAL.unicode,
                    crate::icons::IconPack::Ascii => crate::icons::glyphs::FILE_SPECIAL.ascii,
                    crate::icons::IconPack::None => "",
                };
                (format!("{} ", symbol.trim()), palette.yellow)
            }
            EntryKind::File => crate::icons::mappings::resolve_file_icon(
                &self.name,
                self.extension.as_deref(),
                pack,
                palette,
            ),
        }
    }

    pub fn is_dir(&self) -> bool {
        matches!(self.kind, EntryKind::Directory)
    }
}

fn file_icon(ext: Option<&str>) -> &'static str {
    match ext.unwrap_or("").to_lowercase().as_str() {
        "rs" => "🦀",
        "toml" | "yaml" | "yml" | "json" => "⚙ ",
        "md" | "txt" | "rst" => "📄",
        "py" => "🐍",
        "js" | "ts" => "📜",
        "html" | "css" => "🌐",
        "c" | "cpp" | "h" | "hpp" => "⚡",
        "java" => "☕",
        "sh" | "bash" | "zsh" => "🖥 ",
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" => "🖼 ",
        "zip" | "tar" | "gz" | "xz" | "bz2" | "7z" => "📦",
        "pdf" => "📕",
        "mp3" | "wav" | "flac" | "ogg" => "🎵",
        "mp4" | "mkv" | "avi" | "mov" => "🎬",
        "log" => "📋",
        "lock" => "🔒",
        "xml" => "📰",
        "sql" => "🗃 ",
        _ => "📄",
    }
}

// ── Directory listing ──────────────────────────────────────────────────────

/// Read a directory and return a sorted list of entries.
/// Hidden files (name starts with '.') are included based on `show_hidden`.
pub fn list_dir(path: &Path, show_hidden: bool, sort: SortOrder) -> Result<Vec<DirEntry>> {
    let read_dir = std::fs::read_dir(path)
        .with_context(|| format!("Reading directory: {}", path.display()))?;

    let mut entries = Vec::new();

    for item in read_dir {
        let item = match item {
            Ok(i) => i,
            Err(_) => continue, // skip unreadable entries
        };

        let name = item.file_name().to_string_lossy().to_string();
        let is_hidden = name.starts_with('.');

        if is_hidden && !show_hidden {
            continue;
        }

        let metadata = match item.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let (kind, size) = if metadata.is_symlink() {
            let target = std::fs::read_link(item.path()).ok();
            let broken = target.as_ref().map(|t| !t.exists()).unwrap_or(true);
            (EntryKind::Symlink { target, broken }, 0u64)
        } else if metadata.is_dir() {
            (EntryKind::Directory, 0u64)
        } else if metadata.is_file() {
            (EntryKind::File, metadata.len())
        } else {
            (EntryKind::Special, 0u64)
        };

        let modified = metadata.modified().ok().map(DateTime::<Local>::from);

        let extension = std::path::Path::new(&name)
            .extension()
            .map(|e| e.to_string_lossy().to_string());

        let permissions_str = format_permissions(&metadata);

        entries.push(DirEntry {
            name,
            kind,
            size,
            modified,
            is_hidden,
            extension,
            permissions_str,
        });
    }

    sort_entries(&mut entries, sort);
    Ok(entries)
}

fn sort_entries(entries: &mut [DirEntry], sort: SortOrder) {
    // Directories always come first regardless of sort key.
    entries.sort_by(|a, b| {
        let dir_a = a.is_dir();
        let dir_b = b.is_dir();
        if dir_a != dir_b {
            return dir_b.cmp(&dir_a); // dirs first
        }
        match sort {
            SortOrder::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortOrder::NameDesc => b.name.to_lowercase().cmp(&a.name.to_lowercase()),
            SortOrder::Size => a.size.cmp(&b.size),
            SortOrder::SizeDesc => b.size.cmp(&a.size),
            SortOrder::Modified => a.modified.cmp(&b.modified),
            SortOrder::ModifiedDesc => b.modified.cmp(&a.modified),
            SortOrder::Kind => {
                let ext_a = a.extension.as_deref().unwrap_or("");
                let ext_b = b.extension.as_deref().unwrap_or("");
                ext_a.cmp(ext_b)
            }
        }
    });
}

#[cfg(unix)]
fn format_permissions(meta: &std::fs::Metadata) -> String {
    use std::os::unix::fs::PermissionsExt;
    let mode = meta.permissions().mode();
    let chars: Vec<char> = "rwxrwxrwx".chars().collect();
    let bits = [
        0o400, 0o200, 0o100, 0o040, 0o020, 0o010, 0o004, 0o002, 0o001,
    ];
    let perm: String = bits
        .iter()
        .zip(chars.iter())
        .map(|(&bit, &c)| if mode & bit != 0 { c } else { '-' })
        .collect();
    perm
}

#[cfg(not(unix))]
fn format_permissions(_meta: &std::fs::Metadata) -> String {
    String::from("---------")
}
