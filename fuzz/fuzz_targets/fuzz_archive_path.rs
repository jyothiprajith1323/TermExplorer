#![no_main]

use libfuzzer_sys::fuzz_target;
use termexplorer::preview::archive::is_safe_archive_path;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = is_safe_archive_path(s);
    }
});
