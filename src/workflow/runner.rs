//! Asynchronous Process Execution Supervisor & Output Streaming Backend.
//!
//! Spawns non-interactive tasks with process group isolation, captures stdout/stderr
//! streams incrementally, enforces timeouts and bounded buffers, and performs
//! safe SIGTERM / SIGKILL process-tree escalation on cancellation.

use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::{mpsc, Mutex};
use tokio::time::timeout;

use super::tasks::{
    substitute_variables, TaskDefinition, TaskExecution, TaskId, TaskOutputStream, TaskStatus,
    VariableContext,
};

/// Event emitted by a running task execution.
#[derive(Debug, Clone)]
pub enum TaskRunnerEvent {
    Started {
        execution_id: u64,
        task_id: TaskId,
        task_name: String,
    },
    Line {
        execution_id: u64,
        stream: TaskOutputStream,
        text: String,
    },
    Finished {
        execution_id: u64,
        status: TaskStatus,
        exit_code: Option<i32>,
        duration_ms: u64,
    },
}

/// Central supervisor managing active task processes and execution history.
pub struct TaskRunner {
    next_id: AtomicU64,
    executions: Arc<Mutex<HashMap<u64, TaskExecution>>>,
    abort_handles: Arc<Mutex<HashMap<u64, tokio::sync::oneshot::Sender<()>>>>,
    max_output_lines: usize,
    default_timeout: Duration,
}

impl Default for TaskRunner {
    fn default() -> Self {
        Self::new(5000, Duration::from_secs(300))
    }
}

impl TaskRunner {
    pub fn new(max_output_lines: usize, default_timeout: Duration) -> Self {
        Self {
            next_id: AtomicU64::new(1),
            executions: Arc::new(Mutex::new(HashMap::new())),
            abort_handles: Arc::new(Mutex::new(HashMap::new())),
            max_output_lines,
            default_timeout,
        }
    }

    /// Spawns a non-interactive task definition asynchronously and streams events to `event_tx`.
    pub async fn spawn_task(
        &self,
        task: &TaskDefinition,
        workspace_root: &Path,
        var_ctx: &VariableContext,
        event_tx: mpsc::Sender<TaskRunnerEvent>,
    ) -> Result<u64, String> {
        let execution_id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let mut execution = TaskExecution::new(
            execution_id,
            task.id.clone(),
            task.display_name.clone(),
            self.max_output_lines,
        );
        execution.status = TaskStatus::Running;

        // Substitute variables in command and arguments
        let cmd_name = substitute_variables(&task.command, var_ctx)
            .map_err(|e| format!("Command substitution error: {}", e))?;

        let mut final_args = Vec::new();
        for arg in &task.args {
            let sub_arg = substitute_variables(arg, var_ctx)
                .map_err(|e| format!("Arg substitution error: {}", e))?;
            final_args.push(sub_arg);
        }

        let working_dir = if let Some(ref wd) = task.working_dir {
            if wd.is_relative() {
                workspace_root.join(wd)
            } else {
                wd.clone()
            }
        } else {
            workspace_root.to_path_buf()
        };

        let mut cmd = if task.is_shell {
            let mut c = Command::new("sh");
            let mut full_cmd = cmd_name.clone();
            for a in &final_args {
                full_cmd.push(' ');
                full_cmd.push_str(a);
            }
            c.arg("-c").arg(full_cmd);
            c
        } else {
            let mut c = Command::new(&cmd_name);
            c.args(&final_args);
            c
        };

        cmd.current_dir(&working_dir);
        cmd.envs(&task.env);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Process group isolation on Unix so child trees can be cancelled cleanly
        #[cfg(unix)]
        cmd.process_group(0);

        let mut child = cmd.spawn().map_err(|e| {
            format!(
                "Failed to spawn task '{}' ({}): {}",
                task.display_name, cmd_name, e
            )
        })?;

        let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel::<()>();
        {
            let mut handles = self.abort_handles.lock().await;
            handles.insert(execution_id, cancel_tx);
        }

        {
            let mut execs = self.executions.lock().await;
            execs.insert(execution_id, execution.clone());
        }

        let _ = event_tx
            .send(TaskRunnerEvent::Started {
                execution_id,
                task_id: task.id.clone(),
                task_name: task.display_name.clone(),
            })
            .await;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let execs_ref = Arc::clone(&self.executions);
        let abort_handles_ref = Arc::clone(&self.abort_handles);
        let task_timeout = task
            .timeout_secs
            .map(Duration::from_secs)
            .unwrap_or(self.default_timeout);

        tokio::spawn(async move {
            let mut stdout_lines = stdout.map(|s| BufReader::new(s).lines());
            let mut stderr_lines = stderr.map(|s| BufReader::new(s).lines());

            let run_future = async {
                loop {
                    tokio::select! {
                        line_res = async {
                            if let Some(ref mut st) = stdout_lines {
                                st.next_line().await
                            } else {
                                std::future::pending().await
                            }
                        } => {
                            match line_res {
                                Ok(Some(line)) => {
                                    {
                                        let mut execs = execs_ref.lock().await;
                                        if let Some(exec) = execs.get_mut(&execution_id) {
                                            exec.append_line(TaskOutputStream::Stdout, line.clone());
                                        }
                                    }
                                    let _ = event_tx.send(TaskRunnerEvent::Line {
                                        execution_id,
                                        stream: TaskOutputStream::Stdout,
                                        text: line,
                                    }).await;
                                }
                                _ => {
                                    stdout_lines = None;
                                    if stderr_lines.is_none() {
                                        break;
                                    }
                                }
                            }
                        }
                        line_res = async {
                            if let Some(ref mut se) = stderr_lines {
                                se.next_line().await
                            } else {
                                std::future::pending().await
                            }
                        } => {
                            match line_res {
                                Ok(Some(line)) => {
                                    {
                                        let mut execs = execs_ref.lock().await;
                                        if let Some(exec) = execs.get_mut(&execution_id) {
                                            exec.append_line(TaskOutputStream::Stderr, line.clone());
                                        }
                                    }
                                    let _ = event_tx.send(TaskRunnerEvent::Line {
                                        execution_id,
                                        stream: TaskOutputStream::Stderr,
                                        text: line,
                                    }).await;
                                }
                                _ => {
                                    stderr_lines = None;
                                    if stdout_lines.is_none() {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }

                child.wait().await
            };

            let (final_status, exit_code) = tokio::select! {
                res = timeout(task_timeout, run_future) => {
                    match res {
                        Ok(Ok(status)) => {
                            let code = status.code().unwrap_or(if status.success() { 0 } else { 1 });
                            let st = if status.success() {
                                TaskStatus::Completed(code)
                            } else {
                                TaskStatus::Failed(code)
                            };
                            (st, Some(code))
                        }
                        Ok(Err(_)) => {
                            (TaskStatus::Failed(1), Some(1))
                        }
                        Err(_) => {
                            // Timed out
                            #[cfg(unix)]
                            if let Some(pid) = child.id() {
                                unsafe {
                                    libc::kill(-(pid as i32), libc::SIGKILL);
                                }
                            }
                            let _ = child.kill().await;
                            (TaskStatus::TimedOut, None)
                        }
                    }
                }
                _ = &mut cancel_rx => {
                    // Cancelled by user
                    #[cfg(unix)]
                    if let Some(pid) = child.id() {
                        unsafe {
                            libc::kill(-(pid as i32), libc::SIGTERM);
                        }
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        unsafe {
                            libc::kill(-(pid as i32), libc::SIGKILL);
                        }
                    }
                    let _ = child.kill().await;
                    (TaskStatus::Cancelled, None)
                }
            };

            let duration_ms;
            {
                let mut execs = execs_ref.lock().await;
                if let Some(exec) = execs.get_mut(&execution_id) {
                    exec.finish(final_status.clone(), exit_code);
                    duration_ms = exec.duration_ms;
                } else {
                    duration_ms = 0;
                }
            }

            {
                let mut handles = abort_handles_ref.lock().await;
                handles.remove(&execution_id);
            }

            let _ = event_tx
                .send(TaskRunnerEvent::Finished {
                    execution_id,
                    status: final_status,
                    exit_code,
                    duration_ms,
                })
                .await;
        });

        Ok(execution_id)
    }

    /// Requests safe cancellation of an active task execution.
    pub async fn cancel_task(&self, execution_id: u64) -> bool {
        let mut handles = self.abort_handles.lock().await;
        if let Some(sender) = handles.remove(&execution_id) {
            let _ = sender.send(());
            true
        } else {
            false
        }
    }

    /// Returns a snapshot of an execution record.
    pub async fn get_execution(&self, execution_id: u64) -> Option<TaskExecution> {
        let execs = self.executions.lock().await;
        execs.get(&execution_id).cloned()
    }

    /// Returns all execution records in order of ID.
    pub async fn list_executions(&self) -> Vec<TaskExecution> {
        let execs = self.executions.lock().await;
        let mut list: Vec<_> = execs.values().cloned().collect();
        list.sort_by_key(|e| e.execution_id);
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_task_runner_echo_success() {
        let runner = TaskRunner::new(100, Duration::from_secs(5));
        let dir = tempdir().unwrap();
        let (tx, mut rx) = mpsc::channel(32);

        let task = TaskDefinition {
            id: "echo.test".into(),
            display_name: "Echo Test".into(),
            description: String::new(),
            command: "echo".into(),
            args: vec!["Hello TermExplorer Workflow".into()],
            working_dir: None,
            env: HashMap::new(),
            timeout_secs: Some(5),
            depends_on: Vec::new(),
            category: crate::workflow::tasks::TaskCategory::Custom,
            output_mode: crate::workflow::tasks::TaskOutputMode::NonInteractive,
            is_shell: false,
        };

        let var_ctx = VariableContext::default();
        let exec_id = runner
            .spawn_task(&task, dir.path(), &var_ctx, tx)
            .await
            .unwrap();

        let mut lines = Vec::new();
        while let Some(evt) = rx.recv().await {
            match evt {
                TaskRunnerEvent::Line { text, .. } => lines.push(text),
                TaskRunnerEvent::Finished { status, .. } => {
                    assert_eq!(status, TaskStatus::Completed(0));
                    break;
                }
                _ => {}
            }
        }

        assert_eq!(lines, vec!["Hello TermExplorer Workflow"]);
        let exec = runner.get_execution(exec_id).await.unwrap();
        assert_eq!(exec.status, TaskStatus::Completed(0));
    }

    #[tokio::test]
    async fn test_task_runner_cancellation() {
        let runner = TaskRunner::new(100, Duration::from_secs(10));
        let dir = tempdir().unwrap();
        let (tx, mut rx) = mpsc::channel(32);

        let task = TaskDefinition {
            id: "sleep.test".into(),
            display_name: "Sleep Test".into(),
            description: String::new(),
            command: "sleep".into(),
            args: vec!["10".into()],
            working_dir: None,
            env: HashMap::new(),
            timeout_secs: Some(10),
            depends_on: Vec::new(),
            category: crate::workflow::tasks::TaskCategory::Custom,
            output_mode: crate::workflow::tasks::TaskOutputMode::NonInteractive,
            is_shell: false,
        };

        let var_ctx = VariableContext::default();
        let exec_id = runner
            .spawn_task(&task, dir.path(), &var_ctx, tx)
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(runner.cancel_task(exec_id).await);

        while let Some(evt) = rx.recv().await {
            if let TaskRunnerEvent::Finished { status, .. } = evt {
                assert_eq!(status, TaskStatus::Cancelled);
                break;
            }
        }
    }
}
