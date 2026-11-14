//! PDF preview handler extracting metadata, page count, text preview,
//! page navigation, and optional first-page thumbnail rendering.

use crate::preview::capabilities::render_halfblocks_from_image;
use crate::preview::registry::CapabilityRegistry;
use ratatui::style::Color;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct PdfPreview {
    pub page_count: Option<usize>,
    pub current_page: usize,
    pub title: Option<String>,
    pub author: Option<String>,
    pub pdf_version: String,
    pub extracted_text: Vec<String>,
    pub pdftotext_available: bool,
    pub pdftoppm_available: bool,
    pub page_thumbnail: Option<Vec<Vec<(Color, Color, char)>>>,
}

impl PdfPreview {
    pub fn load(path: &Path, max_lines: usize) -> Self {
        Self::load_page(path, 1, max_lines)
    }

    pub fn load_page(path: &Path, page: usize, max_lines: usize) -> Self {
        let pdftotext_available = CapabilityRegistry::is_tool_available("pdftotext");
        let pdftoppm_available = CapabilityRegistry::is_tool_available("pdftoppm");

        let mut page_count = None;
        let mut title = None;
        let mut author = None;
        let mut pdf_version = "PDF-1.4".to_string();
        let mut extracted_text = Vec::new();

        // 1. Pure-Rust scanning for PDF header and catalog
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            for (idx, line_res) in reader.lines().take(500).enumerate() {
                if let Ok(line) = line_res {
                    if idx == 0 && line.starts_with("%PDF-") {
                        pdf_version = line.trim().to_string();
                    }
                    if line.contains("/Count ") {
                        if let Some(count_part) = line.split("/Count ").nth(1) {
                            if let Some(num_str) = count_part.split_whitespace().next() {
                                if let Ok(parsed) = num_str
                                    .trim_end_matches(|c: char| !c.is_ascii_digit())
                                    .parse()
                                {
                                    page_count = Some(page_count.unwrap_or(1).max(parsed));
                                }
                            }
                        }
                    }
                    if line.contains("/Title (") && title.is_none() {
                        if let Some(t) = line
                            .split("/Title (")
                            .nth(1)
                            .and_then(|s| s.split(')').next())
                        {
                            title = Some(t.to_string());
                        }
                    }
                    if line.contains("/Author (") && author.is_none() {
                        if let Some(a) = line
                            .split("/Author (")
                            .nth(1)
                            .and_then(|s| s.split(')').next())
                        {
                            author = Some(a.to_string());
                        }
                    }
                }
            }
        }

        // 2. Extract text for specified page if pdftotext is available
        if pdftotext_available {
            let p_str = page.to_string();
            if let Ok(out) = Command::new("pdftotext")
                .args(["-f", &p_str, "-l", &p_str, "-layout"])
                .arg(path)
                .arg("-")
                .output()
            {
                if out.status.success() {
                    let text = String::from_utf8_lossy(&out.stdout);
                    for line in text.lines().take(max_lines) {
                        extracted_text.push(line.to_string());
                    }
                }
            }
        }

        // 3. Optional first-page thumbnail via pdftoppm if available
        let mut page_thumbnail = None;
        if pdftoppm_available && page == 1 {
            if let Ok(out) = Command::new("pdftoppm")
                .args([
                    "-png",
                    "-f",
                    "1",
                    "-l",
                    "1",
                    "-scale-to-x",
                    "80",
                    "-scale-to-y",
                    "40",
                ])
                .arg(path)
                .output()
            {
                if out.status.success() && !out.stdout.is_empty() {
                    if let Ok(img) = image::load_from_memory(&out.stdout) {
                        page_thumbnail = Some(render_halfblocks_from_image(&img, 40, 20));
                    }
                }
            }
        }

        Self {
            page_count,
            current_page: page,
            title,
            author,
            pdf_version,
            extracted_text,
            pdftotext_available,
            pdftoppm_available,
            page_thumbnail,
        }
    }

    pub fn next_page(&mut self, path: &Path, max_lines: usize) {
        let max_p = self.page_count.unwrap_or(1);
        if self.current_page < max_p {
            *self = Self::load_page(path, self.current_page + 1, max_lines);
        }
    }

    pub fn prev_page(&mut self, path: &Path, max_lines: usize) {
        if self.current_page > 1 {
            *self = Self::load_page(path, self.current_page - 1, max_lines);
        }
    }
}
