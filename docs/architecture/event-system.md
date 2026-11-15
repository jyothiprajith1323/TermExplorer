# Event System & Input Routing Architecture

TermExplorer implements a hierarchical modal event dispatching system designed to avoid shortcut collisions and input lockups.

---

## 1. Event Ingestion Pipeline

```mermaid
flowchart TD
    RawInput["Crossterm Event Stream\n(crossterm::event::Event)"] --> Filter{Event Type?}
    
    Filter -->|Resize| ResizeHandler["Handle Terminal Resize\n(TIOCSWINSZ to PTY, Redraw Layout)"]
    Filter -->|Mouse| MouseHandler["Hit-Test Coordinate Resolver\n(Click to Focus, Scroll to Viewport)"]
    Filter -->|Key| ModalCheck{Active Modal Open?}

    ModalCheck -->|Yes: Welcome Surface| OnboardingDispatch["ui::onboarding::handle_key()"]
    ModalCheck -->|Yes: Settings Studio| SettingsDispatch["ui::settings::handle_key()"]
    ModalCheck -->|Yes: Command Palette| PaletteDispatch["ui::command_palette::handle_key()"]
    ModalCheck -->|Yes: Git Workbench| GitDispatch["ui::git_panel::handle_key()"]
    ModalCheck -->|Yes: Search Modal| SearchDispatch["ui::search_modal::handle_key()"]
    ModalCheck -->|Yes: Help Overlay| HelpDispatch["ui::help::handle_key()"]
    ModalCheck -->|Yes: Confirmation Modal| ConfirmDispatch["Handle Yes/No Confirmation"]

    ModalCheck -->|No: Main Workstation| FocusRouter{Active Panel Focus?}
    FocusRouter -->|Explorer| ExplorerKeys["Directory Navigation & File Ops (j, k, r, d)"]
    FocusRouter -->|Preview| PreviewKeys["Inspector Scrolling & Tab Cycling (1-4, j, k)"]
    FocusRouter -->|Editor| EditorKeys["Rope Buffer Text Editing & Autocomplete"]
    FocusRouter -->|Terminal| TerminalKeys["Pass-through directly to PTY Master FD"]
```

---

## 2. Modal Hierarchy and Dispatch Priority

Modals in TermExplorer always take strict precedence over background panel keys:
1. **Critical Safety Confirmation**: Discard changes, permanent deletion, trust prompts.
2. **First-Run Welcome Surface**: Blocks underlying panel navigation until dismissed.
3. **Command Palette & Search**: Consumes alphanumeric input for fuzzy filtering.
4. **Settings Studio**: Consumes column navigation and value adjustment keys.
5. **Git Workbench**: Dedicated staging, commit, and diff inspection.
6. **Active Focus Panel**: Explorer, Preview, Editor, or Terminal.

Pressing `Esc` universally closes the topmost modal and returns keyboard focus to the File Explorer.

---

## 3. Mouse Hit-Testing & Interaction

TermExplorer enables mouse capture (`EnableMouseCapture`) for intuitive mouse interactions:
- **Click to Focus**: Clicking anywhere within a panel's bounding box moves focus directly to that panel.
- **Mouse Wheel Scrolling**: Scrolling over the Explorer lists scrolls directory entries; scrolling over the Preview pane scrolls preview content; scrolling over the editor adjusts text lines.
- **Tab Clicks**: Clicking on Preview tabs (`[Preview]`, `[Information]`, `[Hex]`, `[Actions]`) switches tabs immediately.
