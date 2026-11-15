//! Declarative Configuration File Parser for Tasks and Workflow Profiles.
//!
//! Loads and saves `.termexplorer/tasks.toml` at the project level and
//! `tasks.toml` under the user XDG config directory with schema validation.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::tasks::{TaskCategory, TaskDefinition, TaskId, TaskOutputMode};

/// Top-level schema for `.termexplorer/tasks.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksConfigFile {
    pub version: u32,
    #[serde(default)]
    pub tasks: Vec<ConfigFileTask>,
}

/// Task structure serialized in TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFileTask {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub working_dir: Option<PathBuf>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default = "default_config_category")]
    pub category: String,
    #[serde(default)]
    pub interactive: bool,
    #[serde(default)]
    pub shell: bool,
}

fn default_config_category() -> String {
    "custom".into()
}

impl From<ConfigFileTask> for TaskDefinition {
    fn from(c: ConfigFileTask) -> Self {
        let cat = match c.category.to_lowercase().as_str() {
            "build" => TaskCategory::Build,
            "test" => TaskCategory::Test,
            "lint" => TaskCategory::Lint,
            "run" => TaskCategory::Run,
            "doc" => TaskCategory::Doc,
            "clean" => TaskCategory::Clean,
            _ => TaskCategory::Custom,
        };

        let output_mode = if c.interactive {
            TaskOutputMode::Interactive
        } else {
            TaskOutputMode::NonInteractive
        };

        TaskDefinition {
            id: TaskId(c.id),
            display_name: c.name,
            description: c.description,
            command: c.command,
            args: c.args,
            working_dir: c.working_dir,
            env: c.env,
            timeout_secs: c.timeout_secs,
            depends_on: c.depends_on.into_iter().map(TaskId).collect(),
            category: cat,
            output_mode,
            is_shell: c.shell,
        }
    }
}

/// Loads tasks from project `.termexplorer/tasks.toml` and user configuration.
pub fn load_project_tasks(project_root: &Path) -> Vec<TaskDefinition> {
    let mut tasks = Vec::new();

    // 1. User-level defaults (~/.config/termexplorer/tasks.toml)
    if let Some(config_dir) = dirs::config_dir() {
        let user_file = config_dir.join("termexplorer").join("tasks.toml");
        if user_file.is_file() {
            if let Ok(content) = fs::read_to_string(&user_file) {
                if let Ok(cf) = toml::from_str::<TasksConfigFile>(&content) {
                    if cf.version == 1 {
                        for t in cf.tasks {
                            tasks.push(t.into());
                        }
                    }
                }
            }
        }
    }

    // 2. Project-level overrides (.termexplorer/tasks.toml)
    let project_file = project_root.join(".termexplorer").join("tasks.toml");
    if project_file.is_file() {
        if let Ok(content) = fs::read_to_string(&project_file) {
            if let Ok(cf) = toml::from_str::<TasksConfigFile>(&content) {
                if cf.version == 1 {
                    for t in cf.tasks {
                        let task_def: TaskDefinition = t.into();
                        // Project definition overrides user default with same ID
                        tasks.retain(|existing: &TaskDefinition| existing.id != task_def.id);
                        tasks.push(task_def);
                    }
                }
            }
        }
    }

    tasks
}

/// Saves a list of task definitions to `.termexplorer/tasks.toml`.
pub fn save_project_tasks(project_root: &Path, tasks: &[TaskDefinition]) -> io::Result<()> {
    let termexplorer_dir = project_root.join(".termexplorer");
    fs::create_dir_all(&termexplorer_dir)?;

    let target_file = termexplorer_dir.join("tasks.toml");

    let mut config_tasks = Vec::new();
    for t in tasks {
        config_tasks.push(ConfigFileTask {
            id: t.id.0.clone(),
            name: t.display_name.clone(),
            description: t.description.clone(),
            command: t.command.clone(),
            args: t.args.clone(),
            working_dir: t.working_dir.clone(),
            env: t.env.clone(),
            timeout_secs: t.timeout_secs,
            depends_on: t.depends_on.iter().map(|d| d.0.clone()).collect(),
            category: t.category.label().to_lowercase(),
            interactive: t.output_mode == TaskOutputMode::Interactive,
            shell: t.is_shell,
        });
    }

    let config_file = TasksConfigFile {
        version: 1,
        tasks: config_tasks,
    };

    let toml_str = toml::to_string_pretty(&config_file).map_err(io::Error::other)?;

    let tmp_file = target_file.with_extension("tmp");
    fs::write(&tmp_file, toml_str)?;
    fs::rename(tmp_file, target_file)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_load_and_save_project_tasks_toml() {
        let dir = tempdir().unwrap();
        let tasks = vec![TaskDefinition {
            id: "build.fast".into(),
            display_name: "Fast Build".into(),
            description: "Quick cargo check".into(),
            command: "cargo".into(),
            args: vec!["check".into()],
            working_dir: None,
            env: HashMap::new(),
            timeout_secs: Some(60),
            depends_on: Vec::new(),
            category: TaskCategory::Build,
            output_mode: TaskOutputMode::NonInteractive,
            is_shell: false,
        }];

        save_project_tasks(dir.path(), &tasks).unwrap();

        let loaded = load_project_tasks(dir.path());
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, TaskId("build.fast".into()));
        assert_eq!(loaded[0].command, "cargo");
    }
}
