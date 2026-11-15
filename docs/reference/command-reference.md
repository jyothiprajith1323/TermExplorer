# Command Palette Action Reference

This reference lists the actions registered in TermExplorer's Command Palette (`src/workflow/command.rs`).

---

## 1. Opening the Command Palette

- Press `Ctrl+Shift+P` (or `Ctrl+P`).
- Type any keyword to fuzzy-search across commands.
- Press `Enter` to execute the highlighted action.
- Press `Esc` to cancel.

---

## 2. Command Registry

| Command ID | Display Name | Category | Default Shortcut | Description |
|---|---|---|---|---|
| `git.dashboard` | Git Workbench | Git | `Ctrl+G` / `g` | Open Git status, diff viewer, and branch manager |
| `task.run_tests` | Run Project Tests | Workflow | — | Execute default test suite (`cargo test`, `npm test`, `pytest`) |
| `task.build` | Run Project Build | Workflow | — | Execute default build task (`cargo build`, `npm run build`) |
| `file.new` | New File | File | `a` | Create new empty file in current directory |
| `file.new_dir` | New Directory | File | `A` | Create new subdirectory in active folder |
| `file.rename` | Rename Selected Item | File | `r` | Rename currently highlighted file or folder |
| `file.delete` | Delete Selected Item | File | `d` | Move selected item to FreeDesktop Trash |
| `search.files` | Find Files | Search | `Ctrl+F` / `/` | Open fuzzy workspace filename search |
| `search.content` | Search in Files | Search | `Ctrl+Shift+F` | Open ripgrep-style full-text content grep |
| `panel.terminal` | Toggle Terminal Drawer | View | `` ` `` / `Ctrl-` ` `` | Open or close bottom interactive PTY shell drawer |
| `panel.dual_pane`| Toggle Dual Pane | View | `F7` | Enable side-by-side secondary file manager |
| `panel.monitor` | Toggle System Monitor | View | `F6` | Show or hide CPU, Memory, and Disk telemetry |
| `settings.open` | Open Settings Studio | Preferences | `Ctrl+,` / `F10` | Configure themes, layout, and workstation preferences |
| `settings.onboarding`| Start Guided Setup Wizard| Help | — | Launch first-run onboarding wizard |
| `app.help` | Keyboard Shortcuts Cheat-Sheet | Help | `F1` / `?` | Display shortcuts reference matrix |
| `app.quit` | Quit TermExplorer | Application | `Ctrl+Q` | Cleanly restore terminal and exit |
