//! Video preview handler extracting container metadata, multi-frame thumbnails,
//! and process-isolated playback integration.

use crate::preview::capabilities::render_halfblocks_from_image;
use crate::preview::registry::CapabilityRegistry;
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
    pub file_size_bytes: u64,
    pub ffprobe_available: bool,
    pub ffmpeg_available: bool,
    pub mpv_available: bool,
    pub selected_frame_idx: usize,
    pub frame_timestamps: Vec<f64>,
    pub frames: Vec<Vec<Vec<(Color, Color, char)>>>,
    pub thumbnail_grid: Vec<Vec<(Color, Color, char)>>,
}

pub type HalfblockGrid = Vec<Vec<(Color, Color, char)>>;

impl VideoPreview {
    pub fn load(path: &Path, preview_w: u32, preview_h: u32) -> Self {
        let ffprobe_available = CapabilityRegistry::is_tool_available("ffprobe");
        let ffmpeg_available = CapabilityRegistry::is_tool_available("ffmpeg");
        let mpv_available = CapabilityRegistry::is_tool_available("mpv");
        let file_size_bytes = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

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

        // Multi-frame timestamps: 0% (1s), 25%, 50%, 75%
        let dur = duration_seconds.unwrap_or(10.0);
        let frame_timestamps = vec![
            1.0f64.min(dur),
            (dur * 0.25).max(1.0),
            (dur * 0.50).max(1.0),
            (dur * 0.75).max(1.0),
        ];

        let mut frames = Vec::new();
        if ffmpeg_available {
            // Extract first frame
            if let Some(grid) = Self::extract_frame(path, frame_timestamps[0], preview_w, preview_h)
            {
                frames.push(grid);
            }
        }

        let thumbnail_grid = frames.first().cloned().unwrap_or_default();

        Self {
            duration_seconds,
            width,
            height,
            video_codec,
            audio_codec,
            frame_rate,
            bitrate_kbps,
            file_size_bytes,
            ffprobe_available,
            ffmpeg_available,
            mpv_available,
            selected_frame_idx: 0,
            frame_timestamps,
            frames,
            thumbnail_grid,
        }
    }

    /// Safely extract a frame at a specific timestamp using ffmpeg with piped stdout.
    pub fn extract_frame(
        path: &Path,
        timestamp_secs: f64,
        preview_w: u32,
        preview_h: u32,
    ) -> Option<Vec<Vec<(Color, Color, char)>>> {
        let ts_str = format!("{:.2}", timestamp_secs);
        let out = Command::new("ffmpeg")
            .args(["-y", "-ss", &ts_str, "-i"])
            .arg(path)
            .args(["-vframes", "1", "-f", "image2pipe", "-vcodec", "png", "-"])
            .output()
            .ok()?;

        if out.status.success() && !out.stdout.is_empty() {
            if let Ok(img) = image::load_from_memory(&out.stdout) {
                return Some(render_halfblocks_from_image(&img, preview_w, preview_h));
            }
        }
        None
    }

    pub fn next_frame(&mut self, path: &Path, preview_w: u32, preview_h: u32) {
        if self.selected_frame_idx + 1 < self.frame_timestamps.len() {
            self.selected_frame_idx += 1;
            self.ensure_frame_loaded(path, self.selected_frame_idx, preview_w, preview_h);
        }
    }

    pub fn prev_frame(&mut self, path: &Path, preview_w: u32, preview_h: u32) {
        if self.selected_frame_idx > 0 {
            self.selected_frame_idx -= 1;
            self.ensure_frame_loaded(path, self.selected_frame_idx, preview_w, preview_h);
        }
    }

    pub fn jump_start(&mut self, path: &Path, preview_w: u32, preview_h: u32) {
        self.selected_frame_idx = 0;
        self.ensure_frame_loaded(path, 0, preview_w, preview_h);
    }

    pub fn jump_midpoint(&mut self, path: &Path, preview_w: u32, preview_h: u32) {
        self.selected_frame_idx = 2; // 50%
        self.ensure_frame_loaded(path, 2, preview_w, preview_h);
    }

    fn ensure_frame_loaded(&mut self, path: &Path, idx: usize, preview_w: u32, preview_h: u32) {
        while self.frames.len() <= idx {
            if self.ffmpeg_available && idx < self.frame_timestamps.len() {
                if let Some(grid) =
                    Self::extract_frame(path, self.frame_timestamps[idx], preview_w, preview_h)
                {
                    self.frames.push(grid);
                } else {
                    self.frames.push(Vec::new());
                }
            } else {
                self.frames.push(Vec::new());
            }
        }
    }

    pub fn current_frame(&self) -> Option<&HalfblockGrid> {
        self.frames.get(self.selected_frame_idx)
    }
}
