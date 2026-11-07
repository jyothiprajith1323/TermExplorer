//! Icon ecosystem — Catppuccin Nerd Font, Unicode, ASCII, None, and custom mappings.

pub mod glyphs;
pub mod manager;
pub mod mappings;
pub mod pack;

pub use glyphs::GlyphSet;
pub use manager::{CustomIconRule, CustomIconsConfig, IconManager};
pub use pack::IconPack;
