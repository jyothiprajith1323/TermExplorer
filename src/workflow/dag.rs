//! Directed Acyclic Graph (DAG) Task Dependency Engine.
//!
//! Computes topological execution order, detects circular dependencies,
//! and orchestrates ordered execution with fail-fast semantics.

use std::collections::{HashMap, HashSet};

use super::tasks::{TaskDefinition, TaskId};

/// Errors encountered while resolving a task dependency graph.
#[derive(Debug, PartialEq, Eq)]
pub enum DagError {
    CycleDetected(Vec<TaskId>),
    MissingDependency { task_id: TaskId, missing: TaskId },
    EmptyGraph,
}

impl std::fmt::Display for DagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CycleDetected(cycle) => {
                let ids: Vec<String> = cycle.iter().map(|t| t.0.clone()).collect();
                write!(f, "Dependency cycle detected: {}", ids.join(" -> "))
            }
            Self::MissingDependency { task_id, missing } => {
                write!(
                    f,
                    "Task '{}' depends on undefined task '{}'",
                    task_id, missing
                )
            }
            Self::EmptyGraph => write!(f, "Cannot schedule empty task graph"),
        }
    }
}

impl std::error::Error for DagError {}

/// Dependency DAG orchestrator for tasks.
#[derive(Debug, Clone)]
pub struct TaskDag {
    tasks: HashMap<TaskId, TaskDefinition>,
}

impl Default for TaskDag {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskDag {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }

    pub fn add_task(&mut self, task: TaskDefinition) {
        self.tasks.insert(task.id.clone(), task);
    }

    pub fn get(&self, id: &TaskId) -> Option<&TaskDefinition> {
        self.tasks.get(id)
    }

    /// Computes a valid topological execution order for the given target task ID
    /// and all its transitive dependencies.
    pub fn plan_execution(&self, target: &TaskId) -> Result<Vec<TaskId>, DagError> {
        if !self.tasks.contains_key(target) {
            return Err(DagError::MissingDependency {
                task_id: target.clone(),
                missing: target.clone(),
            });
        }

        let mut visited = HashSet::new();
        let mut in_stack = HashSet::new();
        let mut order = Vec::new();
        let mut cycle_path = Vec::new();

        self.dfs(
            target,
            &mut visited,
            &mut in_stack,
            &mut order,
            &mut cycle_path,
        )?;

        Ok(order)
    }

    fn dfs(
        &self,
        current: &TaskId,
        visited: &mut HashSet<TaskId>,
        in_stack: &mut HashSet<TaskId>,
        order: &mut Vec<TaskId>,
        cycle_path: &mut Vec<TaskId>,
    ) -> Result<(), DagError> {
        visited.insert(current.clone());
        in_stack.insert(current.clone());
        cycle_path.push(current.clone());

        let task = self.tasks.get(current).ok_or_else(|| {
            let last = cycle_path
                .iter()
                .rev()
                .nth(1)
                .cloned()
                .unwrap_or_else(|| current.clone());
            DagError::MissingDependency {
                task_id: last,
                missing: current.clone(),
            }
        })?;

        for dep in &task.depends_on {
            if in_stack.contains(dep) {
                let mut cycle = cycle_path.clone();
                cycle.push(dep.clone());
                return Err(DagError::CycleDetected(cycle));
            }

            if !visited.contains(dep) {
                self.dfs(dep, visited, in_stack, order, cycle_path)?;
            }
        }

        cycle_path.pop();
        in_stack.remove(current);
        order.push(current.clone());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::tasks::{TaskCategory, TaskOutputMode};

    fn dummy_task(id: &str, deps: Vec<&str>) -> TaskDefinition {
        TaskDefinition {
            id: id.into(),
            display_name: id.into(),
            description: String::new(),
            command: "echo".into(),
            args: Vec::new(),
            working_dir: None,
            env: HashMap::new(),
            timeout_secs: None,
            depends_on: deps.into_iter().map(Into::into).collect(),
            category: TaskCategory::Custom,
            output_mode: TaskOutputMode::NonInteractive,
            is_shell: false,
        }
    }

    #[test]
    fn test_valid_dag_topological_sort() {
        let mut dag = TaskDag::new();
        // A -> B -> C (C depends on B, B depends on A)
        dag.add_task(dummy_task("A", vec![]));
        dag.add_task(dummy_task("B", vec!["A"]));
        dag.add_task(dummy_task("C", vec!["B"]));

        let plan = dag.plan_execution(&"C".into()).unwrap();
        assert_eq!(
            plan,
            vec![TaskId::from("A"), TaskId::from("B"), TaskId::from("C")]
        );
    }

    #[test]
    fn test_dag_cycle_detection() {
        let mut dag = TaskDag::new();
        // A -> B -> A (cycle!)
        dag.add_task(dummy_task("A", vec!["B"]));
        dag.add_task(dummy_task("B", vec!["A"]));

        let result = dag.plan_execution(&"A".into());
        assert!(matches!(result, Err(DagError::CycleDetected(_))));
    }

    #[test]
    fn test_dag_missing_dependency() {
        let mut dag = TaskDag::new();
        dag.add_task(dummy_task("A", vec!["non_existent"]));

        let result = dag.plan_execution(&"A".into());
        assert!(matches!(result, Err(DagError::MissingDependency { .. })));
    }
}
