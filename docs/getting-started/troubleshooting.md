# Troubleshooting Knowledge Base

This guide outlines common diagnostic procedures and remedies for TermExplorer.

---

## 1. Startup & Environment Issues

### Symptom: Application exits immediately with raw mode error
- **Cause**: The terminal stdout or stdin is not connected to an interactive TTY (e.g. running in a non-interactive CI script or piped pipe).
- **Diagnosis**: Run `test -t 0 && echo "TTY" || echo "Not a TTY"`.
- **Solution**: TermExplorer requires a standard interactive terminal device (`/dev/tty`). For automated scripting, use CLI flags like `--help` or `--health`.

### Symptom: `PTY initialization failed` or terminal drawer cannot open
- **Cause**: Inability to allocate pseudo-terminal master device `/dev/ptmx` or permissions restriction.
- **Diagnosis**: Check permissions on `/dev/ptmx`: `ls -l /dev/ptmx`. Inside containers (Docker/Podman), ensure `--privileged` or appropriate mount options (`/dev/pts`) are provided.
- **Solution**: Ensure your user belongs to the `tty` group or that container execution mounts `/dev/pts`.

---

## 2. Display, Color, and Glyph Issues

### Symptom: Colors appear washed out, incorrect, or monochrome
- **Cause**: Host terminal emulator does not advertise 24-bit TrueColor capability.
- **Diagnosis**: Run `termexplorer --health`. Check the values of `$COLORTERM` and `$TERM`.
- **Solution**: Export TrueColor variable in your shell profile (`~/.bashrc` or `~/.zshrc`):
  ```bash
  export COLORTERM=truecolor
  ```
  Ensure your terminal emulator (e.g. Alacritty, Ghostty, WezTerm, Kitty) supports direct color.

### Symptom: Nerd Font icon glyphs display as empty squares or question marks
- **Cause**: Active terminal font does not contain Nerd Font patches, or font fallback is misconfigured.
- **Diagnosis**: Open **Settings Studio** (`F10`) -> **General** -> check **Nerd Fonts**.
- **Solution**:
  1. Install a patched Nerd Font (e.g. *JetBrainsMono Nerd Font*).
  2. Set this font as the primary font in your external terminal emulator settings.
  3. Alternatively, toggle off `Nerd Fonts` in TermExplorer's Settings Studio to use standard ASCII/Unicode text glyphs.

---

## 3. Configuration & Recovery

### Symptom: `Corrupted configuration file detected` warning banner on startup
- **Cause**: `~/.config/termexplorer/config.toml` contains syntax errors or invalid TOML tables.
- **Diagnosis**: TermExplorer automatically archives the broken configuration as `config.toml.corrupted.<timestamp>` and loads factory defaults.
- **Solution**: Inspect the archived file to review custom edits, fix syntax errors, and restore required keys using `ConfigMigrator`.

### Symptom: Settings fail to persist upon exit
- **Cause**: Write permissions error in `~/.config/termexplorer/` or disk quota exhaustion.
- **Diagnosis**: Verify directory ownership:
  ```bash
  ls -ld ~/.config/termexplorer
  ```
- **Solution**: Ensure your user owns the directory:
  ```bash
  sudo chown -R $USER:$USER ~/.config/termexplorer
  ```

---

## 4. Performance & High Resource Usage

### Symptom: Slow rendering when scrolling large source files
- **Cause**: Syntax highlighting complexity or huge continuous lines (>10,000 chars).
- **Diagnosis**: TermExplorer uses viewport-bounded tokenization (`syntect`), but deeply nested structures or minified JavaScript files may tax parsing.
- **Solution**: In **Settings Studio** -> **Editor**, toggle `Syntax Highlight` off for minified assets, or adjust `tab_width` and `word_wrap`.

---

## 5. Log Files & Crash Diagnostics

TermExplorer writes execution logs to a daily rotating file outside the TUI buffer:
- **Log Path**: `~/.local/share/termexplorer/termexplorer.log` (or `$XDG_DATA_HOME/termexplorer/termexplorer.log`)
- **Inspection Command**:
  ```bash
  tail -f ~/.local/share/termexplorer/termexplorer.log
  ```
If an unhandled panic occurs, TermExplorer's emergency panic hook safely restores raw mode, leaves the alternate screen, and prints a backtrace.
