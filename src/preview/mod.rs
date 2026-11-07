//! Universal Multimedia & Document Preview subsystem.

pub mod archive;
pub mod audio;
pub mod cache;
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
pub mod source;
pub mod video;

pub use detector::{detect_file_format, FileCategory, FileFormatInfo};
pub use manager::{FullPreviewData, PreviewAction, PreviewManager, PreviewTab};
pub use metadata::DetailedMetadata;
pub use registry::CapabilityRegistry;
