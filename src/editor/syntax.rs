//! Syntax highlighting using syntect.

use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// A highlighted span: (foreground RGB, text).
pub type Span = (u8, u8, u8, String);

pub struct Highlighter {
    ss: SyntaxSet,
    ts: ThemeSet,
}

// Use a simple once-cell to avoid re-loading on every render.
use std::sync::OnceLock;
static HIGHLIGHTER: OnceLock<Highlighter> = OnceLock::new();

fn get_highlighter() -> &'static Highlighter {
    HIGHLIGHTER.get_or_init(|| Highlighter {
        ss: SyntaxSet::load_defaults_newlines(),
        ts: ThemeSet::load_defaults(),
    })
}

/// Highlight `content` for the file at `path`.
/// Returns lines of spans. If highlighting is unavailable, returns plain text spans.
pub fn highlight_file(path: &Path, content: &str) -> Vec<Vec<Span>> {
    let h = get_highlighter();
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("txt");

    let syntax =
        h.ss.find_syntax_by_extension(ext)
            .or_else(|| h.ss.find_syntax_by_name("Plain Text"))
            .unwrap_or_else(|| h.ss.find_syntax_plain_text());

    // "base16-ocean.dark" is a beautiful dark theme.
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

/// Strip highlighting — just return lines of plain-text spans with the given colour.
#[allow(dead_code)]
pub fn plain_lines(content: &str, r: u8, g: u8, b: u8) -> Vec<Vec<Span>> {
    content
        .lines()
        .map(|l| vec![(r, g, b, l.to_string())])
        .collect()
}
