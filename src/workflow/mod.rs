//! Developer Workflow Engine Subsystem.
//!
//! Provides a unified Command Palette, project-aware Task Runner,
//! workspace trust boundary, dependency DAG scheduling, and configurable profiles.

pub mod command;
pub mod config;
pub mod dag;
pub mod detect;
pub mod runner;
pub mod tasks;
pub mod trust;

pub use command::{
    CommandAction, CommandCategory, CommandContext, CommandDefinition, CommandHistory,
    CommandRegistry, CommandSearchResult,
};
pub use config::{load_project_tasks, save_project_tasks};
pub use dag::{DagError, TaskDag};
pub use detect::{detect_project, ProjectProfile, TechStack};
pub use runner::{TaskRunner, TaskRunnerEvent};
pub use tasks::{
    substitute_variables, TaskCategory, TaskDefinition, TaskExecution, TaskId, TaskOutputLine,
    TaskOutputMode, TaskOutputStream, TaskStatus, VariableContext,
};
pub use trust::WorkspaceTrustStore;
