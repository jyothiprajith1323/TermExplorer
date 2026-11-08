//! Incremental syntax highlighting engine with broad language support,
//! line-level token caching, and canonical Catppuccin Mocha theme tokens.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;
use std::sync::{OnceLock, RwLock};
use syntect::easy::HighlightLines;
use syntect::highlighting::{
    Color as SynColor, HighlightIterator, HighlightState, Highlighter as SynHighlighter, Style,
    ThemeSet,
};
use syntect::parsing::{ParseState, ScopeStack, SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

#[derive(Clone)]
pub struct LineSyntaxState {
    pub parse_state: ParseState,
    pub highlight_state: HighlightState,
}

impl LineSyntaxState {
    pub fn new(syntax: &SyntaxReference, highlighter: &SynHighlighter) -> Self {
        let parse_state = ParseState::new(syntax);
        let highlight_state = HighlightState::new(highlighter, ScopeStack::new());
        Self {
            parse_state,
            highlight_state,
        }
    }

    pub fn highlight_line<'a>(
        &mut self,
        line: &'a str,
        syntax_set: &SyntaxSet,
        highlighter: &SynHighlighter,
    ) -> Vec<(Style, &'a str)> {
        match self.parse_state.parse_line(line, syntax_set) {
            Ok(ops) => {
                let iter =
                    HighlightIterator::new(&mut self.highlight_state, &ops[..], line, highlighter);
                iter.collect()
            }
            Err(_) => vec![(Style::default(), line)],
        }
    }
}

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

    // Comments / Grayscale / Muted text (Base16-ocean comment is #65737e: 101, 115, 126 -> delta=25)
    if delta <= 28 {
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
    } else if r > 150 && b > 150 && g < 160 {
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

// ── Incremental Line-Level Syntax Cache with LRU Eviction & Checkpoints ────

pub const MAX_SYNTAX_CACHE_ENTRIES: usize = 4096;
pub const CHECKPOINT_INTERVAL: usize = 64;

#[derive(Debug, Clone)]
pub struct CachedLine {
    pub line_hash: u64,
    pub spans: Vec<Span>,
    pub access_tick: u64,
}

pub struct BoundedSyntaxCache {
    entries: HashMap<(u64, usize), CachedLine>,
    max_entries: usize,
    clock: u64,
}

impl BoundedSyntaxCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(max_entries),
            max_entries,
            clock: 0,
        }
    }

    pub fn get(&mut self, key: &(u64, usize), current_line_hash: u64) -> Option<Vec<Span>> {
        self.clock = self.clock.wrapping_add(1);
        if let Some(entry) = self.entries.get_mut(key) {
            if entry.line_hash == current_line_hash {
                entry.access_tick = self.clock;
                return Some(entry.spans.clone());
            }
        }
        None
    }

    pub fn insert(&mut self, key: (u64, usize), line_hash: u64, spans: Vec<Span>) {
        self.clock = self.clock.wrapping_add(1);
        if self.entries.len() >= self.max_entries {
            // Evict oldest 25% of entries by access_tick
            let evict_count = self.max_entries / 4;
            let mut items: Vec<((u64, usize), u64)> = self
                .entries
                .iter()
                .map(|(&k, v)| (k, v.access_tick))
                .collect();
            items.sort_by_key(|&(_, tick)| tick);
            for (k, _) in items.into_iter().take(evict_count) {
                self.entries.remove(&k);
            }
        }
        self.entries.insert(
            key,
            CachedLine {
                line_hash,
                spans,
                access_tick: self.clock,
            },
        );
    }

    pub fn invalidate_from_line(&mut self, path_hash: u64, from_line: usize) {
        self.entries
            .retain(|&(p, row), _| p != path_hash || row < from_line);
    }

    pub fn invalidate_path(&mut self, path_hash: u64) {
        self.entries.retain(|&(p, _), _| p != path_hash);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

static SYNTAX_CACHE: OnceLock<RwLock<BoundedSyntaxCache>> = OnceLock::new();
static SYNTAX_CHECKPOINTS: OnceLock<
    RwLock<HashMap<u64, std::collections::BTreeMap<usize, LineSyntaxState>>>,
> = OnceLock::new();

fn get_syntax_cache() -> &'static RwLock<BoundedSyntaxCache> {
    SYNTAX_CACHE.get_or_init(|| RwLock::new(BoundedSyntaxCache::new(MAX_SYNTAX_CACHE_ENTRIES)))
}

fn get_checkpoints(
) -> &'static RwLock<HashMap<u64, std::collections::BTreeMap<usize, LineSyntaxState>>> {
    SYNTAX_CHECKPOINTS.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Invalidate cache and syntax checkpoints for a given path from a starting line index.
pub fn invalidate_syntax_from_line(path: Option<&Path>, from_line: usize) {
    let p_hash = hash_path(path);
    if let Ok(mut cache) = get_syntax_cache().write() {
        cache.invalidate_from_line(p_hash, from_line);
    }
    if let Ok(mut cp_map) = get_checkpoints().write() {
        if let Some(tree) = cp_map.get_mut(&p_hash) {
            let keys_to_remove: Vec<usize> = tree.range(from_line..).map(|(&k, _)| k).collect();
            for k in keys_to_remove {
                tree.remove(&k);
            }
        }
    }
}

/// Invalidate entire syntax cache and checkpoints for a given path (e.g. tab close, rename).
pub fn invalidate_syntax_cache_for_path(path: Option<&Path>) {
    let p_hash = hash_path(path);
    if let Ok(mut cache) = get_syntax_cache().write() {
        cache.invalidate_path(p_hash);
    }
    if let Ok(mut cp_map) = get_checkpoints().write() {
        cp_map.remove(&p_hash);
    }
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

    // Check if entire viewport is already cached
    let mut all_cached = true;
    let mut cached_results = Vec::with_capacity(count);

    if let Ok(mut cache) = get_syntax_cache().write() {
        for row in start_row..start_row + count {
            if let Some(line) = lines.get(row) {
                let l_hash = hash_str(line);
                if let Some(spans) = cache.get(&(p_hash, row), l_hash) {
                    cached_results.push(spans);
                } else {
                    all_cached = false;
                    break;
                }
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

    // Compute syntax highlighting using verified checkpoints for 100% multiline correctness
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
    let syn_highlighter = SynHighlighter::new(theme);

    // Retrieve closest verified checkpoint <= start_row
    let (mut current_row, mut state) = {
        let mut found = None;
        if let Ok(cp_map) = get_checkpoints().read() {
            if let Some(tree) = cp_map.get(&p_hash) {
                if let Some((&cp_line, cp_state)) = tree.range(..=start_row).next_back() {
                    found = Some((cp_line, cp_state.clone()));
                }
            }
        }
        found.unwrap_or_else(|| (0, LineSyntaxState::new(syntax, &syn_highlighter)))
    };

    // Advance state from checkpoint up to start_row, capturing intermediate checkpoints
    let mut intermediate_checkpoints = Vec::new();
    while current_row < start_row {
        if let Some(line) = lines.get(current_row) {
            let line_with_nl = format!("{line}\n");
            let _ = state.highlight_line(&line_with_nl, &h.ss, &syn_highlighter);
            current_row += 1;
            if current_row % CHECKPOINT_INTERVAL == 0 {
                intermediate_checkpoints.push((current_row, state.clone()));
            }
        } else {
            break;
        }
    }

    let mut result = Vec::with_capacity(count);
    let mut new_entries = Vec::with_capacity(count);

    for row in start_row..start_row + count {
        if let Some(line) = lines.get(row) {
            let line_ending_str = format!("{line}\n");
            let ranges = state.highlight_line(&line_ending_str, &h.ss, &syn_highlighter);
            let spans: Vec<Span> = ranges
                .into_iter()
                .map(|(style, text)| {
                    let (r, g, b) = syntect_to_mocha(style.foreground);
                    (r, g, b, text.trim_end_matches('\n').to_string())
                })
                .collect();

            let l_hash = hash_str(line);
            new_entries.push(((p_hash, row), l_hash, spans.clone()));
            result.push(spans);

            let next_row = row + 1;
            if next_row % CHECKPOINT_INTERVAL == 0 {
                intermediate_checkpoints.push((next_row, state.clone()));
            }
        } else {
            break;
        }
    }

    // Save checkpoints
    if !intermediate_checkpoints.is_empty() {
        if let Ok(mut cp_map) = get_checkpoints().write() {
            let tree = cp_map.entry(p_hash).or_default();
            for (ck_row, ck_state) in intermediate_checkpoints {
                tree.insert(ck_row, ck_state);
            }
        }
    }

    // Populate bounded LRU cache
    if let Ok(mut cache) = get_syntax_cache().write() {
        for (k, l_hash, v) in new_entries {
            cache.insert(k, l_hash, v);
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
