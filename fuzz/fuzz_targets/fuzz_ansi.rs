#![no_main]

use libfuzzer_sys::fuzz_target;
use termexplorer::preview::sanitizer::{sanitize_preview_line, sanitize_preview_text};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = sanitize_preview_text(s);
        let _ = sanitize_preview_line(s);
    }
});
