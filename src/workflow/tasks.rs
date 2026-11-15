//! Task Definitions, Variable Substitution & Execution Data Models.
//!
//! Enforces safe, declarative development tasks with allowlisted variable
//! expansion, output streaming representations, and execution lifecycles.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use serde::{Deserialize, Serialize};

/// Unique identifier of a task definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub String);

impl<T: Into<String>> From<T> for TaskId {
    fn from(s: T) -> Self {
        Self(s.into())
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Category of the task for indexing and quick discovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskCategory {
    Build,
    Test,
    Lint,
    Run,
    Doc,
    Clean,
    Custom,
}

impl TaskCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Build => "Build",
            Self::Test => "Test",
            Self::Lint => "Lint",
            Self::Run => "Run",
            Self::Doc => "Doc",
            Self::Clean => "Clean",
            Self::Custom => "Custom",
        }
    }
}

/// Mode of execution: NonInteractive (piped child process) or Interactive (PTY session).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TaskOutputMode {
    #[default]
    NonInteractive,
    Interactive,
}

/// Declarative task definition supported in `.termexplorer/tasks.toml` and project detectors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskDefinition {
    pub id: TaskId,
    pub display_name: String,
    pub description: String,
    /// Executable path or binary name (e.g. "cargo", "npm", "pytest").
    pub command: String,
    /// Explicit argument list (e.g. `["test", "--", "filter"]`).
    pub args: Vec<String>,
    /// Working directory relative to project root, or absolute path.
    pub working_dir: Option<PathBuf>,
    /// Environment variables specific to this task.
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Timeout in seconds before terminating the process (default 300s).
    pub timeout_secs: Option<u64>,
    /// Task IDs that must run and succeed prior to this task.
    #[serde(default)]
    pub depends_on: Vec<TaskId>,
    /// Category classification.
    #[serde(default = "default_category")]
    pub category: TaskCategory,
    /// NonInteractive (background process stream) or Interactive (embedded PTY shell).
    #[serde(default)]
    pub output_mode: TaskOutputMode,
    /// Whether to invoke via system shell (`sh -c` on Unix). Default false for security.
    #[serde(default)]
    pub is_shell: bool,
}

fn default_category() -> TaskCategory {
    TaskCategory::Custom
}

/// High-level lifecycle states of an executing task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Ready,
    Queued,
    Running,
    Cancelling,
    Completed(i32),
    Failed(i32),
    Cancelled,
    TimedOut,
}

impl TaskStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Queued | Self::Running | Self::Cancelling)
    }

    pub fn is_finished(&self) -> bool {
        !self.is_active()
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Ready => "Ready",
            Self::Queued => "Queued",
            Self::Running => "Running",
            Self::Cancelling => "Cancelling",
            Self::Completed(_) => "Completed",
            Self::Failed(_) => "Failed",
            Self::Cancelled => "Cancelled",
            Self::TimedOut => "Timed Out",
        }
    }
}

/// Stream indicator for captured task output lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskOutputStream {
    Stdout,
    Stderr,
    System,
}

/// A single timestamped output line captured from a running task.
#[derive(Debug, Clone)]
pub struct TaskOutputLine {
    pub timestamp_ms: u64,
    pub stream: TaskOutputStream,
    pub text: String,
}

/// Active or completed execution record for a task run.
#[derive(Debug, Clone)]
pub struct TaskExecution {
    pub execution_id: u64,
    pub task_id: TaskId,
    pub task_name: String,
    pub status: TaskStatus,
    pub started_at: Instant,
    pub finished_at: Option<Instant>,
    pub duration_ms: u64,
    pub exit_code: Option<i32>,
    pub output_lines: Vec<TaskOutputLine>,
    pub max_lines: usize,
}

impl TaskExecution {
    pub fn new(execution_id: u64, task_id: TaskId, task_name: String, max_lines: usize) -> Self {
        Self {
            execution_id,
            task_id,
            task_name,
            status: TaskStatus::Ready,
            started_at: Instant::now(),
            finished_at: None,
            duration_ms: 0,
            exit_code: None,
            output_lines: Vec::with_capacity(128),
            max_lines,
        }
    }

    pub fn append_line(&mut self, stream: TaskOutputStream, text: String) {
        let elapsed = self.started_at.elapsed().as_millis() as u64;
        self.output_lines.push(TaskOutputLine {
            timestamp_ms: elapsed,
            stream,
            text,
        });

        // Enforce bounded output buffer to prevent unbounded RAM consumption
        if self.output_lines.len() > self.max_lines {
            let overflow = self.output_lines.len() - self.max_lines;
            self.output_lines.drain(0..overflow);
        }
    }

    pub fn finish(&mut self, status: TaskStatus, exit_code: Option<i32>) {
        self.status = status;
        self.exit_code = exit_code;
        self.finished_at = Some(Instant::now());
        self.duration_ms = self.started_at.elapsed().as_millis() as u64;
    }
}

/// Context passed to allowlisted variable substitution in task arguments.
#[derive(Debug, Clone, Default)]
pub struct VariableContext {
    pub workspace_root: PathBuf,
    pub current_file: Option<PathBuf>,
    pub git_branch: Option<String>,
}

/// Safe allowlist variable substitution for task command strings and arguments.
///
/// Supported variables:
/// - `${workspace_root}` or `${project_root}`: Absolute path to the active project root.
/// - `${file}`: Full path of the currently selected/open file.
/// - `${file_name}`: Base filename of the currently selected file.
/// - `${file_dir}`: Directory containing the currently selected file.
/// - `${branch}`: Current Git branch name, or empty if not in a repository.
/// - `${env:VAR_NAME}`: Value of the host environment variable `VAR_NAME`.
pub fn substitute_variables(template: &str, ctx: &VariableContext) -> Result<String, String> {
    let mut result = String::with_capacity(template.len());
    let mut chars = template.char_indices().peekable();

    while let Some((idx, c)) = chars.next() {
        if c == '$' && chars.peek().map(|&(_, next_c)| next_c) == Some('{') {
            chars.next(); // Consume '{'
            let start = idx + 2;
            let mut end = None;

            for (e_idx, e_char) in chars.by_ref() {
                if e_char == '}' {
                    end = Some(e_idx);
                    break;
                }
            }

            let end_idx = end.ok_or_else(|| {
                format!(
                    "Unclosed variable substitution '${{{}' in template",
                    &template[start..]
                )
            })?;

            let var_name = &template[start..end_idx];

            match var_name {
                "workspace_root" | "project_root" => {
                    result.push_str(&ctx.workspace_root.to_string_lossy());
                }
                "file" => {
                    if let Some(f) = &ctx.current_file {
                        result.push_str(&f.to_string_lossy());
                    } else {
                        return Err(
                            "Variable ${file} referenced but no active file selected".into()
                        );
                    }
                }
                "file_name" => {
                    if let Some(f) = &ctx.current_file {
                        if let Some(name) = f.file_name() {
                            result.push_str(&name.to_string_lossy());
                        }
                    } else {
                        return Err(
                            "Variable ${file_name} referenced but no active file selected".into(),
                        );
                    }
                }
                "file_dir" => {
                    if let Some(f) = &ctx.current_file {
                        if let Some(parent) = f.parent() {
                            result.push_str(&parent.to_string_lossy());
                        }
                    } else {
                        return Err(
                            "Variable ${file_dir} referenced but no active file selected".into(),
                        );
                    }
                }
                "branch" => {
                    if let Some(b) = &ctx.git_branch {
                        result.push_str(b);
                    }
                }
                other if other.starts_with("env:") => {
                    let env_key = &other[4..];
                    if let Ok(val) = std::env::var(env_key) {
                        result.push_str(&val);
                    }
                }
                invalid => {
                    return Err(format!(
                        "Disallowed or unknown template variable '${{{}}}'. Only allowlisted variables are permitted.",
                        invalid
                    ));
                }
            }
        } else {
            result.push(c);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_substitution_workspace_and_branch() {
        let ctx = VariableContext {
            workspace_root: PathBuf::from("/home/user/myproject"),
            current_file: Some(PathBuf::from("/home/user/myproject/src/main.rs")),
            git_branch: Some("feature/sprint-4c".into()),
        };

        let substituted = substitute_variables(
            "cargo test --manifest-path ${workspace_root}/Cargo.toml --branch=${branch}",
            &ctx,
        )
        .unwrap();

        assert_eq!(
            substituted,
            "cargo test --manifest-path /home/user/myproject/Cargo.toml --branch=feature/sprint-4c"
        );
    }

    #[test]
    fn test_variable_substitution_file_name_and_dir() {
        let ctx = VariableContext {
            workspace_root: PathBuf::from("/home/user/project"),
            current_file: Some(PathBuf::from("/home/user/project/tests/unit_test.rs")),
            git_branch: None,
        };

        let sub_name = substitute_variables("File: ${file_name}", &ctx).unwrap();
        assert_eq!(sub_name, "File: unit_test.rs");

        let sub_dir = substitute_variables("Dir: ${file_dir}", &ctx).unwrap();
        assert_eq!(sub_dir, "Dir: /home/user/project/tests");
    }

    #[test]
    fn test_variable_substitution_rejects_arbitrary_eval() {
        let ctx = VariableContext::default();
        let res = substitute_variables("echo ${arbitrary_code_exec}", &ctx);
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .contains("Disallowed or unknown template variable"));
    }

    #[test]
    fn test_bounded_output_lines() {
        let mut exec = TaskExecution::new(1, "task-1".into(), "Test Task".into(), 5);
        for i in 0..10 {
            exec.append_line(TaskOutputStream::Stdout, format!("Line {}", i));
        }
        assert_eq!(exec.output_lines.len(), 5);
        assert_eq!(exec.output_lines[0].text, "Line 5");
        assert_eq!(exec.output_lines[4].text, "Line 9");
    }
}
