# Native Git Integration & Workbench

TermExplorer provides first-class native Git repository management directly inside the terminal workstation, balancing speed with visual clarity.

---

## 1. Opening the Git Workbench (`g` or `Ctrl+G`)

Press `g` (in File Explorer) or `Ctrl+G` (anywhere) to open the Git Workbench modal:
- **Active Branch Indicator**: Displays current branch name, upstream tracking status, and dirty state (`*`).
- **File Changes Overview**: Categorized list of:
  - Untracked files (`?`)
  - Modified working-tree files (`M`)
  - Staged changes ready for commit (`A` / `M`)
  - Deleted or renamed files (`D` / `R`)
- **Recent Commit History**: The 10 most recent commits showing short SHA, author, relative date, and commit summary.

---

## 2. Inspecting Diffs

Highlight any modified file in the Git Workbench:
- The right panel displays a unified diff with green additions (`+`) and red deletions (`-`).
- Supports syntax-highlighted diff hunks with line numbers.
- Unsaved editor buffers are protected: if you have unsaved edits in an open file, TermExplorer warns before performing operations that would overwrite working files.

---

## 3. Staging and Unstaging Files

- **Stage File**: Highlight an untracked or modified file and press `s`. The file is staged (`git add <file>`) and moves to the staged list.
- **Unstage File**: Highlight a staged file and press `u`. The file is unstaged (`git restore --staged <file>`).
- **Stage All**: Press `S` (Shift+S) to stage all modified and untracked files in the workspace.
- **Unstage All**: Press `U` (Shift+U) to unstage all currently staged changes.

---

## 4. Authoring Commits (`c`)

1. Stage the files you wish to commit.
2. Press `c` in the Git Workbench.
3. A commit message dialog opens. Type your commit message.
4. Press `Enter` (or `Ctrl+Enter`) to commit.
5. The commit is executed using `git commit -m "<message>"`, preserving user identity configured in `~/.gitconfig` (`user.name` and `user.email`).
6. The status panel updates immediately to reflect the new commit.

---

## 5. Branch Management

- **List Branches**: Press `b` inside the Git Workbench to view all local branches.
- **Switch Branch**: Select a branch from the list and press `Enter` (`git checkout <branch>` / `git switch <branch>`).
- **Create Branch**: Press `n` in branch mode, enter a new branch name, and press `Enter`.
