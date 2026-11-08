//! Typography and font detection module.

pub mod config;
pub mod detection;
pub mod installer;
pub mod unicode;

pub use config::{get_terminal_guides, TerminalGuide, FONT_EXPLANATION};
pub use detection::FontStatus;
pub use installer::{install_script_content, user_fonts_dir};
pub use unicode::*;
