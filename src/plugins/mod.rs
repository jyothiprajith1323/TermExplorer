//! Plugin ecosystem — WebAssembly sandboxed plugins, permissions, manifest, and manager.

pub mod manager;
pub mod manifest;
pub mod permissions;
pub mod sandbox;

pub use manager::{LoadedPlugin, PluginManager, PluginState};
pub use manifest::PluginManifest;
pub use permissions::PluginPermission;
pub use sandbox::{HostState, PluginInstance, DEFAULT_FUEL_LIMIT};
