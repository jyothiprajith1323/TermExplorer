//! Project Workspace and Smart Environment Detection.

use crate::settings::ProfileKind;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectKind {
    Rust,
    Python,
    NodeJs,
    Go,
    CPlusPlus,
    Java,
    GenericMake,
    Unknown,
}

impl ProjectKind {
    pub fn badge(&self) -> &'static str {
        match self {
            Self::Rust => "🦀 Rust (Cargo)",
            Self::Python => "🐍 Python",
            Self::NodeJs => "⬢ Node.js / TypeScript",
            Self::Go => "🐹 Go",
            Self::CPlusPlus => "⚙ C / C++",
            Self::Java => "☕ Java",
            Self::GenericMake => "🛠 Makefile",
            Self::Unknown => "📁 Workspace",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectQuickAction {
    pub label: &'static str,
    pub command: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone)]
pub struct Workspace {
    pub name: String,
    pub root_dir: PathBuf,
    pub active_dir: PathBuf,
    pub open_files: Vec<PathBuf>,
    pub project_kind: ProjectKind,
    pub profile: ProfileKind,
}

impl Workspace {
    pub fn from_dir(dir: &Path) -> Self {
        let name = dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Default")
            .to_string();

        let project_kind = detect_project_kind(dir);

        Self {
            name,
            root_dir: dir.to_path_buf(),
            active_dir: dir.to_path_buf(),
            open_files: Vec::new(),
            project_kind,
            profile: ProfileKind::Default,
        }
    }

    /// Provide project-specific quick actions based on detected project type.
    pub fn quick_actions(&self) -> Vec<ProjectQuickAction> {
        match self.project_kind {
            ProjectKind::Rust => vec![
                ProjectQuickAction {
                    label: "cargo check",
                    command: "cargo check",
                    description: "Quickly verify syntax and typecheck",
                },
                ProjectQuickAction {
                    label: "cargo test",
                    command: "cargo test",
                    description: "Run Rust integration & unit tests",
                },
                ProjectQuickAction {
                    label: "cargo build",
                    command: "cargo build --release",
                    description: "Compile release binary",
                },
                ProjectQuickAction {
                    label: "cargo clippy",
                    command: "cargo clippy -- -D warnings",
                    description: "Run strict Rust linter",
                },
            ],
            ProjectKind::Python => vec![
                ProjectQuickAction {
                    label: "pytest",
                    command: "pytest",
                    description: "Run Python test suite",
                },
                ProjectQuickAction {
                    label: "python -m venv .venv",
                    command: "python3 -m venv .venv",
                    description: "Create virtual environment",
                },
                ProjectQuickAction {
                    label: "black / ruff",
                    command: "ruff check .",
                    description: "Run Python linter",
                },
            ],
            ProjectKind::NodeJs => vec![
                ProjectQuickAction {
                    label: "npm test",
                    command: "npm test",
                    description: "Run Node / JS test suite",
                },
                ProjectQuickAction {
                    label: "npm run build",
                    command: "npm run build",
                    description: "Build production bundle",
                },
                ProjectQuickAction {
                    label: "npm start",
                    command: "npm start",
                    description: "Start local development server",
                },
            ],
            ProjectKind::Go => vec![
                ProjectQuickAction {
                    label: "go test ./...",
                    command: "go test ./...",
                    description: "Run Go test suite",
                },
                ProjectQuickAction {
                    label: "go build",
                    command: "go build -v",
                    description: "Build Go executable",
                },
            ],
            ProjectKind::GenericMake | ProjectKind::CPlusPlus => vec![
                ProjectQuickAction {
                    label: "make",
                    command: "make",
                    description: "Build default target",
                },
                ProjectQuickAction {
                    label: "make test",
                    command: "make test",
                    description: "Execute test target",
                },
                ProjectQuickAction {
                    label: "make clean",
                    command: "make clean",
                    description: "Clean build artifacts",
                },
            ],
            _ => Vec::new(),
        }
    }
}

/// Detect project type from indicator files.
pub fn detect_project_kind(dir: &Path) -> ProjectKind {
    if dir.join("Cargo.toml").exists() {
        return ProjectKind::Rust;
    }
    if dir.join("pyproject.toml").exists()
        || dir.join("requirements.txt").exists()
        || dir.join("Pipfile").exists()
    {
        return ProjectKind::Python;
    }
    if dir.join("package.json").exists() {
        return ProjectKind::NodeJs;
    }
    if dir.join("go.mod").exists() {
        return ProjectKind::Go;
    }
    if dir.join("CMakeLists.txt").exists() {
        return ProjectKind::CPlusPlus;
    }
    if dir.join("pom.xml").exists() || dir.join("build.gradle").exists() {
        return ProjectKind::Java;
    }
    if dir.join("Makefile").exists() || dir.join("makefile").exists() {
        return ProjectKind::GenericMake;
    }

    ProjectKind::Unknown
}
