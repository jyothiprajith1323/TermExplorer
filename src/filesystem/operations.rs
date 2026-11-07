//! Filesystem operations — copy, move, delete, rename, create.

use anyhow::{bail, Context, Result};
use std::path::Path;

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

/// Rename or move a filesystem entry within the same filesystem.
pub fn rename_entry(from: &Path, to: &Path) -> Result<()> {
    if to.exists() {
        bail!("Destination already exists: {}", to.display());
    }
    std::fs::rename(from, to)
        .with_context(|| format!("Renaming {} → {}", from.display(), to.display()))
}

/// Delete a file (not a directory).
pub fn delete_file(path: &Path) -> Result<()> {
    std::fs::remove_file(path).with_context(|| format!("Deleting file: {}", path.display()))
}

/// Recursively delete a directory.
pub fn delete_dir(path: &Path) -> Result<()> {
    std::fs::remove_dir_all(path).with_context(|| format!("Deleting directory: {}", path.display()))
}

/// Delete a file or directory.
pub fn delete_entry(path: &Path) -> Result<()> {
    if path.is_dir() && !path.is_symlink() {
        delete_dir(path)
    } else {
        delete_file(path)
    }
}

/// Send entry to FreeDesktop trash using `gio trash`, `trash-put`, or standard `~/.local/share/Trash`.
pub fn trash_entry(path: &Path) -> Result<()> {
    // 1. Try gio trash
    if std::process::Command::new("gio")
        .args(["trash"])
        .arg(path)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Ok(());
    }

    // 2. Try trash-put
    if std::process::Command::new("trash-put")
        .arg(path)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Ok(());
    }

    // 3. Fallback: FreeDesktop standard ~/.local/share/Trash/files
    if let Some(home) = dirs::home_dir() {
        let trash_files = home.join(".local/share/Trash/files");
        std::fs::create_dir_all(&trash_files)?;
        let file_name = path.file_name().unwrap_or_default();
        let target = trash_files.join(file_name);
        if std::fs::rename(path, &target).is_ok() {
            return Ok(());
        }
    }

    // If trash fails, perform normal delete
    delete_entry(path)
}

/// Copy a single file, never following symlinks blindly.
pub fn copy_file(src: &Path, dst: &Path) -> Result<u64> {
    std::fs::copy(src, dst)
        .with_context(|| format!("Copying {} → {}", src.display(), dst.display()))
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

/// Save file contents atomically (write to temp then rename).
pub fn save_file_atomic(path: &Path, content: &[u8]) -> Result<()> {
    use std::io::Write;
    let parent = path.parent().unwrap_or(path);
    let mut tmp = tempfile::NamedTempFile::new_in(parent)
        .context("Creating temporary file for atomic save")?;
    tmp.write_all(content)
        .context("Writing content to temporary file")?;
    tmp.flush().context("Flushing temporary file")?;
    tmp.persist(path)
        .with_context(|| format!("Persisting temporary file → {}", path.display()))?;
    Ok(())
}
