//! File metadata helpers.

use anyhow::Result;
use chrono::{DateTime, Local};
use std::path::Path;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub path: String,
    pub name: String,
    pub extension: Option<String>,
    pub size_bytes: u64,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub modified: Option<DateTime<Local>>,
    pub permissions: String,
    pub mime_guess: String,
}

pub fn get_metadata(path: &Path) -> Result<FileMetadata> {
    let meta = std::fs::metadata(path)?;
    let sym_meta = std::fs::symlink_metadata(path)?;

    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let extension = path.extension().map(|e| e.to_string_lossy().to_string());

    let modified = meta.modified().ok().map(DateTime::<Local>::from);

    let permissions = {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = meta.permissions().mode();
            format!("{:o}", mode & 0o7777)
        }
        #[cfg(not(unix))]
        {
            String::from("N/A")
        }
    };

    let mime_guess = guess_mime(extension.as_deref());

    Ok(FileMetadata {
        path: path.to_string_lossy().to_string(),
        name,
        extension,
        size_bytes: meta.len(),
        is_dir: meta.is_dir(),
        is_symlink: sym_meta.file_type().is_symlink(),
        modified,
        permissions,
        mime_guess,
    })
}

fn guess_mime(ext: Option<&str>) -> String {
    let s = match ext.unwrap_or("").to_lowercase().as_str() {
        "rs" => "text/x-rust",
        "py" => "text/x-python",
        "js" => "text/javascript",
        "ts" => "text/typescript",
        "json" => "application/json",
        "toml" => "text/x-toml",
        "yaml" | "yml" => "text/yaml",
        "md" => "text/markdown",
        "txt" => "text/plain",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "c" => "text/x-c",
        "cpp" | "cxx" | "cc" => "text/x-c++",
        "h" | "hpp" => "text/x-c-header",
        "java" => "text/x-java",
        "sh" | "bash" | "zsh" | "fish" => "text/x-shellscript",
        "xml" => "text/xml",
        "log" => "text/plain",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        _ => "application/octet-stream",
    };
    s.to_string()
}

/// Quick heuristic: is the byte slice likely to be binary?
pub fn is_binary(bytes: &[u8]) -> bool {
    // If more than 10% of the first 8KB are non-text bytes, treat as binary.
    let sample = &bytes[..bytes.len().min(8192)];
    let non_text = sample
        .iter()
        .filter(|&&b| b == 0 || (b < 8 && b != b'\n' && b != b'\r' && b != b'\t'))
        .count();
    non_text * 10 > sample.len()
}
