//! Virtualized binary and hex viewer with arbitrary offset seeking,
//! executable format header detection, and byte search.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct HexRow {
    pub offset: usize,
    pub hex_left: String,
    pub hex_right: String,
    pub ascii: String,
}

#[derive(Debug, Clone)]
pub struct HexPreview {
    pub base_offset: usize,
    pub rows: Vec<HexRow>,
    pub file_size: u64,
    pub detected_format: &'static str,
    pub extracted_strings: Vec<String>,
}

impl HexPreview {
    pub fn load(path: &Path, max_rows: usize) -> Self {
        Self::load_at_offset(path, 0, max_rows)
    }

    /// Read a bounded window of bytes starting at an arbitrary offset using file seeking.
    /// Never loads entire large files into memory ($O(1)$ memory consumption).
    pub fn load_at_offset(path: &Path, offset: usize, max_rows: usize) -> Self {
        let Ok(mut file) = File::open(path) else {
            return Self {
                base_offset: 0,
                rows: Vec::new(),
                file_size: 0,
                detected_format: "Unknown / Inaccessible",
                extracted_strings: Vec::new(),
            };
        };

        let file_size = file.metadata().map(|m| m.len()).unwrap_or(0);
        let safe_offset = (offset as u64).min(file_size.saturating_sub(1));

        let _ = file.seek(SeekFrom::Start(safe_offset));

        let max_bytes = max_rows * 16;
        let mut buffer = vec![0u8; max_bytes];
        let bytes_read = file.read(&mut buffer).unwrap_or(0);
        let data = &buffer[..bytes_read];

        let mut rows = Vec::new();
        for chunk_idx in (0..data.len()).step_by(16) {
            let chunk = &data[chunk_idx..(chunk_idx + 16).min(data.len())];
            let row_offset = safe_offset as usize + chunk_idx;

            let mut hex_left = String::new();
            let mut hex_right = String::new();
            let mut ascii = String::new();

            for (i, &b) in chunk.iter().enumerate() {
                let s = format!("{:02x} ", b);
                if i < 8 {
                    hex_left.push_str(&s);
                } else {
                    hex_right.push_str(&s);
                }

                if (32..=126).contains(&b) {
                    ascii.push(b as char);
                } else {
                    ascii.push('.');
                }
            }

            if chunk.len() < 8 {
                let missing = 8 - chunk.len();
                hex_left.push_str(&"   ".repeat(missing));
                hex_right.push_str(&"   ".repeat(8));
            } else if chunk.len() < 16 {
                let missing = 16 - chunk.len();
                hex_right.push_str(&"   ".repeat(missing));
            }

            rows.push(HexRow {
                offset: row_offset,
                hex_left,
                hex_right,
                ascii,
            });
        }

        // Detect format from initial file header
        let detected_format = detect_executable_format(path);

        // Extract printable strings from the current read window
        let mut extracted_strings = Vec::new();
        let mut current_string = String::new();
        for &b in data {
            if (32..=126).contains(&b) {
                current_string.push(b as char);
            } else {
                if current_string.len() >= 4 {
                    extracted_strings.push(current_string.clone());
                    if extracted_strings.len() >= 20 {
                        break;
                    }
                }
                current_string.clear();
            }
        }

        Self {
            base_offset: safe_offset as usize,
            rows,
            file_size,
            detected_format,
            extracted_strings,
        }
    }

    /// Search for an ASCII or hex byte pattern in the file and return matching offsets.
    pub fn search_pattern(path: &Path, pattern: &[u8], max_matches: usize) -> Vec<usize> {
        let mut matches = Vec::new();
        if pattern.is_empty() {
            return matches;
        }

        if let Ok(mut file) = File::open(path) {
            let mut buffer = vec![0u8; 65536]; // 64 KiB buffer
            let mut global_offset = 0usize;

            while let Ok(bytes_read) = file.read(&mut buffer) {
                if bytes_read == 0 {
                    break;
                }
                let slice = &buffer[..bytes_read];

                let mut pos = 0;
                while let Some(idx) = slice[pos..]
                    .windows(pattern.len())
                    .position(|w| w == pattern)
                {
                    let match_offset = global_offset + pos + idx;
                    matches.push(match_offset);
                    if matches.len() >= max_matches {
                        return matches;
                    }
                    pos += idx + 1;
                    if pos >= slice.len() {
                        break;
                    }
                }

                global_offset += bytes_read;
            }
        }

        matches
    }
}

fn detect_executable_format(path: &Path) -> &'static str {
    let mut header = [0u8; 64];
    if let Ok(mut f) = File::open(path) {
        if let Ok(n) = f.read(&mut header) {
            let buf = &header[..n];
            if buf.starts_with(b"\x7fELF") {
                if buf.len() > 4 && buf[4] == 2 {
                    return "ELF 64-bit Executable / Shared Object";
                } else {
                    return "ELF 32-bit Executable";
                }
            }
            if buf.starts_with(b"MZ") {
                return "Windows PE Executable";
            }
            if buf.starts_with(&[0xfe, 0xed, 0xfa, 0xce])
                || buf.starts_with(&[0xfe, 0xed, 0xfa, 0xcf])
            {
                return "Mach-O Binary";
            }
            if buf.starts_with(b"\0asm") {
                return "WebAssembly Binary (WASM)";
            }
            if buf.starts_with(b"SQLite format 3\0") {
                return "SQLite 3 Database File";
            }
            if buf.starts_with(b"%PDF-") {
                return "PDF Document";
            }
            if buf.starts_with(b"PK\x03\x04") {
                return "ZIP Compressed File";
            }
        }
    }
    "Binary Data"
}
