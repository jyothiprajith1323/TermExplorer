//! TermExplorer library interface.

pub mod app;
pub mod commands;
pub mod completion;
pub mod config;
pub mod editor;
pub mod events;
pub mod filesystem;
pub mod fonts;
pub mod git;
pub mod icons;
pub mod monitor;
pub mod plugins;
pub mod preview;
pub mod settings;
pub mod shell;
pub mod state;
pub mod theme;
pub mod ui;
pub mod workspace;

pub use app::App;
pub use config::Config;
pub use state::AppState;
pub use theme::Theme;
