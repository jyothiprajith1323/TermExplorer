//! PDF preview handler extracting metadata, page count, and text preview.

use crate::preview::registry::CapabilityRegistry;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct PdfPreview {
    pub page_count: Option<usize>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub pdf_version: String,
    pub extracted_text: Vec<String>,
    pub pdftotext_available: bool,
    pub pdftoppm_available: bool,
}

impl PdfPreview {
    pub fn load(path: &Path, max_lines: usize) -> Self {
        let pdftotext_available = CapabilityRegistry::is_tool_available("pdftotext");
        let pdftoppm_available = CapabilityRegistry::is_tool_available("pdftoppm");

        let mut page_count = None;
        let mut title = None;
        let mut author = None;
        let mut pdf_version = "Unknown".to_string();
        let mut extracted_text = Vec::new();

        // 1. Try pure Rust scanning for PDF header and object catalog
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            for (idx, line_res) in reader.lines().take(200).enumerate() {
                if let Ok(line) = line_res {
                    if idx == 0 && line.starts_with("%PDF-") {
                        pdf_version = line.trim().to_string();
                    }
                    if line.contains("/Count ") && page_count.is_none() {
                        if let Some(count_part) = line.split("/Count ").nth(1) {
                            if let Some(num_str) = count_part.split_whitespace().next() {
                                page_count = num_str
                                    .trim_end_matches(|c: char| !c.is_ascii_digit())
                                    .parse()
                                    .ok();
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

        // 2. Extract text stream if pdftotext is available
        if pdftotext_available {
            if let Ok(out) = Command::new("pdftotext")
                .args(["-f", "1", "-l", "3", "-layout"])
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

        Self {
            page_count,
            title,
            author,
            pdf_version,
            extracted_text,
            pdftotext_available,
            pdftoppm_available,
        }
    }
}
