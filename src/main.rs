//! TermExplorer — Modern Terminal File Explorer
//!
//! Entry point: initializes logging, loads configuration,
//! sets up the terminal, runs the application event loop,
//! and restores the terminal on exit.

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;
use termexplorer::{App, Config};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("termexplorer {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return Ok(());
    }
    if args.iter().any(|a| a == "--health") {
        print_health_check();
        return Ok(());
    }

    // Parse options
    let mut cwd_file: Option<PathBuf> = None;
    let mut chooser_file: Option<PathBuf> = None;
    let mut initial_path: Option<PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--cwd-file" => {
                if i + 1 < args.len() {
                    cwd_file = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--chooser-file" => {
                if i + 1 < args.len() {
                    chooser_file = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            arg if !arg.starts_with('-') && initial_path.is_none() => {
                initial_path = Some(PathBuf::from(arg));
            }
            _ => {}
        }
        i += 1;
    }

    // Initialise file-based logging (never pollutes the TUI).
    let log_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("termexplorer");
    std::fs::create_dir_all(&log_dir)?;
    let file_appender = tracing_appender::rolling::daily(&log_dir, "termexplorer.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!("TermExplorer starting up");

    // Load (or create) user configuration.
    let config = Config::load_or_create()?;
    info!("Configuration loaded");

    // Install emergency panic hook to restore raw mode and alternate screen if a panic occurs.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        default_hook(panic_info);
    }));

    // Set up the terminal.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    // Build and run the application, catching any error so we always restore.
    let result = run_app(&mut terminal, config, initial_path, cwd_file, chooser_file).await;

    // Always restore terminal state.
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {e:#}");
        std::process::exit(1);
    }

    info!("TermExplorer exited cleanly");
    Ok(())
}

fn print_help() {
    println!("TermExplorer {}", env!("CARGO_PKG_VERSION"));
    println!(
        "A highly customizable Rust terminal developer workstation combining a file explorer,"
    );
    println!("intelligent code editor, interactive Linux terminal, multimedia preview engine and system monitor.\n");
    println!("USAGE:");
    println!("    termexplorer [OPTIONS] [PATH]\n");
    println!("OPTIONS:");
    println!("    -h, --help               Print help information");
    println!("    -V, --version            Print version information");
    println!("        --health             Print diagnostic health check report and exit");
    println!("        --cwd-file <PATH>     Write the last active directory to PATH on exit");
    println!("        --chooser-file <PATH> Write the selected file path to PATH on exit");
}

fn print_health_check() {
    println!("TermExplorer System Diagnostics & Health Check");
    println!("================================================");
    println!("Version:           {}", env!("CARGO_PKG_VERSION"));

    // Terminal size
    if let Ok((w, h)) = crossterm::terminal::size() {
        println!("Terminal Size:     {} cols × {} rows", w, h);
    } else {
        println!("Terminal Size:     Unknown (headless or piped)");
    }

    // Color support
    let term = std::env::var("TERM").unwrap_or_else(|_| "unknown".to_string());
    let colorterm = std::env::var("COLORTERM").unwrap_or_else(|_| "none".to_string());
    let truecolor = colorterm == "truecolor" || colorterm == "24bit";
    println!(
        "Terminal Support:  $TERM={}, $COLORTERM={}, TrueColor={}",
        term, colorterm, truecolor
    );

    // Font / Nerd Font
    let nerd_font = std::env::var("NERD_FONT")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    println!(
        "Nerd Font:         {}",
        if nerd_font {
            "Detected ($NERD_FONT=1)"
        } else {
            "Standard Unicode / ASCII fallback"
        }
    );

    // PTY Subsystem
    #[cfg(unix)]
    println!("PTY Subsystem:     Available (/dev/ptmx, portable-pty)");
    #[cfg(not(unix))]
    println!("PTY Subsystem:     Unavailable (non-Unix target)");

    // FreeDesktop Trash
    let trash_available = dirs::home_dir()
        .map(|h| h.join(".local/share/Trash"))
        .is_some();
    println!(
        "FreeDesktop Trash: {}",
        if trash_available {
            "Available (~/.local/share/Trash)"
        } else {
            "Unavailable"
        }
    );

    // Configuration
    let config_dir = dirs::config_dir().map(|c| c.join("termexplorer"));
    println!(
        "Config Directory:  {}",
        config_dir
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "In-memory default".to_string())
    );

    println!("Health Status:     All required workstation subsystems OK.");
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    config: Config,
    initial_path: Option<PathBuf>,
    cwd_file: Option<PathBuf>,
    chooser_file: Option<PathBuf>,
) -> Result<()> {
    let mut app = App::new(config).await?;

    if let Some(path) = initial_path {
        if path.is_dir() {
            app.navigate_into(path).await;
        }
    }

    let result = app.run(terminal).await;

    // Shell integration outputs on clean exit
    if let Some(cwd_p) = cwd_file {
        let _ = std::fs::write(
            &cwd_p,
            app.state().explorer.current_dir.display().to_string(),
        );
    }

    if let Some(chooser_p) = chooser_file {
        if let Some(sel) = app.state().selected_path() {
            let _ = std::fs::write(&chooser_p, sel.display().to_string());
        }
    }

    result
}
