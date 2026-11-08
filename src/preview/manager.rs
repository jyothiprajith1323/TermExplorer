//! Central preview manager orchestrating background loading, caching, and tab switching.

use crate::preview::archive::ArchivePreview;
use crate::preview::audio::AudioPreview;
use crate::preview::cache::PreviewLruCache;
use crate::preview::data::DataPreview;
use crate::preview::database::DatabasePreview;
use crate::preview::detector::{detect_file_format, FileCategory, FileFormatInfo};
use crate::preview::hex::HexPreview;
use crate::preview::image::ImagePreview;
use crate::preview::markdown::MarkdownPreview;
use crate::preview::metadata::DetailedMetadata;
use crate::preview::office::OfficePreview;
use crate::preview::pdf::PdfPreview;
use crate::preview::source::SourcePreview;
use crate::preview::video::VideoPreview;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PreviewTab {
    #[default]
    Preview,
    Information,
    Hex,
    Actions,
}

#[derive(Debug, Clone)]
pub struct PreviewAction {
    pub label: &'static str,
    pub description: &'static str,
    pub command: &'static str,
}

#[derive(Debug, Clone)]
pub struct FullPreviewData {
    pub path: PathBuf,
    pub active_tab: PreviewTab,
    pub format_info: FileFormatInfo,
    pub metadata: Option<DetailedMetadata>,
    pub source: Option<SourcePreview>,
    pub markdown: Option<MarkdownPreview>,
    pub image: Option<ImagePreview>,
    pub video: Option<VideoPreview>,
    pub audio: Option<AudioPreview>,
    pub pdf: Option<PdfPreview>,
    pub office: Option<OfficePreview>,
    pub data: Option<DataPreview>,
    pub archive: Option<ArchivePreview>,
    pub hex: Option<HexPreview>,
    pub database: Option<DatabasePreview>,
    pub actions: Vec<PreviewAction>,
}

impl FullPreviewData {
    pub fn empty(path: PathBuf) -> Self {
        Self {
            path,
            active_tab: PreviewTab::Preview,
            format_info: FileFormatInfo {
                category: FileCategory::Other,
                mime_type: "application/octet-stream".to_string(),
                description: "File not found or inaccessible",
                is_binary: false,
            },
            metadata: None,
            source: None,
            markdown: None,
            image: None,
            video: None,
            audio: None,
            pdf: None,
            office: None,
            data: None,
            archive: None,
            hex: None,
            database: None,
            actions: Vec::new(),
        }
    }
}

pub struct PreviewManager {
    cache: PreviewLruCache<FullPreviewData>,
    max_preview_bytes: u64,
}

impl PreviewManager {
    pub fn new(max_preview_bytes: u64) -> Self {
        Self {
            cache: PreviewLruCache::new(64),
            max_preview_bytes,
        }
    }

    pub fn get_cached(&mut self, path: &Path, tab: PreviewTab) -> Option<FullPreviewData> {
        let modified = std::fs::metadata(path).and_then(|m| m.modified()).ok();
        if let Some(mut cached) = self.cache.get(path, modified) {
            cached.active_tab = tab;
            Some(cached)
        } else {
            None
        }
    }

    pub fn insert_cache(&mut self, path: PathBuf, data: FullPreviewData) {
        let modified = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
        self.cache.insert(path, modified, data);
    }

    /// Load file using LRU cache or synchronous generation.
    pub fn load_file(
        &mut self,
        path: &Path,
        tab: PreviewTab,
        preview_width: u32,
        preview_height: u32,
    ) -> FullPreviewData {
        if !path.exists() {
            self.cache.invalidate(path);
            let mut empty = FullPreviewData::empty(path.to_path_buf());
            empty.active_tab = tab;
            return empty;
        }

        let modified = std::fs::metadata(path).and_then(|m| m.modified()).ok();

        // Check LRU cache
        if let Some(mut cached) = self.cache.get(path, modified) {
            cached.active_tab = tab;
            return cached;
        }

        let full_data = Self::generate_preview_cancellable(
            path,
            tab,
            self.max_preview_bytes,
            preview_width,
            preview_height,
            None,
        )
        .unwrap_or_else(|| FullPreviewData::empty(path.to_path_buf()));

        self.cache
            .insert(path.to_path_buf(), modified, full_data.clone());
        full_data
    }

    /// Generate full preview data with monotonic cancellation checking.
    pub fn generate_preview_cancellable(
        path: &Path,
        tab: PreviewTab,
        max_preview_bytes: u64,
        preview_width: u32,
        preview_height: u32,
        cancel_token: Option<(&AtomicU64, u64)>,
    ) -> Option<FullPreviewData> {
        if !path.exists() {
            let mut empty = FullPreviewData::empty(path.to_path_buf());
            empty.active_tab = tab;
            return Some(empty);
        }

        let is_cancelled = || -> bool {
            if let Some((token, expected)) = cancel_token {
                token.load(Ordering::Relaxed) != expected
            } else {
                false
            }
        };

        if is_cancelled() {
            return None;
        }

        let format_info = detect_file_format(path);
        let metadata = DetailedMetadata::from_path(path);

        if is_cancelled() {
            return None;
        }

        let mut source = None;
        let mut markdown = None;
        let mut image = None;
        let mut video = None;
        let mut audio = None;
        let mut pdf = None;
        let mut office = None;
        let mut data = None;
        let mut archive = None;
        let mut database = None;

        // Populate appropriate preview based on detected category
        // Limit initial preview lines to 120 for fast syntax highlighting and rendering
        match format_info.category {
            FileCategory::SourceCode => {
                source = Some(SourcePreview::load(path, max_preview_bytes, 120));
            }
            FileCategory::Markdown => {
                markdown = Some(MarkdownPreview::load(path, 120));
            }
            FileCategory::Image => {
                image = Some(ImagePreview::load(path, preview_width, preview_height));
            }
            FileCategory::Video => {
                video = Some(VideoPreview::load(path, preview_width, preview_height));
            }
            FileCategory::Audio => {
                audio = Some(AudioPreview::load(path));
            }
            FileCategory::Pdf => {
                pdf = Some(PdfPreview::load(path, 150));
            }
            FileCategory::Office => {
                office = Some(OfficePreview::load(path, 150));
            }
            FileCategory::Data => {
                data = Some(DataPreview::load(path, 150));
            }
            FileCategory::Archive => {
                archive = Some(ArchivePreview::load(path, 200));
            }
            FileCategory::Database => {
                database = Some(DatabasePreview::load(path, 50));
            }
            FileCategory::BinaryHex | FileCategory::Other => {
                // Will populate hex below
            }
        }

        if is_cancelled() {
            return None;
        }

        // Always load Hex view for Hex tab or Binary files
        let hex = Some(HexPreview::load(path, 100));

        // Available actions for this file
        let mut actions = vec![
            PreviewAction {
                label: "Open with System Default",
                description: "Launch in default external Linux app via xdg-open",
                command: "xdg-open",
            },
            PreviewAction {
                label: "Edit in TermExplorer (F2)",
                description: "Open in built-in code editor",
                command: "edit",
            },
            PreviewAction {
                label: "Copy Absolute Path",
                description: "Copy full path to system clipboard",
                command: "copy_path",
            },
        ];

        if format_info.category == FileCategory::Archive {
            actions.push(PreviewAction {
                label: "Extract Archive Here",
                description: "Safely extract archive entries to a directory",
                command: "extract",
            });
        }

        if format_info.category == FileCategory::Audio
            || format_info.category == FileCategory::Video
        {
            actions.push(PreviewAction {
                label: "Play with mpv",
                description: "Stream playback via external mpv player",
                command: "play_mpv",
            });
        }

        Some(FullPreviewData {
            path: path.to_path_buf(),
            active_tab: tab,
            format_info,
            metadata,
            source,
            markdown,
            image,
            video,
            audio,
            pdf,
            office,
            data,
            archive,
            hex,
            database,
            actions,
        })
    }
}
