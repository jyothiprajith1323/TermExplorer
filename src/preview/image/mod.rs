//! Image and SVG preview handler with in-process vector rasterization,
//! truecolor half-block rendering, zoom/pan controls, and terminal graphics detection.

use crate::preview::capabilities::{
    detect_graphics_protocol, render_halfblocks_from_image, TerminalGraphicsProtocol,
};
use ratatui::style::Color;
use std::fs::File;
use std::io::Read;
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
    pub has_alpha: bool,
    pub zoom_level: f32,
    pub pan_x: i32,
    pub pan_y: i32,
    pub fit_to_panel: bool,
    pub protocol: TerminalGraphicsProtocol,
}

impl ImagePreview {
    pub fn load(path: &Path, preview_width: u32, preview_height: u32) -> Self {
        Self::load_with_transform(path, preview_width, preview_height, 1.0, 0, 0, true)
    }

    pub fn load_with_transform(
        path: &Path,
        preview_width: u32,
        preview_height: u32,
        zoom: f32,
        pan_x: i32,
        pan_y: i32,
        fit_to_panel: bool,
    ) -> Self {
        let protocol = detect_graphics_protocol();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        // 1. In-process safe SVG rasterization via resvg
        if ext == "svg" {
            return Self::load_svg(
                path,
                preview_width,
                preview_height,
                zoom,
                pan_x,
                pan_y,
                fit_to_panel,
                protocol,
            );
        }

        // 2. Standard Bitmaps via image crate (PNG, JPEG, GIF, WebP, BMP, ICO, TIFF)
        let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        if file_size > 50 * 1024 * 1024 {
            // Guard against decompression bombs or extreme files > 50MB
            return Self {
                width: 0,
                height: 0,
                format_name: "Oversized Image (>50MB)".to_string(),
                color_type: "Unknown".to_string(),
                aspect_ratio: "Unknown".to_string(),
                halfblock_grid: Vec::new(),
                is_animated: false,
                frame_count: None,
                has_alpha: false,
                zoom_level: zoom,
                pan_x,
                pan_y,
                fit_to_panel,
                protocol,
            };
        }

        match image::open(path) {
            Ok(img) => {
                let w = img.width();
                let h = img.height();
                let aspect = if h > 0 {
                    format!("{:.2}:1", w as f32 / h as f32)
                } else {
                    "1:1".to_string()
                };

                let has_alpha = img.color().has_alpha();
                let color_type = format!("{:?}", img.color());
                let is_gif = ext == "gif";

                let target_w = if fit_to_panel {
                    ((preview_width as f32 * zoom).round() as u32).max(1)
                } else {
                    ((w as f32 * zoom).round() as u32).max(1)
                };

                let target_h = if fit_to_panel {
                    ((preview_height as f32 * zoom).round() as u32).max(1)
                } else {
                    ((h as f32 * zoom).round() as u32).max(1)
                };

                let halfblock_grid = render_halfblocks_from_image(&img, target_w, target_h);

                let format_name = image::ImageReader::open(path)
                    .ok()
                    .and_then(|r| r.format())
                    .map(|f| format!("{:?}", f))
                    .unwrap_or_else(|| ext.to_uppercase());

                Self {
                    width: w,
                    height: h,
                    format_name,
                    color_type,
                    aspect_ratio: aspect,
                    halfblock_grid,
                    is_animated: is_gif,
                    frame_count: if is_gif { Some(1) } else { None },
                    has_alpha,
                    zoom_level: zoom,
                    pan_x,
                    pan_y,
                    fit_to_panel,
                    protocol,
                }
            }
            Err(e) => Self {
                width: 0,
                height: 0,
                format_name: format!("Image decode error: {e}"),
                color_type: "Unknown".to_string(),
                aspect_ratio: "Unknown".to_string(),
                halfblock_grid: Vec::new(),
                is_animated: false,
                frame_count: None,
                has_alpha: false,
                zoom_level: zoom,
                pan_x,
                pan_y,
                fit_to_panel,
                protocol,
            },
        }
    }

    /// Safe in-process SVG rasterization using resvg / usvg.
    #[allow(clippy::too_many_arguments)]
    fn load_svg(
        path: &Path,
        preview_width: u32,
        preview_height: u32,
        zoom: f32,
        pan_x: i32,
        pan_y: i32,
        fit_to_panel: bool,
        protocol: TerminalGraphicsProtocol,
    ) -> Self {
        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                return Self::error_svg(
                    format!("Could not open SVG: {e}"),
                    zoom,
                    pan_x,
                    pan_y,
                    fit_to_panel,
                    protocol,
                );
            }
        };

        let mut svg_bytes = Vec::new();
        if file.read_to_end(&mut svg_bytes).is_err() || svg_bytes.len() > 10 * 1024 * 1024 {
            return Self::error_svg(
                "Oversized SVG (>10MB)".to_string(),
                zoom,
                pan_x,
                pan_y,
                fit_to_panel,
                protocol,
            );
        }

        // usvg parse options: no external scripts or external file loading
        let mut opt = usvg::Options::default();
        opt.fontdb_mut().load_system_fonts();

        match usvg::Tree::from_data(&svg_bytes, &opt) {
            Ok(tree) => {
                let size = tree.size();
                let orig_w = size.width().round() as u32;
                let orig_h = size.height().round() as u32;

                let aspect = if orig_h > 0 {
                    format!("{:.2}:1", orig_w as f32 / orig_h as f32)
                } else {
                    "1:1".to_string()
                };

                // Compute render size based on viewport preview dimensions
                let target_w = if fit_to_panel {
                    ((preview_width as f32 * zoom).round() as u32).max(1)
                } else {
                    ((orig_w as f32 * zoom).round() as u32).max(1)
                };

                let target_h = if fit_to_panel {
                    ((preview_height as f32 * zoom).round() as u32).max(1)
                } else {
                    ((orig_h as f32 * zoom).round() as u32).max(1)
                };

                // Double vertical pixels for halfblocks
                let pixel_w = target_w;
                let pixel_h = target_h * 2;

                if let Some(mut pixmap) = tiny_skia::Pixmap::new(pixel_w, pixel_h) {
                    let sx = pixel_w as f32 / size.width();
                    let sy = pixel_h as f32 / size.height();
                    let transform = tiny_skia::Transform::from_scale(sx, sy);

                    resvg::render(&tree, transform, &mut pixmap.as_mut());

                    if let Ok(png_data) = pixmap.encode_png() {
                        if let Ok(img) = image::load_from_memory(&png_data) {
                            let halfblock_grid =
                                render_halfblocks_from_image(&img, target_w, target_h);

                            return Self {
                                width: orig_w,
                                height: orig_h,
                                format_name: "Scalable Vector Graphics (SVG)".to_string(),
                                color_type: "Vector / RGBA".to_string(),
                                aspect_ratio: aspect,
                                halfblock_grid,
                                is_animated: false,
                                frame_count: None,
                                has_alpha: true,
                                zoom_level: zoom,
                                pan_x,
                                pan_y,
                                fit_to_panel,
                                protocol,
                            };
                        }
                    }
                }

                Self {
                    width: orig_w,
                    height: orig_h,
                    format_name: "Scalable Vector Graphics (SVG)".to_string(),
                    color_type: "Vector".to_string(),
                    aspect_ratio: aspect,
                    halfblock_grid: Vec::new(),
                    is_animated: false,
                    frame_count: None,
                    has_alpha: true,
                    zoom_level: zoom,
                    pan_x,
                    pan_y,
                    fit_to_panel,
                    protocol,
                }
            }
            Err(e) => Self::error_svg(
                format!("SVG parse error: {e}"),
                zoom,
                pan_x,
                pan_y,
                fit_to_panel,
                protocol,
            ),
        }
    }

    fn error_svg(
        err: String,
        zoom: f32,
        pan_x: i32,
        pan_y: i32,
        fit_to_panel: bool,
        protocol: TerminalGraphicsProtocol,
    ) -> Self {
        Self {
            width: 0,
            height: 0,
            format_name: err,
            color_type: "Vector (Invalid)".to_string(),
            aspect_ratio: "Unknown".to_string(),
            halfblock_grid: Vec::new(),
            is_animated: false,
            frame_count: None,
            has_alpha: false,
            zoom_level: zoom,
            pan_x,
            pan_y,
            fit_to_panel,
            protocol,
        }
    }

    pub fn zoom_in(&mut self) {
        self.zoom_level = (self.zoom_level + 0.25).min(4.0);
    }

    pub fn zoom_out(&mut self) {
        self.zoom_level = (self.zoom_level - 0.25).max(0.25);
    }

    pub fn reset_zoom(&mut self) {
        self.zoom_level = 1.0;
        self.pan_x = 0;
        self.pan_y = 0;
    }

    pub fn pan(&mut self, dx: i32, dy: i32) {
        self.pan_x += dx;
        self.pan_y += dy;
    }

    pub fn toggle_fit(&mut self) {
        self.fit_to_panel = !self.fit_to_panel;
    }
}
