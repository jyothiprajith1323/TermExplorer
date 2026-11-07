//! Audio preview handler extracting duration, codec, tags, and playback controls.

use crate::preview::registry::CapabilityRegistry;
use std::path::Path;
use std::process::Command;

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
    pub ffprobe_available: bool,
    pub player_available: Option<&'static str>,
}

impl AudioPreview {
    pub fn load(path: &Path) -> Self {
        let ffprobe_available = CapabilityRegistry::is_tool_available("ffprobe");
        let player_available = if CapabilityRegistry::is_tool_available("mpv") {
            Some("mpv")
        } else if CapabilityRegistry::is_tool_available("ffplay") {
            Some("ffplay")
        } else if CapabilityRegistry::is_tool_available("aplay") {
            Some("aplay")
        } else {
            None
        };

        let mut duration_seconds = None;
        let mut codec = None;
        let mut sample_rate_hz = None;
        let mut channels = None;
        let mut bitrate_kbps = None;
        let mut title = None;
        let mut artist = None;
        let mut album = None;

        if ffprobe_available {
            if let Ok(out) = Command::new("ffprobe")
                .args([
                    "-v", "error",
                    "-select_streams", "a:0",
                    "-show_entries", "stream=codec_name,sample_rate,channels,bit_rate,duration:format_tags=title,artist,album",
                    "-of", "default=noprint_wrappers=1",
                ])
                .arg(path)
                .output()
            {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines() {
                    let mut parts = line.splitn(2, '=');
                    if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                        match k {
                            "codec_name" => codec = Some(v.to_string()),
                            "sample_rate" => sample_rate_hz = v.parse().ok(),
                            "channels" => channels = v.parse().ok(),
                            "duration" => duration_seconds = v.parse().ok(),
                            "bit_rate" => bitrate_kbps = v.parse::<u64>().ok().map(|b| b / 1000),
                            "TAG:title" | "title" => title = Some(v.to_string()),
                            "TAG:artist" | "artist" => artist = Some(v.to_string()),
                            "TAG:album" | "album" => album = Some(v.to_string()),
                            _ => {}
                        }
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
            ffprobe_available,
            player_available,
        }
    }
}
