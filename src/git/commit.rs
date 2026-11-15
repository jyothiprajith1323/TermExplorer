//! Git commit creation, commit message execution, and hook error reporting.

use std::path::Path;
use std::process::Command;

/// Create a new commit with the specified commit message.
pub fn create_commit(dir: &Path, message: &str) -> Result<String, String> {
    let trimmed = message.trim();
    if trimmed.is_empty() {
        return Err("Commit message cannot be empty".to_string());
    }

    let out = Command::new("git")
        .current_dir(dir)
        .args(["commit", "-m", trimmed])
        .output()
        .map_err(|e| format!("Failed to execute git commit: {e}"))?;

    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !stderr.is_empty() {
            Err(stderr)
        } else if !stdout.is_empty() {
            Err(stdout)
        } else {
            Err("Commit failed without error message".to_string())
        }
    }
}

/// Amend the previous commit with an optional new commit message.
pub fn amend_commit(dir: &Path, message: Option<&str>) -> Result<String, String> {
    let mut args = vec!["commit", "--amend"];
    let msg_str;
    if let Some(m) = message {
        msg_str = m.trim().to_string();
        if msg_str.is_empty() {
            return Err("Amend commit message cannot be empty".to_string());
        }
        args.push("-m");
        args.push(&msg_str);
    } else {
        args.push("--no-edit");
    }

    let out = Command::new("git")
        .current_dir(dir)
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to execute git commit --amend: {e}"))?;

    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}
