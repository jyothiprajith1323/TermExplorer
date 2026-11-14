//! Content-based and extension-based file format detection.
//!
//! Implements a 5-layer detection strategy:
//! 1. Bounded magic-byte inspection (first 8 KiB).
//! 2. Content XML/SVG/ASCII structure inspection.
//! 3. File extension matching with Office / Archive disambiguation.
//! 4. MIME metadata inspection via `mime_guess`.
//! 5. Binary null-byte / non-printable heuristic fallback.

use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileCategory {
    SourceCode,
    Markdown,
    Image,
    Video,
    Audio,
    Pdf,
    Office,
    Data,
    Archive,
    Database,
    BinaryHex,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFormatInfo {
    pub category: FileCategory,
    pub mime_type: String,
    pub description: &'static str,
    pub is_binary: bool,
}

impl FileFormatInfo {
    pub fn new(
        category: FileCategory,
        mime_type: impl Into<String>,
        desc: &'static str,
        is_binary: bool,
    ) -> Self {
        Self {
            category,
            mime_type: mime_type.into(),
            description: desc,
            is_binary,
        }
    }
}

/// Sniff file category and MIME info from a bounded sample (up to 8 KiB) and extension.
pub fn detect_file_format(path: &Path) -> FileFormatInfo {
    // Read up to first 8192 bytes (8 KiB) for magic number and content analysis
    let mut header = [0u8; 8192];
    let bytes_read = match File::open(path) {
        Ok(mut f) => f.read(&mut header).unwrap_or(0),
        Err(_) => 0,
    };
    let buf = &header[..bytes_read];

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();

    // ── LAYER 1: STRICT MAGIC BYTES (Overrides any misleading extension) ─────────

    // 1.1 Executable Binaries (ELF, Mach-O, Windows PE, WebAssembly)
    if buf.starts_with(b"\x7fELF") {
        return FileFormatInfo::new(
            FileCategory::BinaryHex,
            "application/x-executable",
            "ELF Executable / Shared Object",
            true,
        );
    }
    if buf.starts_with(&[0xfe, 0xed, 0xfa, 0xce])
        || buf.starts_with(&[0xfe, 0xed, 0xfa, 0xcf])
        || buf.starts_with(&[0xce, 0xfa, 0xed, 0xfe])
        || buf.starts_with(&[0xcf, 0xfa, 0xed, 0xfe])
        || buf.starts_with(&[0xca, 0xfe, 0xba, 0xbe])
    {
        return FileFormatInfo::new(
            FileCategory::BinaryHex,
            "application/x-mach-binary",
            "Mach-O Binary / Universal Binary",
            true,
        );
    }
    if buf.starts_with(b"MZ") {
        return FileFormatInfo::new(
            FileCategory::BinaryHex,
            "application/vnd.microsoft.portable-executable",
            "Windows PE Executable / DLL",
            true,
        );
    }
    if buf.starts_with(b"\0asm") {
        return FileFormatInfo::new(
            FileCategory::BinaryHex,
            "application/wasm",
            "WebAssembly Binary (WASM)",
            true,
        );
    }

    // 1.2 SQLite 3 Database
    if buf.starts_with(b"SQLite format 3\0") {
        return FileFormatInfo::new(
            FileCategory::Database,
            "application/vnd.sqlite3",
            "SQLite Database",
            true,
        );
    }

    // 1.3 PDF Document
    if buf.starts_with(b"%PDF-") {
        return FileFormatInfo::new(FileCategory::Pdf, "application/pdf", "PDF Document", true);
    }

    // 1.4 Native Bitmaps
    if buf.starts_with(b"\x89PNG\r\n\x1a\n") {
        return FileFormatInfo::new(FileCategory::Image, "image/png", "PNG Image", true);
    }
    if buf.starts_with(&[0xff, 0xd8, 0xff]) {
        return FileFormatInfo::new(FileCategory::Image, "image/jpeg", "JPEG Image", true);
    }
    if buf.starts_with(b"GIF87a") || buf.starts_with(b"GIF89a") {
        return FileFormatInfo::new(FileCategory::Image, "image/gif", "GIF Image", true);
    }
    if buf.starts_with(b"RIFF") && buf.len() > 12 && &buf[8..12] == b"WEBP" {
        return FileFormatInfo::new(FileCategory::Image, "image/webp", "WebP Image", true);
    }
    if buf.starts_with(b"BM") {
        return FileFormatInfo::new(FileCategory::Image, "image/bmp", "BMP Image", true);
    }
    if buf.starts_with(b"II*\x00") || buf.starts_with(b"MM\x00*") {
        return FileFormatInfo::new(FileCategory::Image, "image/tiff", "TIFF Image", true);
    }

    // 1.5 Video / Audio Magic Bytes
    if buf.starts_with(&[0x1a, 0x45, 0xdf, 0xa3]) {
        return FileFormatInfo::new(
            FileCategory::Video,
            "video/webm",
            "Matroska / WebM Container",
            true,
        );
    }
    if buf.len() >= 8 && (&buf[4..8] == b"ftyp" || &buf[4..8] == b"moov") {
        return FileFormatInfo::new(
            FileCategory::Video,
            "video/mp4",
            "MP4 / QuickTime Video",
            true,
        );
    }
    if buf.starts_with(b"RIFF") && buf.len() > 12 && &buf[8..12] == b"AVI " {
        return FileFormatInfo::new(FileCategory::Video, "video/x-msvideo", "AVI Video", true);
    }
    if buf.starts_with(b"ID3") || (buf.len() >= 2 && buf[0] == 0xff && (buf[1] & 0xe0) == 0xe0) {
        return FileFormatInfo::new(FileCategory::Audio, "audio/mpeg", "MP3 Audio", true);
    }
    if buf.starts_with(b"RIFF") && buf.len() > 12 && &buf[8..12] == b"WAVE" {
        return FileFormatInfo::new(FileCategory::Audio, "audio/wav", "WAV Audio", true);
    }
    if buf.starts_with(b"fLaC") {
        return FileFormatInfo::new(FileCategory::Audio, "audio/flac", "FLAC Audio", true);
    }
    if buf.starts_with(b"OggS") {
        return FileFormatInfo::new(FileCategory::Audio, "audio/ogg", "Ogg Audio/Video", true);
    }

    // 1.6 Compressed Archives
    if buf.starts_with(&[0x1f, 0x8b]) {
        return FileFormatInfo::new(
            FileCategory::Archive,
            "application/gzip",
            "Gzip Compressed Archive",
            true,
        );
    }
    if buf.starts_with(b"BZh") {
        return FileFormatInfo::new(
            FileCategory::Archive,
            "application/x-bzip2",
            "Bzip2 Compressed Archive",
            true,
        );
    }
    if buf.starts_with(&[0xfd, b'7', b'z', b'X', b'Z', 0x00]) {
        return FileFormatInfo::new(
            FileCategory::Archive,
            "application/x-xz",
            "XZ Compressed Archive",
            true,
        );
    }
    if buf.starts_with(b"7z\xbc\xaf\x27\x1c") {
        return FileFormatInfo::new(
            FileCategory::Archive,
            "application/x-7z-compressed",
            "7-Zip Archive",
            true,
        );
    }
    if buf.len() >= 262 && &buf[257..262] == b"ustar" {
        return FileFormatInfo::new(
            FileCategory::Archive,
            "application/x-tar",
            "Tarball Archive",
            true,
        );
    }

    // 1.7 ZIP / Office Disambiguation (ZIP Magic PK\x03\x04 or PK\x05\x06)
    if buf.starts_with(b"PK\x03\x04") || buf.starts_with(b"PK\x05\x06") {
        // Disambiguate Microsoft Office OpenXML & OpenDocument from generic ZIP
        match ext.as_str() {
            "docx" => {
                return FileFormatInfo::new(
                    FileCategory::Office,
                    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                    "Microsoft Word Document (DOCX)",
                    true,
                );
            }
            "xlsx" => {
                return FileFormatInfo::new(
                    FileCategory::Office,
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                    "Microsoft Excel Spreadsheet (XLSX)",
                    true,
                );
            }
            "pptx" => {
                return FileFormatInfo::new(
                    FileCategory::Office,
                    "application/vnd.openxmlformats-officedocument.presentationml.presentation",
                    "Microsoft PowerPoint Presentation (PPTX)",
                    true,
                );
            }
            "odt" => {
                return FileFormatInfo::new(
                    FileCategory::Office,
                    "application/vnd.oasis.opendocument.text",
                    "OpenDocument Text (ODT)",
                    true,
                );
            }
            "ods" => {
                return FileFormatInfo::new(
                    FileCategory::Office,
                    "application/vnd.oasis.opendocument.spreadsheet",
                    "OpenDocument Spreadsheet (ODS)",
                    true,
                );
            }
            "odp" => {
                return FileFormatInfo::new(
                    FileCategory::Office,
                    "application/vnd.oasis.opendocument.presentation",
                    "OpenDocument Presentation (ODP)",
                    true,
                );
            }
            _ => {
                // If named .txt or .rs but starts with PK, it IS an archive!
                return FileFormatInfo::new(
                    FileCategory::Archive,
                    "application/zip",
                    "ZIP Archive",
                    true,
                );
            }
        }
    }

    // ── LAYER 2: TEXT CONTENT INSPECTION (SVG, XML, HTML, JSON) ───────────────────
    if let Ok(sample_str) = std::str::from_utf8(buf) {
        let trimmed = sample_str.trim_start();
        if trimmed.starts_with("<svg")
            || (trimmed.starts_with("<?xml") && sample_str.contains("<svg"))
        {
            return FileFormatInfo::new(
                FileCategory::Image,
                "image/svg+xml",
                "Scalable Vector Graphics (SVG)",
                false,
            );
        }
    }

    // ── LAYER 3: FILE EXTENSION & MIME CORRELATION ────────────────────────────────

    // 3.1 Images & Vectors by extension
    if ext == "svg" {
        return FileFormatInfo::new(
            FileCategory::Image,
            "image/svg+xml",
            "Scalable Vector Graphics (SVG)",
            false,
        );
    }
    if ext == "ico" {
        return FileFormatInfo::new(FileCategory::Image, "image/x-icon", "Icon (ICO)", true);
    }
    if ext == "avif" {
        return FileFormatInfo::new(FileCategory::Image, "image/avif", "AVIF Image", true);
    }

    // 3.2 Video / Audio extensions
    if matches!(
        ext.as_str(),
        "mp4" | "m4v" | "mov" | "mkv" | "webm" | "avi" | "mpg" | "mpeg"
    ) {
        return FileFormatInfo::new(FileCategory::Video, "video/mp4", "Video File", true);
    }
    if matches!(
        ext.as_str(),
        "mp3" | "wav" | "flac" | "ogg" | "opus" | "aac" | "m4a"
    ) {
        return FileFormatInfo::new(FileCategory::Audio, "audio/mpeg", "Audio File", true);
    }

    // 3.3 Database files
    if matches!(ext.as_str(), "db" | "sqlite" | "sqlite3") {
        return FileFormatInfo::new(
            FileCategory::Database,
            "application/vnd.sqlite3",
            "SQLite Database",
            true,
        );
    }

    // 3.4 Markdown & Documentation
    if matches!(ext.as_str(), "md" | "markdown" | "mdown" | "mkdn") {
        return FileFormatInfo::new(
            FileCategory::Markdown,
            "text/markdown",
            "Markdown Document",
            false,
        );
    }

    // 3.5 Structured Data
    if ext == "csv" {
        return FileFormatInfo::new(
            FileCategory::Data,
            "text/csv",
            "Comma-Separated Values (CSV)",
            false,
        );
    }
    if ext == "tsv" {
        return FileFormatInfo::new(
            FileCategory::Data,
            "text/tab-separated-values",
            "Tab-Separated Values (TSV)",
            false,
        );
    }
    if ext == "json" {
        return FileFormatInfo::new(FileCategory::Data, "application/json", "JSON Data", false);
    }
    if matches!(ext.as_str(), "jsonl" | "ndjson") {
        return FileFormatInfo::new(
            FileCategory::Data,
            "application/x-ndjson",
            "JSON Lines (JSONL)",
            false,
        );
    }
    if matches!(ext.as_str(), "yaml" | "yml") {
        return FileFormatInfo::new(
            FileCategory::Data,
            "application/yaml",
            "YAML Document",
            false,
        );
    }
    if ext == "toml" {
        return FileFormatInfo::new(
            FileCategory::Data,
            "application/toml",
            "TOML Document",
            false,
        );
    }
    if ext == "xml" {
        return FileFormatInfo::new(FileCategory::Data, "application/xml", "XML Document", false);
    }

    // 3.6 Source Code Formats
    let source_extensions = [
        "rs",
        "py",
        "c",
        "h",
        "cpp",
        "hpp",
        "cc",
        "cxx",
        "go",
        "js",
        "ts",
        "jsx",
        "tsx",
        "java",
        "kt",
        "kts",
        "sh",
        "bash",
        "zsh",
        "fish",
        "html",
        "htm",
        "css",
        "scss",
        "sql",
        "lua",
        "zig",
        "dart",
        "swift",
        "rb",
        "php",
        "r",
        "scala",
        "nim",
        "pl",
        "pm",
        "dockerfile",
        "makefile",
        "cmake",
        "ini",
        "conf",
        "env",
        "proto",
        "graphql",
    ];
    if source_extensions.contains(&ext.as_str()) {
        let mime = mime_guess::from_path(path)
            .first_or_text_plain()
            .to_string();
        return FileFormatInfo::new(FileCategory::SourceCode, mime, "Source Code", false);
    }

    // ── LAYER 4: HEURISTIC BINARY VS TEXT DETECTION ───────────────────────────────
    // If the file contains null bytes in its initial sample, it is binary
    let has_null_bytes = buf.contains(&0u8);
    let non_ascii_count = buf
        .iter()
        .filter(|&&b| b < 0x09 || (b > 0x0d && b < 0x20))
        .count();

    if has_null_bytes || (bytes_read > 0 && (non_ascii_count * 100 / bytes_read) > 30) {
        let mime = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();
        return FileFormatInfo::new(FileCategory::BinaryHex, mime, "Binary File", true);
    }

    // Default: Plain text
    let mime = mime_guess::from_path(path)
        .first_or_text_plain()
        .to_string();
    FileFormatInfo::new(FileCategory::SourceCode, mime, "Plain Text", false)
}
