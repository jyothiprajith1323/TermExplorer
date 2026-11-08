//! Filesystem operations — safe copy, cross-filesystem move, delete, trash, and atomic save.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

/// Collision handling policies when creating or moving entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CollisionPolicy {
    #[default]
    Fail,
    Overwrite,
    AutoRename,
}

/// Generate a non-colliding destination path: `file (1).txt`, `file (2).txt`, etc.
pub fn auto_rename_target(dst: &Path) -> PathBuf {
    if !dst.exists() {
        return dst.to_path_buf();
    }
    let parent = dst.parent().unwrap_or_else(|| Path::new(""));
    let stem = dst.file_stem().and_then(|s| s.to_str()).unwrap_or("entry");
    let ext = dst.extension().and_then(|s| s.to_str());

    for counter in 1..=10_000 {
        let new_name = match ext {
            Some(extension) => format!("{} ({}).{}", stem, counter, extension),
            None => format!("{} ({})", stem, counter),
        };
        let candidate = parent.join(new_name);
        if !candidate.exists() {
            return candidate;
        }
    }
    // Fallback if loop exhausted
    parent.join(format!(
        "{}_{}",
        stem,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    ))
}

/// Create a new empty file, failing if the file already exists.
pub fn create_file(path: &Path) -> Result<()> {
    if path.exists() {
        bail!("File already exists: {}", path.display());
    }
    let parent = path.parent().unwrap_or(path);
    std::fs::create_dir_all(parent)
        .with_context(|| format!("Creating parent directories: {}", parent.display()))?;
    std::fs::File::create(path).with_context(|| format!("Creating file: {}", path.display()))?;
    Ok(())
}

/// Create a directory (and any missing parents).
pub fn create_dir(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path).with_context(|| format!("Creating directory: {}", path.display()))
}

/// Check if an error represents a cross-device / cross-filesystem link error (EXDEV = 18 on Linux).
fn is_cross_device_error(err: &std::io::Error) -> bool {
    err.raw_os_error() == Some(18) || err.kind() == std::io::ErrorKind::CrossesDevices
}

/// Move a filesystem entry, supporting cross-filesystem boundary operations with verification and rollback.
pub fn move_entry(from: &Path, to: &Path, policy: CollisionPolicy) -> Result<PathBuf> {
    if !from.exists() {
        bail!("Source does not exist: {}", from.display());
    }

    let mut target = to.to_path_buf();
    if target.exists() {
        match policy {
            CollisionPolicy::Fail => {
                bail!("Destination already exists: {}", target.display());
            }
            CollisionPolicy::AutoRename => {
                target = auto_rename_target(&target);
            }
            CollisionPolicy::Overwrite => {
                // If destination is a directory, remove recursively; else remove file
                if target.is_dir() && !target.is_symlink() {
                    delete_dir(&target)?;
                } else {
                    delete_file(&target)?;
                }
            }
        }
    }

    // Ensure destination parent directory exists
    if let Some(parent) = target.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Creating destination directory: {}", parent.display()))?;
        }
    }

    // First attempt atomic rename
    match std::fs::rename(from, &target) {
        Ok(_) => Ok(target),
        Err(err) if is_cross_device_error(&err) => {
            // Cross-filesystem move: copy-then-verify-then-remove with rollback
            copy_entry(from, &target).with_context(|| {
                format!(
                    "Cross-device copy {} → {}",
                    from.display(),
                    target.display()
                )
            })?;

            // Verification phase
            let verified = if from.is_file() {
                let src_meta = std::fs::metadata(from)?;
                let dst_meta = std::fs::metadata(&target)?;
                src_meta.len() == dst_meta.len()
            } else {
                target.exists()
            };

            if !verified {
                // Rollback: delete incomplete destination
                let _ = delete_entry(&target);
                bail!(
                    "Cross-device move verification failed for {}. Destination rolled back.",
                    target.display()
                );
            }

            // Verification passed: remove source
            delete_entry(from).with_context(|| {
                format!(
                    "Cleaning source after cross-device move: {}",
                    from.display()
                )
            })?;

            Ok(target)
        }
        Err(err) => {
            Err(err).with_context(|| format!("Moving {} → {}", from.display(), target.display()))
        }
    }
}

/// Rename or move a filesystem entry with default strict failure if destination exists.
pub fn rename_entry(from: &Path, to: &Path) -> Result<()> {
    move_entry(from, to, CollisionPolicy::Fail).map(|_| ())
}

/// Delete a file (not a directory).
pub fn delete_file(path: &Path) -> Result<()> {
    std::fs::remove_file(path).with_context(|| format!("Deleting file: {}", path.display()))
}

/// Recursively delete a directory.
pub fn delete_dir(path: &Path) -> Result<()> {
    std::fs::remove_dir_all(path).with_context(|| format!("Deleting directory: {}", path.display()))
}

/// Delete a file or directory safely (does not follow symlinks).
pub fn delete_entry(path: &Path) -> Result<()> {
    if path.is_dir() && !path.is_symlink() {
        delete_dir(path)
    } else {
        delete_file(path)
    }
}

/// Send entry to FreeDesktop trash using standard `~/.local/share/Trash` with `.trashinfo` metadata,
/// returning the unique trash filename for ledger tracking and reversible restore.
pub fn trash_entry_tracked(path: &Path) -> Result<String> {
    if !path.exists() {
        bail!("Entry to trash does not exist: {}", path.display());
    }

    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Home directory not found"))?;
    let trash_files = home.join(".local/share/Trash/files");
    let trash_info = home.join(".local/share/Trash/info");
    std::fs::create_dir_all(&trash_files)?;
    std::fs::create_dir_all(&trash_info)?;

    let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("entry");

    let dest_file = auto_rename_target(&trash_files.join(file_name));
    let dest_name = dest_file
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(file_name)
        .to_string();

    let info_path = trash_info.join(format!("{}.trashinfo", dest_name));

    // Format ISO 8601 deletion timestamp
    let now = chrono_now_iso();
    let abs_path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let info_content = format!(
        "[Trash Info]\nPath={}\nDeletionDate={}\n",
        abs_path.display(),
        now
    );

    // Write .trashinfo first
    std::fs::write(&info_path, info_content)
        .with_context(|| format!("Writing trashinfo: {}", info_path.display()))?;

    // Move entry to trash files directory
    if let Err(e) = move_entry(path, &dest_file, CollisionPolicy::Overwrite) {
        let _ = std::fs::remove_file(&info_path);
        return Err(e);
    }

    Ok(dest_name)
}

/// Send entry to FreeDesktop trash.
pub fn trash_entry(path: &Path) -> Result<()> {
    trash_entry_tracked(path).map(|_| ())
}

/// Helper to generate ISO 8601 UTC timestamp without heavyweight external chrono dep
fn chrono_now_iso() -> String {
    use std::time::SystemTime;
    let dur = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();

    // Standard civil days conversion
    let days = (secs / 86400) as i64;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    let (year, month, day) = civil_from_days(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
        year, month, day, hours, minutes, seconds
    )
}

fn civil_from_days(days: i64) -> (i64, u8, u8) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as u8, d as u8)
}

/// Restore a trashed entry back to its original location using the FreeDesktop standard.
pub fn restore_from_trash(trash_filename: &str) -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Home directory not found"))?;
    let trash_files = home.join(".local/share/Trash/files");
    let trash_info = home.join(".local/share/Trash/info");

    let trashed_file = trash_files.join(trash_filename);
    let info_path = trash_info.join(format!("{}.trashinfo", trash_filename));

    if !trashed_file.exists() {
        bail!("Trashed file does not exist: {}", trashed_file.display());
    }
    if !info_path.exists() {
        bail!(
            "Trash info metadata does not exist: {}",
            info_path.display()
        );
    }

    let content = std::fs::read_to_string(&info_path)
        .with_context(|| format!("Reading trashinfo: {}", info_path.display()))?;

    let mut original_path: Option<PathBuf> = None;
    for line in content.lines() {
        if let Some(stripped) = line.strip_prefix("Path=") {
            original_path = Some(PathBuf::from(stripped.trim()));
            break;
        }
    }

    let original =
        original_path.ok_or_else(|| anyhow::anyhow!("Path key missing in .trashinfo"))?;
    let restored = move_entry(&trashed_file, &original, CollisionPolicy::AutoRename)?;
    let _ = std::fs::remove_file(&info_path);

    Ok(restored)
}

/// Copy a single file, preserving permissions.
pub fn copy_file(src: &Path, dst: &Path) -> Result<u64> {
    let bytes = std::fs::copy(src, dst)
        .with_context(|| format!("Copying {} → {}", src.display(), dst.display()))?;
    Ok(bytes)
}

/// Copy a file or directory recursively.
pub fn copy_entry(src: &Path, dst: &Path) -> Result<()> {
    if src.is_dir() && !src.is_symlink() {
        std::fs::create_dir_all(dst)
            .with_context(|| format!("Creating directory: {}", dst.display()))?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let entry_dst = dst.join(entry.file_name());
            copy_entry(&entry.path(), &entry_dst)?;
        }
        Ok(())
    } else {
        copy_file(src, dst).map(|_| ())
    }
}

/// Save file contents atomically (write to temp then rename, preserving original permissions).
pub fn save_file_atomic(path: &Path, content: &[u8]) -> Result<()> {
    use std::io::Write;
    let parent = path.parent().unwrap_or(path);

    // Read permissions of existing file if present
    let existing_perms = std::fs::metadata(path).ok().map(|m| m.permissions());

    let mut tmp = tempfile::NamedTempFile::new_in(parent)
        .with_context(|| format!("Creating temporary file in: {}", parent.display()))?;

    tmp.write_all(content)
        .context("Writing content to temporary file")?;
    tmp.flush().context("Flushing temporary file")?;
    tmp.as_file()
        .sync_all()
        .context("Synchronizing temporary file data to storage")?;

    // Restore permissions if previously set
    if let Some(perms) = existing_perms {
        let _ = std::fs::set_permissions(tmp.path(), perms);
    }

    tmp.persist(path)
        .with_context(|| format!("Persisting temporary file → {}", path.display()))?;

    // Synchronize parent directory to commit rename entry to filesystem metadata
    if let Ok(dir_file) = std::fs::File::open(parent) {
        let _ = dir_file.sync_all();
    }

    Ok(())
}
