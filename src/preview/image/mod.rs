//! Image preview handler with dimension metadata and half-block truecolor rendering.

use crate::preview::rendering::GraphicsCapability;
use ratatui::style::Color;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ImagePreview {
    pub width: u32,
    pub height: u32,
    pub format_name: String,
    pub color_type: String,
    pub aspect_ratio: String,
    pub halfblock_grid: Vec<Vec<(Color, Color, char)>>,
    pub is_animated: bool,
    pub frame_count: Option<usize>,
}

impl ImagePreview {
    pub fn load(path: &Path, preview_width: u32, preview_height: u32) -> Self {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        // Special handling for SVG
        if ext == "svg" {
            let svg_str = std::fs::read_to_string(path).unwrap_or_default();
            let (w, h) = parse_svg_dimensions(&svg_str);
            return Self {
                width: w,
                height: h,
                format_name: "Scalable Vector Graphics (SVG)".to_string(),
                color_type: "Vector / XML".to_string(),
                aspect_ratio: if h > 0 {
                    format!("{:.2}:1", w as f32 / h as f32)
                } else {
                    "Unknown".to_string()
                },
                halfblock_grid: Vec::new(),
                is_animated: false,
                frame_count: None,
            };
        }

        // Use image crate to decode image
        match image::image_dimensions(path) {
            Ok((w, h)) => {
                let aspect = if h > 0 {
                    format!("{:.2}:1", w as f32 / h as f32)
                } else {
                    "1:1".to_string()
                };

                // Attempt to decode thumbnail (guard against huge files >20MB to prevent UI freeze)
                let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
                let halfblock_grid = if file_size <= 20 * 1024 * 1024 {
                    match image::open(path) {
                        Ok(img) => {
                            GraphicsCapability::render_halfblocks(&img, preview_width, preview_height)
                        }
                        Err(_) => Vec::new(),
                    }
                } else {
                    Vec::new()
                };

                let format_name = image::ImageReader::open(path)
                    .ok()
                    .and_then(|r| r.format())
                    .map(|f| format!("{:?}", f))
                    .unwrap_or_else(|| ext.to_uppercase());

                let is_gif = ext == "gif";

                Self {
                    width: w,
                    height: h,
                    format_name,
                    color_type: "RGB / RGBA".to_string(),
                    aspect_ratio: aspect,
                    halfblock_grid,
                    is_animated: is_gif,
                    frame_count: if is_gif { Some(1) } else { None },
                }
            }
            Err(e) => Self {
                width: 0,
                height: 0,
                format_name: format!("Image error: {}", e),
                color_type: "Unknown".to_string(),
                aspect_ratio: "Unknown".to_string(),
                halfblock_grid: Vec::new(),
                is_animated: false,
                frame_count: None,
            },
        }
    }
}

fn parse_svg_dimensions(svg: &str) -> (u32, u32) {
    let mut w = 0u32;
    let mut h = 0u32;
    for part in svg.split_whitespace().take(50) {
        if part.starts_with("width=\"") {
            if let Some(val) = part
                .strip_prefix("width=\"")
                .and_then(|s| s.split('"').next())
            {
                w = val
                    .trim_end_matches(|c: char| !c.is_ascii_digit())
                    .parse()
                    .unwrap_or(0);
            }
        } else if part.starts_with("height=\"") {
            if let Some(val) = part
                .strip_prefix("height=\"")
                .and_then(|s| s.split('"').next())
            {
                h = val
                    .trim_end_matches(|c: char| !c.is_ascii_digit())
                    .parse()
                    .unwrap_or(0);
            }
        }
    }
    (w, h)
}
