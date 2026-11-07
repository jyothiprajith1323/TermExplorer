//! Content-based and extension-based file format detection.

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

/// Sniff file category and MIME info from magic bytes and extension.
pub fn detect_file_format(path: &Path) -> FileFormatInfo {
    // Read up to first 512 bytes for magic number analysis
    let mut header = [0u8; 512];
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

    // 1. SQLite Database: magic "SQLite format 3\0"
    if buf.starts_with(b"SQLite format 3\0") || ext == "db" || ext == "sqlite" || ext == "sqlite3" {
        return FileFormatInfo::new(
            FileCategory::Database,
            "application/vnd.sqlite3",
            "SQLite Database",
            true,
        );
    }

    // 2. Images
    if buf.starts_with(b"\x89PNG\r\n\x1a\n") || ext == "png" {
        return FileFormatInfo::new(FileCategory::Image, "image/png", "PNG Image", true);
    }
    if buf.starts_with(&[0xff, 0xd8, 0xff]) || ext == "jpg" || ext == "jpeg" {
        return FileFormatInfo::new(FileCategory::Image, "image/jpeg", "JPEG Image", true);
    }
    if buf.starts_with(b"GIF87a") || buf.starts_with(b"GIF89a") || ext == "gif" {
        return FileFormatInfo::new(
            FileCategory::Image,
            "image/gif",
            "GIF Animation/Image",
            true,
        );
    }
    if (buf.starts_with(b"RIFF") && buf.len() > 12 && &buf[8..12] == b"WEBP") || ext == "webp" {
        return FileFormatInfo::new(FileCategory::Image, "image/webp", "WebP Image", true);
    }
    if buf.starts_with(b"BM") || ext == "bmp" {
        return FileFormatInfo::new(FileCategory::Image, "image/bmp", "BMP Image", true);
    }
    if ext == "svg" {
        return FileFormatInfo::new(
            FileCategory::Image,
            "image/svg+xml",
            "Scalable Vector Graphics (SVG)",
            false,
        );
    }
    if ext == "ico" || buf.starts_with(&[0x00, 0x00, 0x01, 0x00]) {
        return FileFormatInfo::new(FileCategory::Image, "image/x-icon", "Icon (ICO)", true);
    }
    if ext == "tiff" || ext == "tif" || buf.starts_with(b"II*\x00") || buf.starts_with(b"MM\x00*") {
        return FileFormatInfo::new(FileCategory::Image, "image/tiff", "TIFF Image", true);
    }
    if ext == "avif" {
        return FileFormatInfo::new(FileCategory::Image, "image/avif", "AVIF Image", true);
    }

    // 3. PDF
    if buf.starts_with(b"%PDF-") || ext == "pdf" {
        return FileFormatInfo::new(FileCategory::Pdf, "application/pdf", "PDF Document", true);
    }

    // 4. Video containers
    if ext == "mp4" || ext == "m4v" || (buf.len() >= 8 && &buf[4..8] == b"ftyp") {
        return FileFormatInfo::new(FileCategory::Video, "video/mp4", "MP4 Video", true);
    }
    if ext == "mkv" || ext == "webm" || buf.starts_with(&[0x1a, 0x45, 0xdf, 0xa3]) {
        return FileFormatInfo::new(
            FileCategory::Video,
            "video/webm",
            "Matroska / WebM Video",
            true,
        );
    }
    if ext == "mov" {
        return FileFormatInfo::new(
            FileCategory::Video,
            "video/quicktime",
            "QuickTime Video",
            true,
        );
    }
    if ext == "avi" || (buf.starts_with(b"RIFF") && buf.len() > 12 && &buf[8..12] == b"AVI ") {
        return FileFormatInfo::new(FileCategory::Video, "video/x-msvideo", "AVI Video", true);
    }

    // 5. Audio files
    if buf.starts_with(b"ID3") || ext == "mp3" {
        return FileFormatInfo::new(FileCategory::Audio, "audio/mpeg", "MP3 Audio", true);
    }
    if (buf.starts_with(b"RIFF") && buf.len() > 12 && &buf[8..12] == b"WAVE") || ext == "wav" {
        return FileFormatInfo::new(FileCategory::Audio, "audio/wav", "WAV Audio", true);
    }
    if buf.starts_with(b"fLaC") || ext == "flac" {
        return FileFormatInfo::new(FileCategory::Audio, "audio/flac", "FLAC Audio", true);
    }
    if buf.starts_with(b"OggS") || ext == "ogg" || ext == "opus" {
        return FileFormatInfo::new(FileCategory::Audio, "audio/ogg", "Ogg / Opus Audio", true);
    }
    if ext == "aac" || ext == "m4a" {
        return FileFormatInfo::new(FileCategory::Audio, "audio/aac", "AAC Audio", true);
    }

    // 6. Office documents (Word, Excel, PowerPoint, OpenDocument)
    if ext == "docx" {
        return FileFormatInfo::new(
            FileCategory::Office,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "Microsoft Word Document (DOCX)",
            true,
        );
    }
    if ext == "xlsx" {
        return FileFormatInfo::new(
            FileCategory::Office,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "Microsoft Excel Spreadsheet (XLSX)",
            true,
        );
    }
    if ext == "pptx" {
        return FileFormatInfo::new(
            FileCategory::Office,
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            "Microsoft PowerPoint Presentation (PPTX)",
            true,
        );
    }
    if ext == "odt" {
        return FileFormatInfo::new(
            FileCategory::Office,
            "application/vnd.oasis.opendocument.text",
            "OpenDocument Text (ODT)",
            true,
        );
    }
    if ext == "ods" {
        return FileFormatInfo::new(
            FileCategory::Office,
            "application/vnd.oasis.opendocument.spreadsheet",
            "OpenDocument Spreadsheet (ODS)",
            true,
        );
    }
    if ext == "odp" {
        return FileFormatInfo::new(
            FileCategory::Office,
            "application/vnd.oasis.opendocument.presentation",
            "OpenDocument Presentation (ODP)",
            true,
        );
    }
    if ext == "doc" || ext == "xls" || ext == "ppt" {
        return FileFormatInfo::new(
            FileCategory::Office,
            "application/msword",
            "Legacy Office Document",
            true,
        );
    }

    // 7. Archives and compressed files
    if buf.starts_with(b"PK\x03\x04") || ext == "zip" {
        return FileFormatInfo::new(
            FileCategory::Archive,
            "application/zip",
            "ZIP Archive",
            true,
        );
    }
    if buf.starts_with(&[0x1f, 0x8b]) || ext == "gz" || ext == "tgz" {
        return FileFormatInfo::new(
            FileCategory::Archive,
            "application/gzip",
            "Gzip Compressed Archive",
            true,
        );
    }
    if buf.starts_with(b"BZh") || ext == "bz2" {
        return FileFormatInfo::new(
            FileCategory::Archive,
            "application/x-bzip2",
            "Bzip2 Compressed Archive",
            true,
        );
    }
    if buf.starts_with(&[0xfd, b'7', b'z', b'X', b'Z', 0x00]) || ext == "xz" {
        return FileFormatInfo::new(
            FileCategory::Archive,
            "application/x-xz",
            "XZ Compressed Archive",
            true,
        );
    }
    if buf.starts_with(b"7z\xbc\xaf\x27\x1c") || ext == "7z" {
        return FileFormatInfo::new(
            FileCategory::Archive,
            "application/x-7z-compressed",
            "7-Zip Archive",
            true,
        );
    }
    if ext == "tar" || (buf.len() >= 262 && &buf[257..262] == b"ustar") {
        return FileFormatInfo::new(
            FileCategory::Archive,
            "application/x-tar",
            "Tarball Archive",
            true,
        );
    }

    // 8. Markdown and Documentation
    if ext == "md" || ext == "markdown" || ext == "mdown" {
        return FileFormatInfo::new(
            FileCategory::Markdown,
            "text/markdown",
            "Markdown Document",
            false,
        );
    }

    // 9. Structured Data files
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
    if ext == "jsonl" || ext == "ndjson" {
        return FileFormatInfo::new(
            FileCategory::Data,
            "application/x-ndjson",
            "JSON Lines (JSONL)",
            false,
        );
    }
    if ext == "yaml" || ext == "yml" {
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

    // 10. Source Code and Configuration
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
        "php",
        "rb",
        "r",
        "swift",
        "scala",
        "dart",
        "zig",
        "ex",
        "exs",
        "erl",
        "hs",
        "lhs",
        "clj",
        "cljs",
        "diff",
        "patch",
        "dockerfile",
        "makefile",
        "ini",
        "conf",
        "cfg",
        "env",
        "log",
        "txt",
        "ron",
        "proto",
        "graphql",
    ];
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();

    if source_extensions.contains(&ext.as_str())
        || file_name == "dockerfile"
        || file_name == "makefile"
        || file_name == "cmakelists.txt"
        || file_name.starts_with(".env")
    {
        return FileFormatInfo::new(
            FileCategory::SourceCode,
            "text/plain",
            "Source Code / Text",
            false,
        );
    }

    // 11. Binary Sniffing: Null-byte check in the first 512 bytes
    let has_null_bytes = buf.iter().take(512).any(|&b| b == 0);
    if has_null_bytes {
        // Unknown binary format -> Hex Viewer
        let mime = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();
        return FileFormatInfo::new(FileCategory::BinaryHex, mime, "Binary File", true);
    }

    // 12. Fallback to Source/Text if valid UTF-8
    if std::str::from_utf8(buf).is_ok() {
        FileFormatInfo::new(FileCategory::SourceCode, "text/plain", "Text File", false)
    } else {
        FileFormatInfo::new(
            FileCategory::BinaryHex,
            "application/octet-stream",
            "Binary Data",
            true,
        )
    }
}
