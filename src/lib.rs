//! TermExplorer library interface.

pub mod app;
pub mod config;
pub mod editor;
pub mod events;
pub mod filesystem;
pub mod monitor;
pub mod shell;
pub mod state;
pub mod theme;
pub mod ui;

pub use app::App;
pub use config::Config;
pub use state::AppState;
pub use theme::Theme;
