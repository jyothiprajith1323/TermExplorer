//! Comprehensive Performance Benchmarks for TermExplorer.
//! Measures and outputs actual, reproducible metrics across:
//! - Startup initialization latency
//! - Memory footprint (RSS)
//! - Directory loading & sorting
//! - Syntax highlighting & preview generation
//! - Editor keystroke responsiveness
//! - Autocomplete query latency
//! - Command palette fuzzy ranking
//! - Theme switching & token resolution
//! - WASM plugin engine initialization

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tempfile::tempdir;
use termexplorer::config::Config;
use termexplorer::editor::buffer::Buffer;
use termexplorer::filesystem::browser::{list_dir, SortOrder};
use termexplorer::plugins::{PluginInstance, PluginManifest, PluginPermission};
use termexplorer::preview::detector::detect_file_format;
use termexplorer::state::{AppState, FocusedPanel};
use termexplorer::theme::Theme;

#[test]
fn test_reproducible_performance_benchmarks() {
    println!("\n══════════════════════════════════════════════════════════════════════════════");
    println!("                   TERMEXPLORER PERFORMANCE BENCHMARKS");
    println!("══════════════════════════════════════════════════════════════════════════════");

    // 1. Startup Time
    let t0 = Instant::now();
    let config = Config::default();
    let state = AppState::new(config);
    let startup_elapsed = t0.elapsed();
    println!(
        " [1] Startup Latency (AppState init) : {:>8.2} ms",
        startup_elapsed.as_secs_f64() * 1000.0
    );

    // 2. Memory Usage (RSS from /proc/self/statm)
    let pages = fs_read_statm_rss().unwrap_or(0);
    let page_size_kb = 4; // standard 4KB Linux page
    let rss_mb = (pages * page_size_kb) as f64 / 1024.0;
    println!(" [2] Resident Memory (Current RSS)   : {:>8.2} MB", rss_mb);

    // 3. Directory Loading (1,000 files listing & sorting)
    let tmp = tempdir().unwrap();
    for i in 0..1000 {
        let p = tmp.path().join(format!("file_{:04}.txt", i));
        let _ = File::create(&p).unwrap();
    }
    let t_dir = Instant::now();
    let entries = list_dir(tmp.path(), false, SortOrder::Name).unwrap();
    let dir_elapsed = t_dir.elapsed();
    assert_eq!(entries.len(), 1000);
    println!(
        " [3] Directory Traversal (1,000 fs) : {:>8.2} ms",
        dir_elapsed.as_secs_f64() * 1000.0
    );

    // 4. Preview Generation (Syntax highlighting 1,000 lines of Rust)
    let sample_code = "fn calculate_hash<T: std::hash::Hash>(t: &T) -> u64 {\n    let mut s = std::collections::hash_map::DefaultHasher::new();\n    t.hash(&mut s);\n    std::hash::Hasher::finish(&s)\n}\n".repeat(200);
    let preview_file = tmp.path().join("syntax_sample.rs");
    let mut f = File::create(&preview_file).unwrap();
    f.write_all(sample_code.as_bytes()).unwrap();

    let t_syntax = Instant::now();
    let _ft = detect_file_format(&preview_file);
    let ps = syntect::parsing::SyntaxSet::load_defaults_newlines();
    let syntax = ps.find_syntax_by_extension("rs").unwrap();
    let theme_set = syntect::highlighting::ThemeSet::load_defaults();
    let mut h = syntect::easy::HighlightLines::new(syntax, &theme_set.themes["base16-ocean.dark"]);
    for line in sample_code.lines().take(500) {
        let _ = h.highlight_line(line, &ps);
    }
    let syntax_elapsed = t_syntax.elapsed();
    println!(
        " [4] Syntax Highlighting (500 lines): {:>8.2} ms",
        syntax_elapsed.as_secs_f64() * 1000.0
    );

    // 5. Editor Responsiveness (10,000 keystroke inserts with undo history)
    let mut buf = Buffer::new();
    let t_edit = Instant::now();
    for _ in 0..1000 {
        buf.insert_char('a');
    }
    let edit_elapsed = t_edit.elapsed();
    let per_keystroke_us = (edit_elapsed.as_secs_f64() * 1_000_000.0) / 1000.0;
    println!(
        " [5] Editor Keystroke Latency       : {:>8.2} µs (0.00{:.1} ms)",
        per_keystroke_us,
        per_keystroke_us / 1000.0
    );

    // 6. Autocomplete Latency (Layered Engine query)
    let engine = termexplorer::completion::CompletionEngine::new();
    engine.index_workspace(Path::new(env!("CARGO_MANIFEST_DIR")));
    let t_comp = Instant::now();
    let comps = engine.complete_editor(
        &["fn calculate() {}".to_string()],
        "calc",
        Some(Path::new("test.rs")),
        10,
    );
    let comp_elapsed = t_comp.elapsed();
    assert!(!comps.is_empty());
    println!(
        " [6] Layered Autocomplete Query     : {:>8.2} µs",
        comp_elapsed.as_secs_f64() * 1_000_000.0
    );

    // 7. Command Palette Ranking (Fuzzy matching across actions)
    let t_pal = Instant::now();
    let pal_comps = engine.complete_palette("sett", &[], &[], FocusedPanel::Explorer);
    let pal_elapsed = t_pal.elapsed();
    assert!(!pal_comps.is_empty());
    println!(
        " [7] Command Palette Ranking        : {:>8.2} µs",
        pal_elapsed.as_secs_f64() * 1_000_000.0
    );

    // 8. Theme Switching (Iterate all 12 themes & re-resolve palette)
    let theme_names = [
        "Catppuccin Mocha",
        "Catppuccin Macchiato",
        "Catppuccin Frappe",
        "Catppuccin Latte",
        "Dracula",
        "Nord",
        "Gruvbox",
        "Tokyo Night",
        "One Dark",
        "Solarized Dark",
        "Solarized Light",
        "Monokai",
    ];
    let t_theme = Instant::now();
    for name in &theme_names {
        let _ = Theme::from_name(name, true);
    }
    let theme_elapsed = t_theme.elapsed();
    println!(
        " [8] Theme Resolution (12 themes)   : {:>8.2} µs ({:.2} µs/theme)",
        theme_elapsed.as_secs_f64() * 1_000_000.0,
        (theme_elapsed.as_secs_f64() * 1_000_000.0) / 12.0
    );

    // 9. WASM Plugin Engine Init & Verification
    let wasm_file = PathBuf::from("plugins/file-inspector/plugin.wasm");
    if wasm_file.exists() {
        let wasm_bytes = std::fs::read(&wasm_file).unwrap();
        let manifest = PluginManifest {
            id: "bench-plugin".to_string(),
            name: "Bench Plugin".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            description: None,
            entrypoint: "plugin.wasm".to_string(),
            permissions: [PluginPermission::Commands].into_iter().collect(),
            sha256: None,
        };
        let t_wasm = Instant::now();
        let mut inst = PluginInstance::new(manifest, &wasm_bytes, ".").unwrap();
        let init_time = t_wasm.elapsed();
        let _ = inst.trigger_command("test");
        println!(
            " [9] WASM Plugin Init & Sandbox     : {:>8.2} ms",
            init_time.as_secs_f64() * 1000.0
        );
    }

    // 10. Frame Redraw Latency
    let backend = ratatui::backend::TestBackend::new(120, 40);
    let mut term = ratatui::Terminal::new(backend).unwrap();
    let theme = Theme::catppuccin_mocha();
    let mut hit_reg = termexplorer::ui::hit_test::HitRegistry::new();
    let layout_cfg = termexplorer::ui::layout::LayoutConfig {
        show_sidebar: false,
        show_preview: true,
        show_bottom: false,
        dual_pane: false,
        sidebar_pct: 20,
        preview_pct: 75,
        bottom_height: 6,
        preview_maximized: false,
        terminal_maximized: false,
    };
    let areas = termexplorer::ui::layout::compute_layout(term.get_frame().area(), &layout_cfg);

    let t_frame = Instant::now();
    for _ in 0..100 {
        let _ = term.draw(|f| {
            termexplorer::ui::header::render_header(
                f,
                areas.header,
                &state,
                None,
                &theme,
                &mut hit_reg,
            );
            termexplorer::ui::file_explorer::render_file_explorer(
                f,
                areas.explorer,
                &state,
                &theme,
                &mut hit_reg,
            );
            if let Some(pv) = areas.preview {
                termexplorer::ui::preview::render_preview(f, pv, &state, &theme, &mut hit_reg);
            }
            termexplorer::ui::status_bar::render_status_bar(f, areas.status_bar, &state, &theme);
        });
    }
    let frame_elapsed = t_frame.elapsed();
    let per_frame_ms = (frame_elapsed.as_secs_f64() * 1000.0) / 100.0;
    println!(
        "[10] TUI Frame Redraw Latency       : {:>8.2} ms (~{:.0} FPS capability)",
        per_frame_ms,
        1000.0 / per_frame_ms
    );
    println!("══════════════════════════════════════════════════════════════════════════════\n");
}

fn fs_read_statm_rss() -> Option<u64> {
    let statm = std::fs::read_to_string("/proc/self/statm").ok()?;
    let mut parts = statm.split_whitespace();
    let _vsize = parts.next()?;
    let rss = parts.next()?.parse::<u64>().ok()?;
    Some(rss)
}
