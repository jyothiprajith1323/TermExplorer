//! Audio preview handler extracting duration, codec, tags, embedded artwork,
//! and playback controls using pure-Rust Lofty and optional player integration.

use crate::preview::capabilities::render_halfblocks_from_image;
use crate::preview::registry::CapabilityRegistry;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::Accessor;
use ratatui::style::Color;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct AudioPreview {
    pub duration_seconds: Option<f64>,
    pub codec: Option<String>,
    pub sample_rate_hz: Option<u32>,
    pub channels: Option<u32>,
    pub bitrate_kbps: Option<u64>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub file_size_bytes: u64,
    pub cover_art: Option<Vec<Vec<(Color, Color, char)>>>,
    pub player_available: Option<&'static str>,
    pub is_playing: bool,
    pub volume_percent: u32,
}

impl AudioPreview {
    pub fn load(path: &Path) -> Self {
        let player_available = if CapabilityRegistry::is_tool_available("mpv") {
            Some("mpv")
        } else if CapabilityRegistry::is_tool_available("ffplay") {
            Some("ffplay")
        } else if CapabilityRegistry::is_tool_available("aplay") {
            Some("aplay")
        } else {
            None
        };

        let file_size_bytes = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

        let mut duration_seconds = None;
        let mut codec = None;
        let mut sample_rate_hz = None;
        let mut channels = None;
        let mut bitrate_kbps = None;
        let mut title = None;
        let mut artist = None;
        let mut album = None;
        let mut cover_art = None;

        // 1. Pure-Rust metadata parsing via Lofty
        if let Ok(tagged_file) = Probe::open(path).and_then(|p| p.read()) {
            let properties = tagged_file.properties();
            duration_seconds = Some(properties.duration().as_secs_f64());
            sample_rate_hz = properties.sample_rate();
            channels = properties.channels().map(|c| c as u32);
            bitrate_kbps = properties.audio_bitrate().map(|b| b as u64);
            codec = Some(format!("{:?}", tagged_file.file_type()));

            if let Some(tag) = tagged_file
                .primary_tag()
                .or_else(|| tagged_file.first_tag())
            {
                title = tag.title().as_deref().map(str::to_string);
                artist = tag.artist().as_deref().map(str::to_string);
                album = tag.album().as_deref().map(str::to_string);

                // 2. Extract embedded album artwork if available
                if let Some(picture) = tag.pictures().first() {
                    let pic_data = picture.data();
                    if let Ok(img) = image::load_from_memory(pic_data) {
                        cover_art = Some(render_halfblocks_from_image(&img, 30, 15));
                    }
                }
            }
        }

        Self {
            duration_seconds,
            codec,
            sample_rate_hz,
            channels,
            bitrate_kbps,
            title,
            artist,
            album,
            file_size_bytes,
            cover_art,
            player_available,
            is_playing: false,
            volume_percent: 80,
        }
    }
}
