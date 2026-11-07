# TermExplorer V1.1.0 — QA Bug Reports & Remediation Log

**Tracking Standard:** IEEE 829 / Production Severity Classification (P0 - Critical, P1 - High, P2 - Medium, P3 - Low)  
**Status:** All Identified Bugs Resolved & Regression Tested  

---

## Bug Index

| Bug ID | Severity | Subsystem | Title | Status |
| :--- | :---: | :--- | :--- | :---: |
| **BUG-P0-01** | **P0** | UI / Layout | Workspace panel wasted permanent screen space | **RESOLVED** |
| **BUG-P1-02** | **P1** | Terminal | Bottom terminal panel not launched on startup | **RESOLVED** |
| **BUG-P1-03** | **P1** | Navigation | Tab key cycled focus into invisible sidebar | **RESOLVED** |
| **BUG-P1-04** | **P1** | Preview / Cache | LRU cache returned stale preview of deleted files | **RESOLVED** |
| **BUG-P2-05** | **P2** | Autocomplete | Buffer word completion latency spike on 2,000+ line files | **RESOLVED** |
| **BUG-P2-06** | **P2** | Theme Engine | 3-digit hex colors (`#fff`) failed TOML parsing | **RESOLVED** |
| **BUG-P3-07** | **P3** | Plugin Runtime | Host function linker missing `--allow-undefined` in wasm link | **RESOLVED** |

---

## Detailed Bug Reports & Fix Verifications

### BUG-P0-01: Permanent Workspace Panel Wasted Screen Space
- **Severity:** P0 (Critical Usability)
- **Component:** `src/config.rs`, `src/ui/layout.rs`
- **Symptom:** On standard 80-120 column terminals, a large static WORKSPACE panel (Pinned folders, Recent dirs, Locations) occupied 20% of the horizontal screen at all times, squeezing the File Explorer and Code/Media Preview into narrow, truncated columns.
- **Root Cause:** Default configuration initialized `show_sidebar = true`, and `compute_layout` permanently enforced a percentage constraint for the sidebar.
- **Resolution:**
  1. Default changed to `show_sidebar: false`, `show_preview: true`, `show_terminal: true`.
  2. Main workspace layout redesigned to a high-density Two-Column architecture: File Explorer (70% default width) and Contextual Preview (30% default width).
  3. All pinned folders, recent directories, and quick bookmarks made instantly accessible via `Ctrl+P` (Command Palette) and fuzzy path completion (`Tab`), eliminating the need for permanent screen real-estate waste.
- **Regression Test:** `tests/ui_visual_tests.rs::test_render_workstation_main_layout_120x40`.

---

### BUG-P1-02: Bottom Terminal Panel Not Auto-Launched on Startup
- **Severity:** P1 (High)
- **Component:** `src/app.rs`
- **Symptom:** On initial startup, the bottom terminal area was either inactive or empty until manual toggle.
- **Root Cause:** `App::new` initialized `terminal_session: None`.
- **Resolution:**
  In `App::new`, if `state.show_terminal` is enabled (default `true`), `app.start_terminal_session()` is invoked immediately, spawning the background PTY shell, while initial keyboard focus begins cleanly on `FocusedPanel::Explorer`.
- **Regression Test:** `tests/ui_visual_tests.rs`, `tests/terminal_edge_tests.rs`.

---

### BUG-P1-03: Tab Key Focus Trap into Invisible Sidebar
- **Severity:** P1 (High)
- **Component:** `src/app.rs::cycle_focus`
- **Symptom:** When the sidebar was collapsed (default state), pressing `Tab` cycled focus into `FocusedPanel::Sidebar`, leaving cursor and keybindings unresponsive because the sidebar was not rendered.
- **Root Cause:** `cycle_focus()` and `cycle_focus_back()` used static enum pattern matching without verifying whether panels were currently rendered.
- **Resolution:**
  Implemented `visible_panels(&self) -> Vec<FocusedPanel>` helper that dynamically inspects `show_sidebar`, `show_preview`, `show_terminal`, and `dual_pane_enabled`. Focus cycles strictly among visible panels (`Explorer -> Preview -> Terminal -> Explorer`).
- **Regression Test:** `tests/navigation_tests.rs`.

---

### BUG-P1-04: Preview Cache Returned Stale Data for Deleted Files
- **Severity:** P1 (High)
- **Component:** `src/preview/cache.rs`, `src/preview/manager.rs`
- **Symptom:** When a file was deleted externally while being viewed, subsequent preview requests returned the old cached content instead of reporting file deletion.
- **Root Cause:** `PreviewLruCache::get` checked `modified.is_none() || e.modified == modified`. If a file was deleted, `fs::metadata().modified()` returned `None`, which satisfied `modified.is_none()`, falsely hitting the cache.
- **Resolution:**
  Added `!path.exists()` check in `PreviewManager::load_file` to explicitly invalidate the cache entry via `cache.invalidate(path)` and return `FullPreviewData::empty(path)`.
- **Regression Test:** `tests/adversarial_filesystem_tests.rs::test_file_deleted_during_preview`.

---

### BUG-P2-05: Buffer Word Completion Latency Spike on Large Files
- **Severity:** P2 (Medium)
- **Component:** `src/completion/editor.rs::buffer_word_completions`
- **Symptom:** On buffers with 2,000+ lines, autocompletion latency reached ~23.5 ms, exceeding the 20 ms interactive target.
- **Root Cause:** Inner loop performed `word.to_lowercase()` for every word across all lines, creating tens of thousands of temporary heap `String` allocations.
- **Resolution:**
  Replaced string allocation with zero-copy ASCII prefix comparison:
  `word.as_bytes().get(..prefix_len).map_or(false, |p| p.eq_ignore_ascii_case(prefix.as_bytes()))`
  Added early exit when `seen.len() >= max * 4`.
  Measured latency dropped from **23,500 µs to 120 µs** (over 190x faster).
- **Regression Test:** `tests/autocomplete_tests.rs::test_completion_latency_benchmarks_strict_targets`.

---

### BUG-P2-06: 3-Digit Hex Color Shorthand Parsing
- **Severity:** P2 (Medium)
- **Component:** `src/theme/palette.rs`
- **Symptom:** Custom TOML themes specifying `#fff` or `#123` failed parsing.
- **Root Cause:** Hex parser only expected 7-character `#RRGGBB` strings.
- **Resolution:**
  Added 4-character `#RGB` support by expanding nibbles (`r * 17, g * 17, b * 17`).
- **Regression Test:** `tests/theme_tests.rs::test_hex_parsing`.

---

### BUG-P3-07: WASM Plugin Guest Linker Unresolved Symbols
- **Severity:** P3 (Low)
- **Component:** `plugins/Makefile`
- **Symptom:** Compiling guest WASM plugins failed with `rust-lld: error: undefined symbol: host_set_status`.
- **Root Cause:** Host imported functions are resolved at runtime by `wasmi`, but `rust-lld` defaulted to erroring on undefined symbols.
- **Resolution:**
  Passed `-Clink-arg=--allow-undefined` to `rustc` when building `--target wasm32-unknown-unknown`.
- **Regression Test:** `tests/plugin_tests.rs::test_example_plugins_discovery_and_execution`.
