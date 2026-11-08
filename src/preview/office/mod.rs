//! Office document preview handler (DOCX, XLSX, PPTX, ODT, ODS).

use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct OfficePreview {
    pub document_type: &'static str,
    pub title: Option<String>,
    pub sheet_or_slide_count: Option<usize>,
    pub extracted_paragraphs: Vec<String>,
    pub table_rows: Vec<Vec<String>>,
}

impl OfficePreview {
    pub fn load(path: &Path, max_lines: usize) -> Self {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        match ext.as_str() {
            "docx" => Self::load_docx(path, max_lines),
            "xlsx" => Self::load_xlsx(path, max_lines),
            "pptx" => Self::load_pptx(path, max_lines),
            "odt" | "ods" | "odp" => Self::load_odf(path, max_lines, &ext),
            _ => Self {
                document_type: "Legacy Office Document",
                title: None,
                sheet_or_slide_count: None,
                extracted_paragraphs: vec![
                    "[Use external viewer to view legacy binary office document]".to_string(),
                ],
                table_rows: Vec::new(),
            },
        }
    }

    fn load_docx(path: &Path, max_lines: usize) -> Self {
        let Ok(file) = File::open(path) else {
            return Self::error("Could not open DOCX archive");
        };

        let Ok(mut archive) = zip::ZipArchive::new(file) else {
            return Self::error("Invalid DOCX format");
        };

        let mut extracted_paragraphs = Vec::new();

        if let Ok(mut doc_xml) = archive.by_name("word/document.xml") {
            let mut content = String::new();
            if doc_xml.read_to_string(&mut content).is_ok() {
                extracted_paragraphs = extract_xml_tag_text(&content, "w:p", "w:t", max_lines);
            }
        }

        Self {
            document_type: "Microsoft Word (DOCX)",
            title: None,
            sheet_or_slide_count: None,
            extracted_paragraphs,
            table_rows: Vec::new(),
        }
    }

    fn load_xlsx(path: &Path, max_lines: usize) -> Self {
        let Ok(file) = File::open(path) else {
            return Self::error("Could not open XLSX archive");
        };

        let Ok(mut archive) = zip::ZipArchive::new(file) else {
            return Self::error("Invalid XLSX format");
        };

        // 1. Shared strings table
        let mut shared_strings = Vec::new();
        if let Ok(mut sst_file) = archive.by_name("xl/sharedStrings.xml") {
            let mut sst_content = String::new();
            if sst_file.read_to_string(&mut sst_content).is_ok() {
                for part in sst_content.split("<t>") {
                    if let Some(txt) = part.split("</t>").next() {
                        if !txt.is_empty() && !txt.starts_with("<?xml") {
                            shared_strings.push(txt.to_string());
                        }
                    }
                }
            }
        }

        // 2. Parse sheet1.xml
        let mut table_rows = Vec::new();
        if let Ok(mut sheet_file) = archive.by_name("xl/worksheets/sheet1.xml") {
            let mut sheet_content = String::new();
            if sheet_file.read_to_string(&mut sheet_content).is_ok() {
                for row_part in sheet_content.split("<row").skip(1).take(max_lines) {
                    let mut row = Vec::new();
                    for c_part in row_part.split("<c").skip(1).take(8) {
                        let is_string = c_part.contains("t=\"s\"");
                        if let Some(v_part) = c_part.split("<v>").nth(1) {
                            if let Some(val) = v_part.split("</v>").next() {
                                if is_string {
                                    if let Ok(idx) = val.parse::<usize>() {
                                        if let Some(s) = shared_strings.get(idx) {
                                            row.push(s.clone());
                                            continue;
                                        }
                                    }
                                }
                                row.push(val.to_string());
                            }
                        }
                    }
                    if !row.is_empty() {
                        table_rows.push(row);
                    }
                }
            }
        }

        Self {
            document_type: "Microsoft Excel (XLSX)",
            title: None,
            sheet_or_slide_count: Some(1),
            extracted_paragraphs: Vec::new(),
            table_rows,
        }
    }

    fn load_pptx(path: &Path, _max_lines: usize) -> Self {
        let Ok(file) = File::open(path) else {
            return Self::error("Could not open PPTX archive");
        };

        let Ok(mut archive) = zip::ZipArchive::new(file) else {
            return Self::error("Invalid PPTX format");
        };

        let mut slide_count = 0;
        for i in 0..archive.len() {
            if let Ok(entry) = archive.by_index(i) {
                if let Ok(name) = entry.name() {
                    if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                        slide_count += 1;
                    }
                }
            }
        }

        let mut extracted_paragraphs = Vec::new();
        if let Ok(mut slide1) = archive.by_name("ppt/slides/slide1.xml") {
            let mut content = String::new();
            if slide1.read_to_string(&mut content).is_ok() {
                for part in content.split("<a:t>").skip(1) {
                    if let Some(txt) = part.split("</a:t>").next() {
                        if !txt.is_empty() {
                            extracted_paragraphs.push(txt.to_string());
                        }
                    }
                }
            }
        }

        Self {
            document_type: "Microsoft PowerPoint (PPTX)",
            title: None,
            sheet_or_slide_count: Some(slide_count),
            extracted_paragraphs,
            table_rows: Vec::new(),
        }
    }

    fn load_odf(path: &Path, max_lines: usize, ext: &str) -> Self {
        let Ok(file) = File::open(path) else {
            return Self::error("Could not open OpenDocument archive");
        };

        let Ok(mut archive) = zip::ZipArchive::new(file) else {
            return Self::error("Invalid OpenDocument format");
        };

        let mut extracted_paragraphs = Vec::new();
        if let Ok(mut content_file) = archive.by_name("content.xml") {
            let mut content = String::new();
            if content_file.read_to_string(&mut content).is_ok() {
                for p in content.split("<text:p").skip(1).take(max_lines) {
                    if let Some(txt_with_tags) = p.split("</text:p>").next() {
                        // Strip internal XML tags
                        let mut clean = String::new();
                        let mut in_tag = false;
                        for ch in txt_with_tags.chars() {
                            if ch == '<' {
                                in_tag = true;
                            } else if ch == '>' {
                                in_tag = false;
                            } else if !in_tag {
                                clean.push(ch);
                            }
                        }
                        let trimmed = clean.trim();
                        if !trimmed.is_empty() {
                            extracted_paragraphs.push(trimmed.to_string());
                        }
                    }
                }
            }
        }

        let doc_type = match ext {
            "odt" => "OpenDocument Text (ODT)",
            "ods" => "OpenDocument Spreadsheet (ODS)",
            "odp" => "OpenDocument Presentation (ODP)",
            _ => "OpenDocument File",
        };

        Self {
            document_type: doc_type,
            title: None,
            sheet_or_slide_count: None,
            extracted_paragraphs,
            table_rows: Vec::new(),
        }
    }

    fn error(msg: &'static str) -> Self {
        Self {
            document_type: "Office Document",
            title: None,
            sheet_or_slide_count: None,
            extracted_paragraphs: vec![format!("[Error: {}]", msg)],
            table_rows: Vec::new(),
        }
    }
}

fn extract_xml_tag_text(xml: &str, row_tag: &str, text_tag: &str, max_items: usize) -> Vec<String> {
    let mut results = Vec::new();
    let row_open = format!("<{}", row_tag);
    let row_close = format!("</{}>", row_tag);
    let text_open = format!("<{}", text_tag);
    let text_close = format!("</{}>", text_tag);

    for row_part in xml.split(&row_open).skip(1).take(max_items) {
        if let Some(row_content) = row_part.split(&row_close).next() {
            let mut row_text = String::new();
            for t_part in row_content.split(&text_open).skip(1) {
                if let Some(content) = t_part.split('>').nth(1) {
                    if let Some(txt) = content.split(&text_close).next() {
                        row_text.push_str(txt);
                    }
                }
            }
            let trimmed = row_text.trim();
            if !trimmed.is_empty() {
                results.push(trimmed.to_string());
            }
        }
    }
    results
}
