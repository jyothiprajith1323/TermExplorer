# Embedded Terminal Drawer & PTY Engine

TermExplorer integrates an interactive pseudo-terminal (PTY) shell directly into the workstation layout, providing a terminal experience without requiring external tmux panes or window manager switching.

---

## 1. Opening & Toggling the Terminal Drawer

- **Toggle Terminal Drawer**: Press `` ` `` (backtick) or `Ctrl+` ` ``.
- **Default Geometry**: When opened, the drawer appears at the bottom of the workstation occupying **15%** of the height.
- **Background Execution**: Hiding the drawer does **not** terminate child processes. If you run `cargo build` or a long server script and press `` ` ``, the task continues running in the background.

---

## 2. Interactive Shell Behavior

The embedded terminal drawer runs a real Unix PTY backed by `portable-pty`:
- Spawns the user's default `$SHELL` (or `/bin/zsh`, `/bin/bash`, `/usr/bin/fish`).
- Launches as an interactive login shell (`-l`) to source user configuration profiles (`~/.zshrc`, `~/.bashrc`), setting up `$PATH`, environment variables, and shell aliases.
- Non-blocking channel I/O ensures high-throughput output (e.g. fast compilation output or `cat /dev/urandom`) will not freeze or drop frames in the Ratatui TUI.

---

## 3. Resizing the Drawer

You can adjust the height of the terminal drawer at any time:
- Increase drawer height (+2 rows): `Ctrl+Up` or `Alt+}`
- Decrease drawer height (-2 rows): `Ctrl+Down` or `Alt+{`
- Maximize drawer: `Alt+F6` or `Ctrl+Shift+T` (expands drawer to ~80% workstation height)

Height limits are constrained between **10%** and **40%** during standard resizing, and terminal dimensions are communicated to the child shell via `TIOCSWINSZ` ioctls so full-screen curses tools (`htop`, `less`, `vim`) redraw cleanly.

---

## 4. Signal Handling & Keyboard Interaction

When the terminal drawer holds focus:
- Standard control signals are passed directly to the child process:
  - `Ctrl+C`: Sends `SIGINT` to interrupt the foreground process.
  - `Ctrl+D`: Sends `EOF` (exits shell if at prompt).
  - `Ctrl+L`: Clears the screen buffer.
  - `Ctrl+Z`: Sends `SIGTSTP`.
- Navigation intercept keys:
  - `Tab` / `Shift+Tab`: Switches keyboard focus to the File Explorer or Preview pane.
  - `` ` `` / `Ctrl-` ` ``: Hides the terminal drawer.

---

## 5. Terminal Capabilities & Diagnostics

TermExplorer actively probes the host terminal to configure PTY attributes:
- **TrueColor**: Detects 24-bit direct color support via `COLORTERM=truecolor` or modern terminal identifiers (`ghostty`, `alacritty`, `kitty`, `wezterm`).
- **Unicode Support**: Inspects locale variables (`LANG`, `LC_ALL`) for UTF-8 encoding.
- **PTY Device**: Validates accessibility of `/dev/ptmx`.

View current terminal diagnostics in **Settings Studio** -> **Terminal** -> **Capabilities Diagnostics**, or run `termexplorer --health`.
