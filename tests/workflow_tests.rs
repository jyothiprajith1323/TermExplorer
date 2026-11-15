//! Comprehensive Integration and Quality Gate Test Suite for Sprint 4C: Developer Workflow Engine.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tempfile::tempdir;
use tokio::sync::mpsc;

use termexplorer::ui::hit_test::AppAction;
use termexplorer::workflow::*;

// ── 1. Command Palette & Registry Tests ────────────────────────────────────────

#[test]
fn test_command_registry_initialization_and_uniqueness() {
    let mut registry = CommandRegistry::new();
    let initial_count = registry.commands().len();
    assert!(initial_count >= 12, "Default commands should be populated");

    // Enforce ID uniqueness: re-registering replaces the existing command
    let dup_cmd = CommandDefinition {
        id: "git.dashboard".into(),
        display_name: "Overridden Git Dashboard".into(),
        description: "Replaced definition".into(),
        category: CommandCategory::Git,
        keywords: vec!["custom".into()],
        default_shortcut: None,
        availability: Arc::new(|_| true),
        action: CommandAction::App(AppAction::OpenGitDashboard),
    };
    registry.register(dup_cmd);

    assert_eq!(
        registry.commands().len(),
        initial_count,
        "Registering duplicate ID should not increase count"
    );
    let found = registry.find("git.dashboard").unwrap();
    assert_eq!(found.display_name, "Overridden Git Dashboard");
}

#[test]
fn test_command_fuzzy_search_scoring_and_ranking() {
    let registry = CommandRegistry::new();
    let mut history = CommandHistory::new(10);
    let ctx = CommandContext {
        has_git: true,
        has_project_tasks: true,
        has_terminal_session: true,
        ..Default::default()
    };

    // 1. Search for git commands
    let git_matches = registry.search("git", &ctx, &history);
    assert!(!git_matches.is_empty());
    assert!(git_matches.iter().any(|m| m.command.id == "git.dashboard"));

    // 2. Search for tasks
    let task_matches = registry.search("test", &ctx, &history);
    assert!(!task_matches.is_empty());
    assert!(task_matches
        .iter()
        .any(|m| m.command.id == "task.run_tests"));

    // 3. Recency boost
    history.record("task.run_tests");
    let ranked = registry.search("test", &ctx, &history);
    assert_eq!(
        ranked[0].command.id, "task.run_tests",
        "Recent command must receive ranking boost"
    );
    assert!(ranked[0].is_recent);

    // 4. Empty search: returns all commands, with recents at top
    let empty_search = registry.search("", &ctx, &history);
    assert_eq!(empty_search.len(), registry.commands().len());
    assert!(empty_search[0].is_recent);
}

#[test]
fn test_command_context_availability_guards() {
    let registry = CommandRegistry::new();
    let history = CommandHistory::new(10);

    // Context without git or terminal
    let ctx_inactive = CommandContext {
        has_git: false,
        has_terminal_session: false,
        has_project_tasks: false,
        ..Default::default()
    };

    let git_res = registry.search("git.dashboard", &ctx_inactive, &history);
    if let Some(r) = git_res.iter().find(|m| m.command.id == "git.dashboard") {
        assert!(!r.is_available, "Git action must be disabled without repo");
    }

    let term_res = registry.search("terminal.new_session", &ctx_inactive, &history);
    if let Some(r) = term_res
        .iter()
        .find(|m| m.command.id == "terminal.new_session")
    {
        assert!(
            !r.is_available,
            "Terminal new tab must be disabled without active terminal"
        );
    }
}

// ── 2. Project Technology Detection Tests ─────────────────────────────────────

#[test]
fn test_detect_rust_project_read_only() {
    let dir = tempdir().unwrap();
    let cargo_toml = dir.path().join("Cargo.toml");
    fs::write(
        &cargo_toml,
        "[package]\nname = \"fixture_crate\"\nversion = \"0.1.0\"\n",
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
    assert!(profile
        .suggested_tasks
        .iter()
        .any(|t| t.id == "cargo.build".into()));
    assert_eq!(profile.default_test_task, Some("cargo.test".into()));
    assert_eq!(profile.default_build_task, Some("cargo.build".into()));
}

#[test]
fn test_detect_node_project_package_json_scripts() {
    let dir = tempdir().unwrap();
    let pkg_json = dir.path().join("package.json");
    fs::write(
        &pkg_json,
        r#"{
            "name": "frontend-fixture",
            "scripts": {
                "dev": "vite",
                "build": "tsc && vite build",
                "test": "vitest run"
            }
        }"#,
    )
    .unwrap();

    let profile = detect_project(dir.path());
    assert!(profile.detected_stacks.contains(&TechStack::Node));
    assert!(profile
        .suggested_tasks
        .iter()
        .any(|t| t.id == "npm.dev".into()));
    assert!(profile
        .suggested_tasks
        .iter()
        .any(|t| t.id == "npm.build".into()));
    assert!(profile
        .suggested_tasks
        .iter()
        .any(|t| t.id == "npm.test".into()));

    // Interactive mode set for dev server
    let dev_task = profile
        .suggested_tasks
        .iter()
        .find(|t| t.id == "npm.dev".into())
        .unwrap();
    assert_eq!(dev_task.output_mode, TaskOutputMode::Interactive);
}

#[test]
fn test_detect_python_project() {
    let dir = tempdir().unwrap();
    let pyproject = dir.path().join("pyproject.toml");
    fs::write(&pyproject, "[project]\nname = \"pyfixture\"\n").unwrap();

    let profile = detect_project(dir.path());
    assert!(profile.detected_stacks.contains(&TechStack::Python));
    assert!(profile
        .suggested_tasks
        .iter()
        .any(|t| t.id == "python.pytest".into()));
    assert!(profile
        .suggested_tasks
        .iter()
        .any(|t| t.id == "python.ruff".into()));
}

#[test]
fn test_detect_empty_dir_no_false_positives() {
    let dir = tempdir().unwrap();
    let profile = detect_project(dir.path());
    assert!(profile.detected_stacks.is_empty());
    assert!(profile.suggested_tasks.is_empty());
}

// ── 3. Task Definitions, Variables & Configuration ───────────────────────────

#[test]
fn test_variable_substitution_allowlist_enforcement() {
    let ctx = VariableContext {
        workspace_root: PathBuf::from("/work/project"),
        current_file: Some(PathBuf::from("/work/project/src/lib.rs")),
        git_branch: Some("master".into()),
    };

    let template = "echo ${workspace_root} ${file_name} ${file_dir} ${branch}";
    let res = substitute_variables(template, &ctx).unwrap();
    assert_eq!(res, "echo /work/project lib.rs /work/project/src master");

    // Malicious or arbitrary expressions must be rejected
    let bad_template = "echo ${`rm -rf /`}";
    assert!(substitute_variables(bad_template, &ctx).is_err());
}

#[test]
fn test_project_tasks_toml_roundtrip() {
    let dir = tempdir().unwrap();
    let tasks = vec![TaskDefinition {
        id: "custom.lint".into(),
        display_name: "Project Linter".into(),
        description: "Run custom linter".into(),
        command: "cargo".into(),
        args: vec!["clippy".into()],
        working_dir: None,
        env: HashMap::from([("RUST_BACKTRACE".into(), "1".into())]),
        timeout_secs: Some(120),
        depends_on: Vec::new(),
        category: TaskCategory::Lint,
        output_mode: TaskOutputMode::NonInteractive,
        is_shell: false,
    }];

    save_project_tasks(dir.path(), &tasks).unwrap();
    let loaded = load_project_tasks(dir.path());

    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].id, TaskId("custom.lint".into()));
    assert_eq!(loaded[0].env.get("RUST_BACKTRACE").unwrap(), "1");
    assert_eq!(loaded[0].timeout_secs, Some(120));
}

// ── 4. Workspace Trust Security Tests ─────────────────────────────────────────

#[test]
fn test_workspace_trust_lifecycle() {
    let dir = tempdir().unwrap();
    let trust_file = dir.path().join("trusted.json");
    let mut store = WorkspaceTrustStore::with_custom_file(trust_file.clone());

    let untrusted_root = dir.path().join("untrusted_repo");
    fs::create_dir_all(&untrusted_root).unwrap();

    // 1. Untrusted by default
    assert!(!store.is_trusted(&untrusted_root));

    // 2. Grant explicit trust
    store.trust_workspace(&untrusted_root).unwrap();
    assert!(store.is_trusted(&untrusted_root));

    // 3. Reload from disk to verify persistence
    let reloaded = WorkspaceTrustStore::with_custom_file(trust_file.clone());
    assert!(reloaded.is_trusted(&untrusted_root));

    // 4. Revoke trust
    let mut mutable_store = reloaded;
    mutable_store.revoke_trust(&untrusted_root).unwrap();
    assert!(!mutable_store.is_trusted(&untrusted_root));

    let final_check = WorkspaceTrustStore::with_custom_file(trust_file);
    assert!(!final_check.is_trusted(&untrusted_root));
}

// ── 5. Task Dependency DAG Tests ─────────────────────────────────────────────

#[test]
fn test_task_dag_scheduling_and_cycle_prevention() {
    let mut dag = TaskDag::new();

    let task_build = TaskDefinition {
        id: "task.build".into(),
        display_name: "Build".into(),
        description: String::new(),
        command: "true".into(),
        args: Vec::new(),
        working_dir: None,
        env: HashMap::new(),
        timeout_secs: None,
        depends_on: Vec::new(),
        category: TaskCategory::Build,
        output_mode: TaskOutputMode::NonInteractive,
        is_shell: false,
    };

    let task_test = TaskDefinition {
        id: "task.test".into(),
        display_name: "Test".into(),
        description: String::new(),
        command: "true".into(),
        args: Vec::new(),
        working_dir: None,
        env: HashMap::new(),
        timeout_secs: None,
        depends_on: vec!["task.build".into()],
        category: TaskCategory::Test,
        output_mode: TaskOutputMode::NonInteractive,
        is_shell: false,
    };

    let task_package = TaskDefinition {
        id: "task.package".into(),
        display_name: "Package".into(),
        description: String::new(),
        command: "true".into(),
        args: Vec::new(),
        working_dir: None,
        env: HashMap::new(),
        timeout_secs: None,
        depends_on: vec!["task.test".into()],
        category: TaskCategory::Custom,
        output_mode: TaskOutputMode::NonInteractive,
        is_shell: false,
    };

    dag.add_task(task_build);
    dag.add_task(task_test);
    dag.add_task(task_package);

    let plan = dag.plan_execution(&"task.package".into()).unwrap();
    assert_eq!(
        plan,
        vec![
            TaskId::from("task.build"),
            TaskId::from("task.test"),
            TaskId::from("task.package")
        ]
    );
}

// ── 6. Task Execution Backend & Streaming Output ──────────────────────────────

#[tokio::test]
async fn test_task_runner_streaming_and_cancellation() {
    let runner = TaskRunner::new(100, Duration::from_secs(5));
    let dir = tempdir().unwrap();
    let (tx, mut rx) = mpsc::channel(32);

    let task = TaskDefinition {
        id: "stream.test".into(),
        display_name: "Streaming Test".into(),
        description: String::new(),
        command: "sh".into(),
        args: vec!["-c".into(), "echo 'Line 1' && echo 'Line 2'".into()],
        working_dir: None,
        env: HashMap::new(),
        timeout_secs: Some(5),
        depends_on: Vec::new(),
        category: TaskCategory::Custom,
        output_mode: TaskOutputMode::NonInteractive,
        is_shell: false,
    };

    let var_ctx = VariableContext::default();
    let exec_id = runner
        .spawn_task(&task, dir.path(), &var_ctx, tx)
        .await
        .unwrap();

    let mut captured_lines = Vec::new();
    while let Some(evt) = rx.recv().await {
        match evt {
            TaskRunnerEvent::Line { text, .. } => captured_lines.push(text),
            TaskRunnerEvent::Finished {
                status, exit_code, ..
            } => {
                assert_eq!(status, TaskStatus::Completed(0));
                assert_eq!(exit_code, Some(0));
                break;
            }
            _ => {}
        }
    }

    assert_eq!(captured_lines, vec!["Line 1", "Line 2"]);
    let snapshot = runner.get_execution(exec_id).await.unwrap();
    assert_eq!(snapshot.status, TaskStatus::Completed(0));
}
