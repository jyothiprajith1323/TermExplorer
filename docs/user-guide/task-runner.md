# Project Workflow Engine & Task Runner

TermExplorer includes a project-aware Developer Workflow Engine that detects technology stacks, configures default build/test tasks, schedules task dependency graphs (DAG), and manages workspace trust boundaries.

---

## 1. Project Tech Stack Auto-Detection

When opening a directory, `detect_project()` identifies the technologies present:

| Detected Marker | Tech Stack | Suggested Tasks |
|---|---|---|
| `Cargo.toml` | **Rust / Cargo** | `cargo check`, `cargo test`, `cargo clippy`, `cargo build` |
| `package.json` | **Node.js / npm** | `npm install`, `npm test`, `npm run build`, `npm dev` |
| `pyproject.toml` / `requirements.txt` | **Python** | `pytest`, `ruff check`, `black --check`, `python main.py` |
| `go.mod` | **Go** | `go test ./...`, `go build`, `go vet ./...` |
| `Makefile` | **Make** | `make`, `make test`, `make clean` |

---

## 2. Defining Custom Tasks (`tasks.toml`)

You can define project-specific tasks by creating `.termexplorer/tasks.toml` in your project root:

```toml
[[tasks]]
id = "build.release"
display_name = "Release Build"
description = "Compile optimized release binary"
command = "cargo"
args = ["build", "--release"]
category = "Build"
output_mode = "NonInteractive"
timeout_secs = 300

[[tasks]]
id = "lint.all"
display_name = "Full Linter"
description = "Run clippy with warnings denied"
command = "cargo"
args = ["clippy", "--all-targets", "--", "-D", "warnings"]
category = "Lint"
depends_on = ["build.release"]
```

### Supported Variables
Tasks support variable substitution:
- `${workspaceRoot}`: Path to the workspace root directory
- `${file}`: Path to the currently active file
- `${fileBasename}`: Filename with extension (e.g. `main.rs`)
- `${fileDirname}`: Directory containing the active file

---

## 3. Running Tasks & Viewing Output

- Open the Command Palette (`Ctrl+Shift+P`), type `run task` or the task name (e.g. `cargo test`), and press `Enter`.
- Alternatively, open the **Task Viewer** modal to inspect running and completed tasks.
- Task execution streams stdout and stderr in real-time into the output viewer.
- Cancel any running task at any time with `Ctrl+C` or by selecting **Cancel Task**.
- Orphan process cleanup ensures that child processes are cleanly terminated with `SIGTERM` followed by `SIGKILL` if unresponsive.

---

## 4. Workspace Trust & Security Boundary

Executing arbitrary commands found in repository config files is an inherent security risk. TermExplorer enforces a strict **Workspace Trust** boundary:
- When opening an untrusted project folder for the first time, tasks will **not** execute automatically.
- TermExplorer prompts: `Trust this workspace to execute tasks? [y/N]`.
- Trusted workspace decisions are stored securely in `~/.local/share/termexplorer/trusted_workspaces.json`.
- Untrusted workspaces run in restricted read-only inspection mode.
