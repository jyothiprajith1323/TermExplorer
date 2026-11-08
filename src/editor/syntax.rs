//! Syntax highlighting engine using syntect with broad language support
//! and Catppuccin Mocha theme integration.

use std::path::Path;
use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// A highlighted span: (foreground RGB: r, g, b, text content).
pub type Span = (u8, u8, u8, String);

pub struct Highlighter {
    ss: SyntaxSet,
    ts: ThemeSet,
}

static HIGHLIGHTER: OnceLock<Highlighter> = OnceLock::new();

pub fn get_highlighter() -> &'static Highlighter {
    HIGHLIGHTER.get_or_init(|| Highlighter {
        ss: SyntaxSet::load_defaults_newlines(),
        ts: ThemeSet::load_defaults(),
    })
}

/// Map file extension or filename to known syntect syntax name or alias.
pub fn resolve_syntax_alias(ext_or_name: &str) -> &'static str {
    match ext_or_name.to_lowercase().as_str() {
        "rs" => "Rust",
        "py" | "python" | "pyw" => "Python",
        "c" | "h" => "C",
        "cpp" | "hpp" | "cc" | "cxx" | "c++" | "h++" => "C++",
        "java" => "Java",
        "sql" => "SQL",
        "toml" => "INI",
        "md" | "markdown" => "Markdown",
        "json" => "JSON",
        "yaml" | "yml" => "YAML",
        "js" | "mjs" | "cjs" => "JavaScript",
        "ts" | "tsx" | "jsx" => "JavaScript",
        "go" => "Go",
        "sh" | "bash" | "zsh" | "fish" | "ksh" => "Bourne Again Shell (bash)",
        "html" | "htm" | "xhtml" => "HTML",
        "css" | "scss" | "sass" | "less" => "CSS",
        "xml" | "svg" | "plist" => "XML",
        "ini" | "cfg" | "conf" | "desktop" => "INI",
        "makefile" | "make" | "mk" => "Makefile",
        "diff" | "patch" => "Diff",
        "dockerfile" => "Bourne Again Shell (bash)",
        _ => "Plain Text",
    }
}

/// Highlight an entire file's content string for a path.
pub fn highlight_file(path: &Path, content: &str) -> Vec<Vec<Span>> {
    let h = get_highlighter();
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    let syntax =
        h.ss.find_syntax_by_extension(ext)
            .or_else(|| {
                let alias = resolve_syntax_alias(ext);
                h.ss.find_syntax_by_name(alias)
            })
            .or_else(|| {
                let alias = resolve_syntax_alias(file_name);
                h.ss.find_syntax_by_name(alias)
            })
            .or_else(|| h.ss.find_syntax_by_first_line(content))
            .unwrap_or_else(|| h.ss.find_syntax_plain_text());

    let theme_name = "base16-ocean.dark";
    let theme =
        h.ts.themes
            .get(theme_name)
            .or_else(|| h.ts.themes.values().next())
            .unwrap();

    let mut hl = HighlightLines::new(syntax, theme);
    let mut result = Vec::new();

    for line in LinesWithEndings::from(content) {
        let ranges: Vec<(Style, &str)> = match hl.highlight_line(line, &h.ss) {
            Ok(r) => r,
            Err(_) => vec![(Style::default(), line)],
        };
        let spans: Vec<Span> = ranges
            .into_iter()
            .map(|(style, text)| {
                let fg = style.foreground;
                (fg.r, fg.g, fg.b, text.trim_end_matches('\n').to_string())
            })
            .collect();
        result.push(spans);
    }
    result
}

/// Highlight only the requested viewport range of lines from a line buffer.
/// Fast and non-blocking for large documents.
pub fn highlight_viewport(
    path: Option<&Path>,
    lines: &[String],
    start_row: usize,
    count: usize,
) -> Vec<Vec<Span>> {
    if lines.is_empty() {
        return Vec::new();
    }

    let h = get_highlighter();
    let ext = path
        .and_then(|p| p.extension())
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let file_name = path
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("");

    let syntax =
        h.ss.find_syntax_by_extension(ext)
            .or_else(|| {
                let alias = resolve_syntax_alias(ext);
                h.ss.find_syntax_by_name(alias)
            })
            .or_else(|| {
                let alias = resolve_syntax_alias(file_name);
                h.ss.find_syntax_by_name(alias)
            })
            .unwrap_or_else(|| h.ss.find_syntax_plain_text());

    let theme_name = "base16-ocean.dark";
    let theme =
        h.ts.themes
            .get(theme_name)
            .or_else(|| h.ts.themes.values().next())
            .unwrap();

    let mut hl = HighlightLines::new(syntax, theme);
    let mut result = Vec::with_capacity(count);

    // Warm up highlighter state up to start_row for accurate multi-line parsing
    let warmup_start = start_row.saturating_sub(100);
    for row in warmup_start..start_row {
        if let Some(line) = lines.get(row) {
            let _ = hl.highlight_line(&format!("{line}\n"), &h.ss);
        }
    }

    for row in start_row..start_row + count {
        if let Some(line) = lines.get(row) {
            let line_ending_str = format!("{line}\n");
            let ranges: Vec<(Style, &str)> = match hl.highlight_line(&line_ending_str, &h.ss) {
                Ok(r) => r,
                Err(_) => vec![(Style::default(), &line_ending_str)],
            };
            let spans: Vec<Span> = ranges
                .into_iter()
                .map(|(style, text)| {
                    let fg = style.foreground;
                    (fg.r, fg.g, fg.b, text.trim_end_matches('\n').to_string())
                })
                .collect();
            result.push(spans);
        } else {
            break;
        }
    }

    result
}

/// Fallback plain lines with specified color.
pub fn plain_lines(content: &str, r: u8, g: u8, b: u8) -> Vec<Vec<Span>> {
    content
        .lines()
        .map(|l| vec![(r, g, b, l.to_string())])
        .collect()
}
