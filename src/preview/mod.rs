//! Universal Multimedia & Document Preview subsystem.

pub mod archive;
pub mod audio;
pub mod cache;
pub mod cancellation;
pub mod capabilities;
pub mod data;
pub mod database;
pub mod detector;
pub mod hex;
pub mod image;
pub mod manager;
pub mod markdown;
pub mod metadata;
pub mod office;
pub mod pdf;
pub mod registry;
pub mod rendering;
pub mod sanitizer;
pub mod source;
pub mod video;

pub use cache::{DiskPreviewCache, PreviewLruCache};
pub use cancellation::MonotonicCancellation;
pub use capabilities::{
    detect_graphics_protocol, render_halfblocks_from_image, TerminalGraphicsProtocol,
};
pub use detector::{detect_file_format, FileCategory, FileFormatInfo};
pub use manager::{FullPreviewData, PreviewAction, PreviewManager, PreviewTab};
pub use metadata::DetailedMetadata;
pub use registry::CapabilityRegistry;
pub use sanitizer::{sanitize_preview_line, sanitize_preview_text};
