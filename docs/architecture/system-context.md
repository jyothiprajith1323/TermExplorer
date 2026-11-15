# System Context & Process Boundaries

The diagram below illustrates TermExplorer's operational context, runtime boundaries, and interactions with host system interfaces:

```mermaid
flowchart TD
    subgraph Host["Host Operating System (Linux / WSL2)"]
        User(["Developer / User"])
        Emulator["Terminal Emulator\n(Ghostty, Alacritty, Kitty, WezTerm)"]
        FS[("Host Filesystem\n(EXT4, Btrfs, ZFS, NTFS)")]
        PTMX["/dev/ptmx\nPseudo-Terminal Subsystem"]
        GitBin["Git Binary\n(CLI & config)"]
        LSPBin["LSP Server\n(e.g., rust-analyzer)"]
        Helpers["Optional Preview Helpers\n(ffmpeg, pdftotext, sqlite3)"]
    end

    subgraph AppProcess["TermExplorer Process (PID)"]
        direction TB
        
        subgraph TUI["TUI & Event Layer"]
            Crossterm["Crossterm Raw Mode\n& Event Stream"]
            Ratatui["Ratatui Terminal Backend\n& Double Buffer"]
            Dispatcher["Input Dispatcher\n& Focus Manager"]
        end

        subgraph Core["Workstation Core State"]
            AppState["App Engine State\n(Explorer, Preview, Editor)"]
            ConfigStore["Config Manager\n(Schema v2 / Atomic Save)"]
            KeyRegistry["Keybinding Registry\n& Conflict Detector"]
        end

        subgraph Engines["Asynchronous Processing Subsystems"]
            FSEngine["Filesystem Engine\n(Chunked Scanner / Ledger)"]
            EditorEngine["Editor Engine\n(Ropey / Undo / Autocomplete)"]
            PreviewEngine["Preview Engine\n(Sniffer / LRU Cache)"]
            SearchEngine["Search Engine\n(ignore crate / ripgrep)"]
            GitEngine["Git Workbench\n(Diff / Branch / Commit)"]
            WorkflowEngine["Task Runner\n(DAG Scheduler / Trust Store)"]
            PTYEngine["Terminal Session Manager\n(portable-pty stream)"]
        end

        subgraph Sandbox["WebAssembly Plugin Sandbox"]
            Wasmi["Wasmi Interpreter\n(16 MiB Memory / 5M Fuel)"]
            Perms["Capability Evaluator\n(Default-Deny)"]
        end
    end

    subgraph ShellChild["Child Shell Process"]
        ActiveShell["Interactive Login Shell\n(zsh, bash, fish)"]
    end

    User -->|Keyboard / Mouse| Emulator
    Emulator <-->|ANSI / VT100 / Escape Sequences| Crossterm
    Crossterm --> Dispatcher
    Dispatcher --> AppState
    AppState --> Ratatui
    Ratatui -->|Render Frames| Emulator

    AppState <--> FSEngine
    AppState <--> EditorEngine
    AppState <--> PreviewEngine
    AppState <--> SearchEngine
    AppState <--> GitEngine
    AppState <--> WorkflowEngine
    AppState <--> PTYEngine
    AppState <--> Sandbox

    FSEngine <-->|POSIX I/O / sync_all| FS
    EditorEngine <-->|Atomic Save| FS
    SearchEngine <-->|Read-only Walk| FS
    PreviewEngine <-->|Read-only Probe| FS
    PreviewEngine -.->|Optional CLI| Helpers

    PTYEngine <-->|ioctl & Master FD| PTMX
    PTMX <-->|Slave FD| ActiveShell

    GitEngine <-->|Query Status & Commit| GitBin
    EditorEngine -.->|JSON-RPC Stdio| LSPBin
    WorkflowEngine -->|Supervised Subprocess| FS

    Sandbox <-->|Host Linker ('env')| Perms
    Perms <-->|Controlled Metadata| AppState
```
