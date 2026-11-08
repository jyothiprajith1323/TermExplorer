//! Incremental syntax highlighting engine with broad language support,
//! line-level token caching, and canonical Catppuccin Mocha theme tokens.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;
use std::sync::{OnceLock, RwLock};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color as SynColor, Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// A highlighted span: (foreground RGB: r, g, b, text content).
pub type Span = (u8, u8, u8, String);

/// Maximum file line count for full syntax highlighting to preserve 60 FPS.
pub const MAX_SYNTAX_LINES: usize = 30_000;

// ── Canonical Catppuccin Mocha Syntax Tokens ────────────────────────────────

pub mod mocha {
    pub const BASE: (u8, u8, u8) = (30, 30, 46); // #1e1e2e
    pub const TEXT: (u8, u8, u8) = (205, 214, 244); // #cdd6f4 (identifiers, default text)
    pub const SUBTEXT0: (u8, u8, u8) = (166, 173, 200); // #a6adc8 (secondary text)
    pub const OVERLAY0: (u8, u8, u8) = (108, 112, 134); // #6c7086 (comments)
    pub const BLUE: (u8, u8, u8) = (137, 180, 250); // #89b4fa (functions, methods)
    pub const MAUVE: (u8, u8, u8) = (203, 166, 247); // #cba6f7 (keywords, control flow)
    pub const GREEN: (u8, u8, u8) = (166, 227, 161); // #a6e3a1 (strings, characters)
    pub const PEACH: (u8, u8, u8) = (250, 179, 135); // #fab387 (numbers, constants, booleans)
    pub const YELLOW: (u8, u8, u8) = (249, 226, 175); // #f9e2af (types, structs, classes)
    pub const RED: (u8, u8, u8) = (243, 139, 168); // #f38ba8 (errors, delimiters)
    pub const TEAL: (u8, u8, u8) = (148, 226, 213); // #94e2d5 (macros, attributes)
    pub const SKY: (u8, u8, u8) = (116, 199, 236); // #74c7ec (operators, symbols)
}

/// Map a Syntect RGB style color to the nearest semantic Catppuccin Mocha token.
pub fn syntect_to_mocha(fg: SynColor) -> (u8, u8, u8) {
    let r = fg.r;
    let g = fg.g;
    let b = fg.b;

    // Comments / Grayscale low saturation
    let max_val = r.max(g).max(b);
    let min_val = r.min(g).min(b);
    let delta = max_val - min_val;

    if delta < 18 {
        if max_val < 140 {
            return mocha::OVERLAY0; // comments
        } else {
            return mocha::TEXT; // neutral text
        }
    }

    // Hue detection for semantic tokens
    if r > 180 && g < 120 && b < 140 {
        mocha::RED // error / tag
    } else if r > 180 && g > 110 && b < 100 {
        mocha::PEACH // numbers / constants
    } else if r > 160 && g > 150 && b < 110 {
        mocha::YELLOW // types / classes
    } else if g > 140 && g >= r && g >= b {
        mocha::GREEN // strings
    } else if b > 150 && g > 130 && r < 120 {
        mocha::TEAL // macros / attributes
    } else if b > 160 && g >= 110 && r < 130 {
        mocha::SKY // operators
    } else if b > 160 && r < 150 {
        mocha::BLUE // functions / methods
    } else if r > 150 && b > 150 && g < 140 {
        mocha::MAUVE // keywords
    } else {
        mocha::TEXT
    }
}

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

// ── Incremental Line-Level Syntax Cache ─────────────────────────────────────

#[derive(Debug, Clone)]
struct CachedLine {
    line_hash: u64,
    spans: Vec<Span>,
}

static SYNTAX_CACHE: OnceLock<RwLock<HashMap<(u64, usize), CachedLine>>> = OnceLock::new();

fn get_syntax_cache() -> &'static RwLock<HashMap<(u64, usize), CachedLine>> {
    SYNTAX_CACHE.get_or_init(|| RwLock::new(HashMap::with_capacity(1024)))
}

fn hash_path(path: Option<&Path>) -> u64 {
    let mut hasher = DefaultHasher::new();
    if let Some(p) = path {
        p.hash(&mut hasher);
    } else {
        "untitled".hash(&mut hasher);
    }
    hasher.finish()
}

fn hash_str(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

/// Map file extension or filename to known syntect syntax name or alias.
pub fn resolve_syntax_alias(ext_or_name: &str) -> &'static str {
    match ext_or_name.to_lowercase().as_str() {
        // Priority 1: Rust
        "rs" => "Rust",
        // Priority 2: Python
        "py" | "python" | "pyw" => "Python",
        // Priority 3 & 4: C and C++
        "c" | "h" => "C",
        "cpp" | "hpp" | "cc" | "cxx" | "c++" | "h++" => "C++",
        // Priority 5: Java
        "java" => "Java",
        // Priority 6: SQL
        "sql" => "SQL",
        // Priority 7: TOML
        "toml" => "INI",
        // Priority 8: Markdown
        "md" | "markdown" => "Markdown",
        // Priority 9: JSON
        "json" => "JSON",
        // Priority 10: YAML
        "yaml" | "yml" => "YAML",
        // Priority 11: JavaScript
        "js" | "mjs" | "cjs" => "JavaScript",
        // Priority 12: TypeScript
        "ts" | "tsx" | "jsx" => "JavaScript",
        // Priority 13: Bash
        "sh" | "bash" | "zsh" | "fish" | "ksh" => "Bourne Again Shell (bash)",
        // Priority 14: Go
        "go" => "Go",
        // Priority 15: HTML & CSS
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

/// Highlight an entire file's content string for a path with Catppuccin Mocha tokens.
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
                let (r, g, b) = syntect_to_mocha(style.foreground);
                (r, g, b, text.trim_end_matches('\n').to_string())
            })
            .collect();
        result.push(spans);
    }
    result
}

/// Highlight only the requested viewport range of lines incrementally with line-level caching.
/// Fast, non-blocking, and maps tokens to the canonical Catppuccin Mocha palette.
pub fn highlight_viewport(
    path: Option<&Path>,
    lines: &[String],
    start_row: usize,
    count: usize,
) -> Vec<Vec<Span>> {
    if lines.is_empty() {
        return Vec::new();
    }

    // Graceful large-file fallback
    if lines.len() > MAX_SYNTAX_LINES {
        return lines
            .iter()
            .skip(start_row)
            .take(count)
            .map(|l| vec![(mocha::TEXT.0, mocha::TEXT.1, mocha::TEXT.2, l.clone())])
            .collect();
    }

    let p_hash = hash_path(path);
    let cache_lock = get_syntax_cache();

    // Check if entire viewport is already cached
    let mut all_cached = true;
    let mut cached_results = Vec::with_capacity(count);

    if let Ok(cache) = cache_lock.read() {
        for row in start_row..start_row + count {
            if let Some(line) = lines.get(row) {
                let l_hash = hash_str(line);
                if let Some(entry) = cache.get(&(p_hash, row)) {
                    if entry.line_hash == l_hash {
                        cached_results.push(entry.spans.clone());
                        continue;
                    }
                }
                all_cached = false;
                break;
            } else {
                break;
            }
        }
    } else {
        all_cached = false;
    }

    if all_cached && !cached_results.is_empty() {
        return cached_results;
    }

    // Compute syntax highlighting
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

    // Warm up state up to start_row for accurate multi-line parsing
    let warmup_start = start_row.saturating_sub(100);
    for row in warmup_start..start_row {
        if let Some(line) = lines.get(row) {
            let _ = hl.highlight_line(&format!("{line}\n"), &h.ss);
        }
    }

    let mut new_entries = Vec::with_capacity(count);

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
                    let (r, g, b) = syntect_to_mocha(style.foreground);
                    (r, g, b, text.trim_end_matches('\n').to_string())
                })
                .collect();

            let l_hash = hash_str(line);
            new_entries.push((
                (p_hash, row),
                CachedLine {
                    line_hash: l_hash,
                    spans: spans.clone(),
                },
            ));
            result.push(spans);
        } else {
            break;
        }
    }

    // Populate incremental cache
    if let Ok(mut cache) = cache_lock.write() {
        if cache.len() > 8192 {
            cache.clear();
        }
        for (k, v) in new_entries {
            cache.insert(k, v);
        }
    }

    result
}

/// Fallback plain lines with specified color.
pub fn plain_lines(lines: &[String], color: (u8, u8, u8)) -> Vec<Vec<Span>> {
    lines
        .iter()
        .map(|l| vec![(color.0, color.1, color.2, l.clone())])
        .collect()
}
