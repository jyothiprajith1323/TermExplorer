//! Built-in icon mappings for folders, filenames, and extensions.

use ratatui::style::Color;

use super::glyphs::*;
use super::pack::IconPack;
use crate::theme::palette::CatppuccinPalette;

pub fn resolve_folder_icon(
    name: &str,
    is_open: bool,
    pack: IconPack,
    palette: &CatppuccinPalette,
) -> (String, Color) {
    if pack == IconPack::None {
        return (String::new(), palette.blue);
    }

    let lower = name.to_ascii_lowercase();
    let (glyph_set, color) = match lower.as_str() {
        "src" | "source" => (FOLDER_SRC, palette.blue),
        "tests" | "test" | "__tests__" => (FOLDER_TEST, palette.green),
        "docs" | "doc" | "documentation" => (FOLDER_DOCS, palette.yellow),
        "target" | "build" | "dist" | "out" | "bin" => (FOLDER_BUILD, palette.peach),
        ".git" | ".github" | ".gitlab" => (FOLDER_GIT, palette.red),
        "config" | ".config" | "settings" => (FOLDER_CONFIG, palette.teal),
        "assets" | "static" | "public" | "images" => (FOLDER_ASSETS, palette.lavender),
        _ => {
            if is_open {
                (FOLDER_OPEN, palette.blue)
            } else {
                (FOLDER_CLOSED, palette.blue)
            }
        }
    };

    let symbol = match pack {
        IconPack::Catppuccin | IconPack::NerdFont => glyph_set.nerd,
        IconPack::Unicode => glyph_set.unicode,
        IconPack::Ascii => glyph_set.ascii,
        IconPack::None => "",
    };

    let col = match pack {
        IconPack::Catppuccin => color,
        IconPack::NerdFont => palette.blue,
        IconPack::Unicode | IconPack::Ascii | IconPack::None => palette.text,
    };

    (format!("{} ", symbol.trim()), col)
}

pub fn resolve_file_icon(
    filename: &str,
    extension: Option<&str>,
    pack: IconPack,
    palette: &CatppuccinPalette,
) -> (String, Color) {
    if pack == IconPack::None {
        return (String::new(), palette.text);
    }

    let lower_name = filename.to_ascii_lowercase();

    // 1. Exact filename matches
    let exact_match: Option<(GlyphSet, Color)> = match lower_name.as_str() {
        "cargo.toml" => Some((FILE_RUST, palette.maroon)),
        "cargo.lock" => Some((FILE_LOCK, palette.overlay2)),
        "package.json" => Some((FILE_JAVASCRIPT, palette.green)),
        "package-lock.json" | "yarn.lock" | "pnpm-lock.yaml" => Some((FILE_LOCK, palette.overlay2)),
        "tsconfig.json" => Some((FILE_TYPESCRIPT, palette.blue)),
        "dockerfile" | "containerfile" => Some((FILE_DOCKER, palette.blue)),
        "docker-compose.yml" | "docker-compose.yaml" => Some((FILE_DOCKER, palette.sapphire)),
        "makefile" | "cmakelists.txt" => Some((FILE_SPECIAL, palette.peach)),
        ".gitignore" | ".gitmodules" | ".gitattributes" => Some((FILE_GIT, palette.red)),
        ".env" | ".env.local" | ".env.production" => Some((FILE_SPECIAL, palette.yellow)),
        "readme.md" | "readme" => Some((FILE_MARKDOWN, palette.teal)),
        "license" | "licence" | "copying" => Some((FILE_GENERIC, palette.yellow)),
        _ => None,
    };

    let (glyph_set, color) = if let Some(m) = exact_match {
        m
    } else {
        // 2. Extension matches
        match extension.unwrap_or("").to_ascii_lowercase().as_str() {
            "rs" => (FILE_RUST, palette.maroon),
            "py" | "pyw" | "ipynb" => (FILE_PYTHON, palette.blue),
            "js" | "mjs" | "cjs" => (FILE_JAVASCRIPT, palette.yellow),
            "ts" | "mts" | "cts" => (FILE_TYPESCRIPT, palette.sapphire),
            "tsx" | "jsx" => (FILE_REACT, palette.sky),
            "c" | "h" => (FILE_C, palette.blue),
            "cpp" | "cxx" | "cc" | "hpp" | "hxx" => (FILE_CPP, palette.sapphire),
            "go" => (FILE_GO, palette.teal),
            "java" | "class" | "jar" => (FILE_JAVA, palette.red),
            "sh" | "bash" | "zsh" | "fish" => (FILE_SHELL, palette.green),
            "html" | "htm" => (FILE_HTML, palette.peach),
            "css" | "scss" | "sass" | "less" => (FILE_CSS, palette.blue),
            "md" | "markdown" | "rst" => (FILE_MARKDOWN, palette.teal),
            "toml" => (FILE_TOML, palette.peach),
            "yaml" | "yml" => (FILE_YAML, palette.peach),
            "json" => (FILE_JSON, palette.yellow),
            "sql" => (FILE_SQL, palette.mauve),
            "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico" | "bmp" => {
                (FILE_IMAGE, palette.pink)
            }
            "mp3" | "wav" | "flac" | "ogg" | "m4a" => (FILE_AUDIO, palette.flamingo),
            "mp4" | "mkv" | "avi" | "mov" | "webm" => (FILE_VIDEO, palette.mauve),
            "zip" | "tar" | "gz" | "xz" | "bz2" | "7z" | "rar" => (FILE_ARCHIVE, palette.peach),
            "pdf" => (FILE_PDF, palette.red),
            "wasm" | "wat" => (FILE_WASM, palette.mauve),
            "lock" => (FILE_LOCK, palette.overlay2),
            _ => (FILE_GENERIC, palette.text),
        }
    };

    let symbol = match pack {
        IconPack::Catppuccin | IconPack::NerdFont => glyph_set.nerd,
        IconPack::Unicode => glyph_set.unicode,
        IconPack::Ascii => glyph_set.ascii,
        IconPack::None => "",
    };

    let col = match pack {
        IconPack::Catppuccin => color,
        IconPack::NerdFont => palette.text,
        IconPack::Unicode | IconPack::Ascii | IconPack::None => palette.text,
    };

    (format!("{} ", symbol.trim()), col)
}
