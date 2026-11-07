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
use termexplorer::{App, Config};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialise file-based logging (never pollutes the TUI).
    let log_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
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

    // Set up the terminal.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    // Build and run the application, catching any error so we always restore.
    let result = run_app(&mut terminal, config).await;

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

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    config: Config,
) -> Result<()> {
    let mut app = App::new(config).await?;
    app.run(terminal).await
}
