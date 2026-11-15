//! Shell and embedded PTY terminal subsystem.

pub mod detection;
pub mod input;
pub mod manager;
pub mod pty;
pub mod session;
pub mod terminal_screen;

pub use detection::{ShellInfo, TerminalCapabilities, ToolIntegration};
pub use manager::TerminalSessionManager;
pub use session::TerminalSession;
pub use terminal_screen::TerminalScreen;
