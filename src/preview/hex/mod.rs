//! Binary and hex viewer with offset, hex bytes, and ASCII representation.

use std::fs::File;
use std::io::Read;
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
    pub rows: Vec<HexRow>,
    pub file_size: u64,
    pub extracted_strings: Vec<String>,
}

impl HexPreview {
    pub fn load(path: &Path, max_rows: usize) -> Self {
        let Ok(mut file) = File::open(path) else {
            return Self {
                rows: Vec::new(),
                file_size: 0,
                extracted_strings: Vec::new(),
            };
        };

        let file_size = file.metadata().map(|m| m.len()).unwrap_or(0);
        let max_bytes = max_rows * 16;
        let mut buffer = vec![0u8; max_bytes];
        let bytes_read = file.read(&mut buffer).unwrap_or(0);
        let data = &buffer[..bytes_read];

        let mut rows = Vec::new();
        for chunk_idx in (0..data.len()).step_by(16) {
            let chunk = &data[chunk_idx..(chunk_idx + 16).min(data.len())];
            let offset = chunk_idx;

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

            // Pad hex strings if incomplete line
            if chunk.len() < 8 {
                let missing = 8 - chunk.len();
                hex_left.push_str(&"   ".repeat(missing));
                hex_right.push_str(&"   ".repeat(8));
            } else if chunk.len() < 16 {
                let missing = 16 - chunk.len();
                hex_right.push_str(&"   ".repeat(missing));
            }

            rows.push(HexRow {
                offset,
                hex_left,
                hex_right,
                ascii,
            });
        }

        // Extract printable strings (min length 4)
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
            rows,
            file_size,
            extracted_strings,
        }
    }
}
