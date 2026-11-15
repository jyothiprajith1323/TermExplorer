//! Read-Only Project Technology Detection & Task Discovery.
//!
//! Inspects directory markers and project manifests to automatically identify
//! technology ecosystems and synthesize safe, suggested task definitions.
//! Strictly read-only: never executes compiler or interpreter binaries during discovery.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::tasks::{TaskCategory, TaskDefinition, TaskId, TaskOutputMode};

/// Supported technology ecosystems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TechStack {
    Rust,
    Node,
    Python,
    Go,
    CMake,
    Make,
}

impl TechStack {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Rust => "Rust (Cargo)",
            Self::Node => "Node.js",
            Self::Python => "Python",
            Self::Go => "Go",
            Self::CMake => "CMake",
            Self::Make => "Makefile",
        }
    }
}

/// Discovered profile and suggested tasks for a project root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectProfile {
    pub root: PathBuf,
    pub detected_stacks: Vec<TechStack>,
    pub suggested_tasks: Vec<TaskDefinition>,
    pub default_build_task: Option<TaskId>,
    pub default_test_task: Option<TaskId>,
    pub default_lint_task: Option<TaskId>,
    pub default_run_task: Option<TaskId>,
}

/// Detects project technologies and synthesizes suggested development tasks.
pub fn detect_project(dir: &Path) -> ProjectProfile {
    let mut detected_stacks = Vec::new();
    let mut suggested_tasks = Vec::new();

    let mut default_build_task = None;
    let mut default_test_task = None;
    let mut default_lint_task = None;
    let mut default_run_task = None;

    // 1. Rust / Cargo
    if dir.join("Cargo.toml").is_file() {
        detected_stacks.push(TechStack::Rust);

        let cargo_check = TaskDefinition {
            id: "cargo.check".into(),
            display_name: "Cargo Check".into(),
            description: "Fast compilation check without code generation".into(),
            command: "cargo".into(),
            args: vec!["check".into()],
            working_dir: None,
            env: HashMap::new(),
            timeout_secs: Some(300),
            depends_on: Vec::new(),
            category: TaskCategory::Lint,
            output_mode: TaskOutputMode::NonInteractive,
            is_shell: false,
        };
        default_lint_task = Some("cargo.check".into());
        suggested_tasks.push(cargo_check);

        let cargo_test = TaskDefinition {
            id: "cargo.test".into(),
            display_name: "Cargo Test".into(),
            description: "Run unit and integration tests".into(),
            command: "cargo".into(),
            args: vec!["test".into()],
            working_dir: None,
            env: HashMap::new(),
            timeout_secs: Some(600),
            depends_on: Vec::new(),
            category: TaskCategory::Test,
            output_mode: TaskOutputMode::NonInteractive,
            is_shell: false,
        };
        default_test_task = Some("cargo.test".into());
        suggested_tasks.push(cargo_test);

        let cargo_clippy = TaskDefinition {
            id: "cargo.clippy".into(),
            display_name: "Cargo Clippy".into(),
            description: "Run Rust linter with warnings as errors".into(),
            command: "cargo".into(),
            args: vec![
                "clippy".into(),
                "--all-targets".into(),
                "--all-features".into(),
                "--".into(),
                "-D".into(),
                "warnings".into(),
            ],
            working_dir: None,
            env: HashMap::new(),
            timeout_secs: Some(300),
            depends_on: Vec::new(),
            category: TaskCategory::Lint,
            output_mode: TaskOutputMode::NonInteractive,
            is_shell: false,
        };
        suggested_tasks.push(cargo_clippy);

        let cargo_build = TaskDefinition {
            id: "cargo.build".into(),
            display_name: "Cargo Build (Debug)".into(),
            description: "Compile project in debug mode".into(),
            command: "cargo".into(),
            args: vec!["build".into()],
            working_dir: None,
            env: HashMap::new(),
            timeout_secs: Some(600),
            depends_on: Vec::new(),
            category: TaskCategory::Build,
            output_mode: TaskOutputMode::NonInteractive,
            is_shell: false,
        };
        default_build_task = Some("cargo.build".into());
        suggested_tasks.push(cargo_build);

        let cargo_build_release = TaskDefinition {
            id: "cargo.build.release".into(),
            display_name: "Cargo Build (Release)".into(),
            description: "Compile optimized release binary with LTO".into(),
            command: "cargo".into(),
            args: vec!["build".into(), "--release".into()],
            working_dir: None,
            env: HashMap::new(),
            timeout_secs: Some(1200),
            depends_on: Vec::new(),
            category: TaskCategory::Build,
            output_mode: TaskOutputMode::NonInteractive,
            is_shell: false,
        };
        suggested_tasks.push(cargo_build_release);

        let cargo_doc = TaskDefinition {
            id: "cargo.doc".into(),
            display_name: "Cargo Doc".into(),
            description: "Generate documentation for crate".into(),
            command: "cargo".into(),
            args: vec!["doc".into(), "--no-deps".into()],
            working_dir: None,
            env: HashMap::new(),
            timeout_secs: Some(300),
            depends_on: Vec::new(),
            category: TaskCategory::Doc,
            output_mode: TaskOutputMode::NonInteractive,
            is_shell: false,
        };
        suggested_tasks.push(cargo_doc);

        let cargo_run = TaskDefinition {
            id: "cargo.run".into(),
            display_name: "Cargo Run".into(),
            description: "Run the main binary interactively".into(),
            command: "cargo".into(),
            args: vec!["run".into()],
            working_dir: None,
            env: HashMap::new(),
            timeout_secs: None,
            depends_on: Vec::new(),
            category: TaskCategory::Run,
            output_mode: TaskOutputMode::Interactive,
            is_shell: false,
        };
        default_run_task = Some("cargo.run".into());
        suggested_tasks.push(cargo_run);
    }

    // 2. Node.js (package.json)
    let package_json = dir.join("package.json");
    if package_json.is_file() {
        detected_stacks.push(TechStack::Node);

        // Detect package manager preference from lockfiles
        let pkg_mgr = if dir.join("pnpm-lock.yaml").is_file() {
            "pnpm"
        } else if dir.join("yarn.lock").is_file() {
            "yarn"
        } else {
            "npm"
        };

        // Parse scripts read-only from package.json without executing anything
        if let Ok(content) = fs::read_to_string(&package_json) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(scripts) = val.get("scripts").and_then(|s| s.as_object()) {
                    for (script_name, script_cmd) in scripts {
                        let cat = match script_name.as_str() {
                            "build" => {
                                default_build_task.get_or_insert_with(|| {
                                    format!("{}.{}", pkg_mgr, script_name).into()
                                });
                                TaskCategory::Build
                            }
                            "test" => {
                                default_test_task.get_or_insert_with(|| {
                                    format!("{}.{}", pkg_mgr, script_name).into()
                                });
                                TaskCategory::Test
                            }
                            "lint" | "check" => {
                                default_lint_task.get_or_insert_with(|| {
                                    format!("{}.{}", pkg_mgr, script_name).into()
                                });
                                TaskCategory::Lint
                            }
                            "dev" | "start" => {
                                default_run_task.get_or_insert_with(|| {
                                    format!("{}.{}", pkg_mgr, script_name).into()
                                });
                                TaskCategory::Run
                            }
                            _ => TaskCategory::Custom,
                        };

                        let output_mode = if script_name == "dev" || script_name == "start" {
                            TaskOutputMode::Interactive
                        } else {
                            TaskOutputMode::NonInteractive
                        };

                        suggested_tasks.push(TaskDefinition {
                            id: format!("{}.{}", pkg_mgr, script_name).into(),
                            display_name: format!("{} run {}", pkg_mgr, script_name),
                            description: format!(
                                "Run script: {}",
                                script_cmd.as_str().unwrap_or("")
                            ),
                            command: pkg_mgr.into(),
                            args: vec!["run".into(), script_name.clone()],
                            working_dir: None,
                            env: HashMap::new(),
                            timeout_secs: Some(600),
                            depends_on: Vec::new(),
                            category: cat,
                            output_mode,
                            is_shell: false,
                        });
                    }
                }
            }
        }
    }

    // 3. Python (pyproject.toml, requirements.txt, setup.py)
    if dir.join("pyproject.toml").is_file()
        || dir.join("requirements.txt").is_file()
        || dir.join("setup.py").is_file()
    {
        detected_stacks.push(TechStack::Python);

        let py_test = TaskDefinition {
            id: "python.pytest".into(),
            display_name: "pytest".into(),
            description: "Run Python tests via pytest".into(),
            command: "pytest".into(),
            args: Vec::new(),
            working_dir: None,
            env: HashMap::new(),
            timeout_secs: Some(300),
            depends_on: Vec::new(),
            category: TaskCategory::Test,
            output_mode: TaskOutputMode::NonInteractive,
            is_shell: false,
        };
        default_test_task.get_or_insert_with(|| "python.pytest".into());
        suggested_tasks.push(py_test);

        let py_lint = TaskDefinition {
            id: "python.ruff".into(),
            display_name: "ruff check".into(),
            description: "Fast Python linter".into(),
            command: "ruff".into(),
            args: vec!["check".into(), ".".into()],
            working_dir: None,
            env: HashMap::new(),
            timeout_secs: Some(180),
            depends_on: Vec::new(),
            category: TaskCategory::Lint,
            output_mode: TaskOutputMode::NonInteractive,
            is_shell: false,
        };
        default_lint_task.get_or_insert_with(|| "python.ruff".into());
        suggested_tasks.push(py_lint);
    }

    // 4. Go (go.mod)
    if dir.join("go.mod").is_file() {
        detected_stacks.push(TechStack::Go);

        let go_build = TaskDefinition {
            id: "go.build".into(),
            display_name: "go build".into(),
            description: "Compile Go packages and dependencies".into(),
            command: "go".into(),
            args: vec!["build".into(), "./...".into()],
            working_dir: None,
            env: HashMap::new(),
            timeout_secs: Some(300),
            depends_on: Vec::new(),
            category: TaskCategory::Build,
            output_mode: TaskOutputMode::NonInteractive,
            is_shell: false,
        };
        default_build_task.get_or_insert_with(|| "go.build".into());
        suggested_tasks.push(go_build);

        let go_test = TaskDefinition {
            id: "go.test".into(),
            display_name: "go test".into(),
            description: "Run Go package test suites".into(),
            command: "go".into(),
            args: vec!["test".into(), "./...".into()],
            working_dir: None,
            env: HashMap::new(),
            timeout_secs: Some(300),
            depends_on: Vec::new(),
            category: TaskCategory::Test,
            output_mode: TaskOutputMode::NonInteractive,
            is_shell: false,
        };
        default_test_task.get_or_insert_with(|| "go.test".into());
        suggested_tasks.push(go_test);
    }

    // 5. C/C++ (CMakeLists.txt, Makefile)
    if dir.join("CMakeLists.txt").is_file() {
        detected_stacks.push(TechStack::CMake);

        let cmake_build = TaskDefinition {
            id: "cmake.build".into(),
            display_name: "cmake --build".into(),
            description: "Build project using configured CMake generator".into(),
            command: "cmake".into(),
            args: vec!["--build".into(), "build".into()],
            working_dir: None,
            env: HashMap::new(),
            timeout_secs: Some(600),
            depends_on: Vec::new(),
            category: TaskCategory::Build,
            output_mode: TaskOutputMode::NonInteractive,
            is_shell: false,
        };
        default_build_task.get_or_insert_with(|| "cmake.build".into());
        suggested_tasks.push(cmake_build);
    } else if dir.join("Makefile").is_file() || dir.join("makefile").is_file() {
        detected_stacks.push(TechStack::Make);

        let make_all = TaskDefinition {
            id: "make.all".into(),
            display_name: "make".into(),
            description: "Execute default Makefile target".into(),
            command: "make".into(),
            args: Vec::new(),
            working_dir: None,
            env: HashMap::new(),
            timeout_secs: Some(600),
            depends_on: Vec::new(),
            category: TaskCategory::Build,
            output_mode: TaskOutputMode::NonInteractive,
            is_shell: false,
        };
        default_build_task.get_or_insert_with(|| "make.all".into());
        suggested_tasks.push(make_all);
    }

    ProjectProfile {
        root: dir.to_path_buf(),
        detected_stacks,
        suggested_tasks,
        default_build_task,
        default_test_task,
        default_lint_task,
        default_run_task,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_detect_rust_cargo_project() {
        let dir = tempdir().unwrap();
        let cargo_toml = dir.path().join("Cargo.toml");
        fs::write(
            &cargo_toml,
            "[package]\nname = \"test_pkg\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        let profile = detect_project(dir.path());
        assert!(profile.detected_stacks.contains(&TechStack::Rust));
        assert!(profile
            .suggested_tasks
            .iter()
            .any(|t| t.id == "cargo.check".into()));
        assert!(profile
            .suggested_tasks
            .iter()
            .any(|t| t.id == "cargo.test".into()));
        assert!(profile
            .suggested_tasks
            .iter()
            .any(|t| t.id == "cargo.clippy".into()));
        assert_eq!(profile.default_test_task, Some("cargo.test".into()));
    }

    #[test]
    fn test_detect_node_project_scripts_read_only() {
        let dir = tempdir().unwrap();
        let pkg_json = dir.path().join("package.json");
        fs::write(
            &pkg_json,
            r#"{
                "name": "my-web-app",
                "scripts": {
                    "build": "vite build",
                    "test": "vitest run",
                    "lint": "eslint src/"
                }
            }"#,
        )
        .unwrap();

        let profile = detect_project(dir.path());
        assert!(profile.detected_stacks.contains(&TechStack::Node));
        assert!(profile
            .suggested_tasks
            .iter()
            .any(|t| t.id == "npm.build".into()));
        assert!(profile
            .suggested_tasks
            .iter()
            .any(|t| t.id == "npm.test".into()));
        assert_eq!(profile.default_build_task, Some("npm.build".into()));
        assert_eq!(profile.default_test_task, Some("npm.test".into()));
    }

    #[test]
    fn test_detect_empty_directory() {
        let dir = tempdir().unwrap();
        let profile = detect_project(dir.path());
        assert!(profile.detected_stacks.is_empty());
        assert!(profile.suggested_tasks.is_empty());
    }
}
