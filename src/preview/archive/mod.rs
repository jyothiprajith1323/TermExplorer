//! Archive and compressed file preview (ZIP, TAR, TAR.GZ, TGZ, 7Z).

use flate2::read::GzDecoder;
use std::fs::File;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ArchiveEntryInfo {
    pub name: String,
    pub is_dir: bool,
    pub uncompressed_size: u64,
    pub compressed_size: Option<u64>,
    pub is_safe: bool, // Traversal protection check
}

#[derive(Debug, Clone)]
pub struct ArchivePreview {
    pub archive_type: &'static str,
    pub entries: Vec<ArchiveEntryInfo>,
    pub total_entries: usize,
    pub total_uncompressed_bytes: u64,
    pub has_unsafe_entries: bool,
}

impl ArchivePreview {
    pub fn load(path: &Path, max_entries: usize) -> Self {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        if ext == "zip" {
            Self::load_zip(path, max_entries)
        } else if ext == "tar" {
            Self::load_tar(path, false, max_entries)
        } else if ext == "gz" || ext == "tgz" {
            Self::load_tar(path, true, max_entries)
        } else {
            Self::load_zip(path, max_entries)
        }
    }

    fn load_zip(path: &Path, max_entries: usize) -> Self {
        let Ok(file) = File::open(path) else {
            return Self::error("Could not open ZIP file");
        };

        let Ok(mut archive) = zip::ZipArchive::new(file) else {
            return Self::error("Invalid or corrupted ZIP archive");
        };

        let total_entries = archive.len();
        let mut entries = Vec::new();
        let mut total_uncompressed_bytes = 0u64;
        let mut has_unsafe_entries = false;

        for i in 0..total_entries {
            if let Ok(entry) = archive.by_index(i) {
                let name = match entry.name() {
                    Ok(n) => n.to_string(),
                    Err(_) => continue,
                };
                let uncompressed_size = entry.size();
                let compressed_size = Some(entry.compressed_size());
                let is_dir = entry.is_dir();
                let is_safe = is_safe_archive_path(&name);

                if !is_safe {
                    has_unsafe_entries = true;
                }

                total_uncompressed_bytes += uncompressed_size;

                if entries.len() < max_entries {
                    entries.push(ArchiveEntryInfo {
                        name,
                        is_dir,
                        uncompressed_size,
                        compressed_size,
                        is_safe,
                    });
                }
            }
        }

        Self {
            archive_type: "ZIP Archive",
            entries,
            total_entries,
            total_uncompressed_bytes,
            has_unsafe_entries,
        }
    }

    fn load_tar(path: &Path, is_gzipped: bool, max_entries: usize) -> Self {
        let Ok(file) = File::open(path) else {
            return Self::error("Could not open Tarball archive");
        };

        let mut entries = Vec::new();
        let mut total_entries = 0;
        let mut total_uncompressed_bytes = 0u64;
        let mut has_unsafe_entries = false;

        if is_gzipped {
            let gz = GzDecoder::new(file);
            let mut archive = tar::Archive::new(gz);
            if let Ok(tar_entries) = archive.entries() {
                for entry_res in tar_entries {
                    total_entries += 1;
                    if let Ok(entry) = entry_res {
                        let name = entry
                            .path()
                            .map(|p| p.display().to_string())
                            .unwrap_or_default();
                        let uncompressed_size = entry.size();
                        let is_dir = entry.header().entry_type().is_dir();
                        let is_safe = is_safe_archive_path(&name);

                        if !is_safe {
                            has_unsafe_entries = true;
                        }
                        total_uncompressed_bytes += uncompressed_size;

                        if entries.len() < max_entries {
                            entries.push(ArchiveEntryInfo {
                                name,
                                is_dir,
                                uncompressed_size,
                                compressed_size: None,
                                is_safe,
                            });
                        }
                    }
                }
            }
        } else {
            let mut archive = tar::Archive::new(file);
            if let Ok(tar_entries) = archive.entries() {
                for entry_res in tar_entries {
                    total_entries += 1;
                    if let Ok(entry) = entry_res {
                        let name = entry
                            .path()
                            .map(|p| p.display().to_string())
                            .unwrap_or_default();
                        let uncompressed_size = entry.size();
                        let is_dir = entry.header().entry_type().is_dir();
                        let is_safe = is_safe_archive_path(&name);

                        if !is_safe {
                            has_unsafe_entries = true;
                        }
                        total_uncompressed_bytes += uncompressed_size;

                        if entries.len() < max_entries {
                            entries.push(ArchiveEntryInfo {
                                name,
                                is_dir,
                                uncompressed_size,
                                compressed_size: None,
                                is_safe,
                            });
                        }
                    }
                }
            }
        }

        Self {
            archive_type: if is_gzipped {
                "Gzip Tarball (TAR.GZ)"
            } else {
                "Tape Archive (TAR)"
            },
            entries,
            total_entries,
            total_uncompressed_bytes,
            has_unsafe_entries,
        }
    }

    /// Safely extract the archive to a destination directory preventing path traversal.
    pub fn extract_all(&self, archive_path: &Path, target_dir: &Path) -> Result<usize, String> {
        let ext = archive_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        std::fs::create_dir_all(target_dir).map_err(|e| e.to_string())?;

        if ext == "zip" {
            let file = File::open(archive_path).map_err(|e| e.to_string())?;
            let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
            let mut extracted = 0;
            for i in 0..zip.len() {
                let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
                let outpath = match entry.enclosed_name() {
                    Some(path) => target_dir.join(path),
                    None => continue, // Skip unsafe path
                };
                if entry.is_dir() {
                    std::fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
                } else {
                    if let Some(p) = outpath.parent() {
                        std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
                    }
                    let mut outfile = File::create(&outpath).map_err(|e| e.to_string())?;
                    std::io::copy(&mut entry, &mut outfile).map_err(|e| e.to_string())?;
                    extracted += 1;
                }
            }
            Ok(extracted)
        } else {
            Err("Extract currently supported for ZIP archives natively".to_string())
        }
    }

    fn error(msg: &'static str) -> Self {
        Self {
            archive_type: "Archive",
            entries: vec![ArchiveEntryInfo {
                name: format!("[Error: {}]", msg),
                is_dir: false,
                uncompressed_size: 0,
                compressed_size: None,
                is_safe: true,
            }],
            total_entries: 0,
            total_uncompressed_bytes: 0,
            has_unsafe_entries: false,
        }
    }
}

/// Protect against Zip Slip and path traversal vulnerabilities.
pub fn is_safe_archive_path(path_str: &str) -> bool {
    let path = PathBuf::from(path_str);
    if path.is_absolute() {
        return false;
    }
    for comp in path.components() {
        if let std::path::Component::ParentDir = comp {
            return false;
        }
    }
    true
}
