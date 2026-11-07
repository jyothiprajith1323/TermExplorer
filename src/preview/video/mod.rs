//! Video preview handler extracting container metadata and thumbnail frames.

use crate::preview::registry::CapabilityRegistry;
use crate::preview::rendering::GraphicsCapability;
use ratatui::style::Color;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct VideoPreview {
    pub duration_seconds: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub frame_rate: Option<String>,
    pub bitrate_kbps: Option<u64>,
    pub ffprobe_available: bool,
    pub ffmpeg_available: bool,
    pub mpv_available: bool,
    pub thumbnail_grid: Vec<Vec<(Color, Color, char)>>,
}

impl VideoPreview {
    pub fn load(path: &Path, preview_w: u32, preview_h: u32) -> Self {
        let ffprobe_available = CapabilityRegistry::is_tool_available("ffprobe");
        let ffmpeg_available = CapabilityRegistry::is_tool_available("ffmpeg");
        let mpv_available = CapabilityRegistry::is_tool_available("mpv");

        let mut duration_seconds = None;
        let mut width = None;
        let mut height = None;
        let mut video_codec = None;
        let mut audio_codec = None;
        let mut frame_rate = None;
        let mut bitrate_kbps = None;

        if ffprobe_available {
            if let Ok(out) = Command::new("ffprobe")
                .args([
                    "-v",
                    "error",
                    "-select_streams",
                    "v:0",
                    "-show_entries",
                    "stream=width,height,codec_name,r_frame_rate,duration,bit_rate",
                    "-of",
                    "default=noprint_wrappers=1",
                ])
                .arg(path)
                .output()
            {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines() {
                    let mut parts = line.splitn(2, '=');
                    if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                        match k {
                            "width" => width = v.parse().ok(),
                            "height" => height = v.parse().ok(),
                            "codec_name" => video_codec = Some(v.to_string()),
                            "r_frame_rate" => frame_rate = Some(v.to_string()),
                            "duration" => duration_seconds = v.parse().ok(),
                            "bit_rate" => bitrate_kbps = v.parse::<u64>().ok().map(|b| b / 1000),
                            _ => {}
                        }
                    }
                }
            }

            // Probe audio stream
            if let Ok(out) = Command::new("ffprobe")
                .args([
                    "-v",
                    "error",
                    "-select_streams",
                    "a:0",
                    "-show_entries",
                    "stream=codec_name",
                    "-of",
                    "default=noprint_wrappers=1",
                ])
                .arg(path)
                .output()
            {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines() {
                    if let Some(v) = line.strip_prefix("codec_name=") {
                        audio_codec = Some(v.to_string());
                    }
                }
            }
        }

        // Thumbnail extraction via ffmpeg if available
        let mut thumbnail_grid = Vec::new();
        if ffmpeg_available {
            if let Ok(out) = Command::new("ffmpeg")
                .args(["-y", "-ss", "00:00:01", "-i"])
                .arg(path)
                .args(["-vframes", "1", "-f", "image2pipe", "-vcodec", "png", "-"])
                .output()
            {
                if out.status.success() && !out.stdout.is_empty() {
                    if let Ok(img) = image::load_from_memory(&out.stdout) {
                        thumbnail_grid =
                            GraphicsCapability::render_halfblocks(&img, preview_w, preview_h);
                    }
                }
            }
        }

        Self {
            duration_seconds,
            width,
            height,
            video_codec,
            audio_codec,
            frame_rate,
            bitrate_kbps,
            ffprobe_available,
            ffmpeg_available,
            mpv_available,
            thumbnail_grid,
        }
    }
}
