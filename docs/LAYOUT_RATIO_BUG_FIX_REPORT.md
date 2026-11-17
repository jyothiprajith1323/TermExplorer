# TermExplorer 1.1.0 — Layout Ratio Architecture Fix & Settings Integration Report

**Date:** 2026-11-17  
**Component:** Ratatui UI Workstation Layout, Dynamic Settings Engine, and Configuration Subsystem  
**Status:** FULLY RESOLVED & VERIFIED  

---

## 1. Executive Summary

A critical user interface configuration defect in TermExplorer caused the workstation layout to render approximately **85% File Explorer** and **15% Preview / Inspector**, despite the Settings Studio displaying **25% Explorer / 75% Preview** and persisting "Settings saved successfully".

This investigation uncovered an architectural disconnect between persisted settings, runtime state, and the Ratatui layout generator. The root causes have been systematically addressed:
1. **Single Source of Truth:** `LayoutSettings` (`explorer_ratio` and `preview_ratio`) is now established as the canonical source of truth for workstation horizontal splits, strictly enforcing `explorer_ratio + preview_ratio == 1.0` (100%) and clamping the File Explorer width between 15% and 50% (Preview derived as 50% to 85%).
2. **Main Renderer Pipeline Reconnected:** The Ratatui rendering loop in `src/app.rs` now directly consumes `state.config.layout.preview_ratio` rather than stale or disconnected root fields.
3. **Dynamic Settings Studio & Shortcut Synchronization:** Adjusting either panel width in Settings Studio or using browse shortcuts (`[` and `]`) immediately updates both complementary proportions in lockstep, validates constraints, and atomically persists to disk with automatic `.bak` backups.
4. **Layout Generator Cleanup:** Removed hardcoded clamping bypasses and special-case hacks in `src/ui/layout.rs`, implementing exact proportional column allocation with minimum terminal column guarantees.
5. **Comprehensive Verification & Regression Suite:** Added a dedicated 10-test suite (`tests/layout_ratio_tests.rs`), captured 5 full terminal text screenshots across key ratios, passed all 38+ repository test suites, and verified all Linux release packages (`.deb`, `.rpm`, `.tar.gz`).

---

## 2. Root Cause Analysis

Tracing the entire configuration pipeline from disk deserialization to screen rendering revealed four distinct points of failure:

### A. Rendering Pipeline Disconnection in `src/app.rs`
- **Defect:** In `App::draw` (`src/app.rs`), the `LayoutConfig` passed to `crate::ui::layout::calculate_layout` initialized `preview_pct` using `self.state.config.preview_width`.
- **Impact:** `preview_width` was a legacy configuration field initialized or corrupted to 15 in user configuration files (`~/.config/termexplorer/config.toml`), completely ignoring the correct modern `[layout]` table where `explorer_ratio = 0.25` and `preview_ratio = 0.75` were stored. The UI thus rendered with a 15% preview pane and 85% explorer pane.

### B. Two Independently Mutable Ratios
- **Defect:** `explorer_ratio` and `preview_ratio` were maintained as separate mutable fields without automatic complementary derivation.
- **Impact:** Modifying one value could produce an invalid configuration where the sum did not equal 100%, or where one panel collapsed beyond minimum usable bounds.

### C. Settings Studio Mutation Desynchronization
- **Defect:** The Settings Studio adjustment handler in `src/app.rs` modified `state.config.layout` without updating `state.config.preview_width`.
- **Impact:** Even when users altered ratios in Settings Studio and saw "Settings saved successfully", the main view continued to draw using the old `preview_width` upon exiting Settings Studio.

### D. Hardcoded Logic & Hacks in `src/ui/layout.rs`
- **Defect:** `calculate_layout` contained brittle conditional checks (`cfg.preview_pct != 30 && cfg.preview_pct != 35`) and static minimum overrides that distorted custom user layout percentages.

---

## 3. Architecture & Implementation Fixes

### A. Single Source of Truth (`src/config/mod.rs`)
Added dedicated helper methods to `LayoutSettings` and `Config` that enforce the mathematical invariant:
```rust
explorer_percent + preview_percent == 100
15 <= explorer_percent <= 50
50 <= preview_percent <= 85
```

```rust
impl LayoutSettings {
    pub fn explorer_percent(&self) -> u16 {
        (self.explorer_ratio * 100.0).round() as u16
    }

    pub fn preview_percent(&self) -> u16 {
        (self.preview_ratio * 100.0).round() as u16
    }

    pub fn set_explorer_percent(&mut self, pct: u16) {
        let clamped_exp = pct.clamp(15, 50);
        self.explorer_ratio = clamped_exp as f32 / 100.0;
        self.preview_ratio = (100 - clamped_exp) as f32 / 100.0;
    }

    pub fn set_preview_percent(&mut self, pct: u16) {
        let clamped_exp = (100u16.saturating_sub(pct)).clamp(15, 50);
        self.explorer_ratio = clamped_exp as f32 / 100.0;
        self.preview_ratio = (100 - clamped_exp) as f32 / 100.0;
    }
}
```

In `Config::validate_and_clamp()`:
- NaN and out-of-range ratios are clamped into `0.15..=0.50`.
- Complementary preview ratio is derived: `100 - clamped_exp`.
- Legacy `preview_width` is kept synchronized with `layout.preview_percent()`.
- Terminal drawer height is independently clamped to `10%..=40%` (default `15%`).

### B. Backward-Compatible Migration (`src/config/migration.rs`)
Configurations from v1 schemas or configs lacking the modern `[layout]` table automatically inherit the user's `preview_width` into `layout.set_preview_percent(pw)` while upgrading the schema to version 2.

### C. UI Layout Engine (`src/ui/layout.rs`)
Cleaned up `calculate_layout()` to calculate exact proportional columns from the available content area width (`avail_w`):
```rust
let preview_pct = cfg.preview_pct.clamp(50, 85);
let mut prev_w = (avail_w * preview_pct) / 100;
let mut exp_w = avail_w.saturating_sub(prev_w);

if exp_w < 12 && avail_w > 12 {
    exp_w = 12;
    prev_w = avail_w.saturating_sub(12);
} else if prev_w < 20 && avail_w > 20 {
    prev_w = 20;
    exp_w = avail_w.saturating_sub(20);
}
```

### D. Settings Studio & Hotkey Dispatcher (`src/app.rs`)
1. **Dynamic Rendering:** The main workstation renderer reads directly from `state.config.layout.preview_ratio`:
   ```rust
   let preview_pct = (self.state.config.layout.preview_ratio * 100.0).round() as u16;
   let layout_cfg = LayoutConfig {
       preview_pct,
       terminal_pct: self.state.config.layout.terminal_height_percent,
       terminal_visible: self.state.terminal_visible,
       zen_mode: self.state.zen_mode,
   };
   ```
2. **Left/Right Adjustments:** Adjusting either `explorer_ratio` or `preview_ratio` in Settings Studio increments/decrements in 5% steps, automatically synchronizes both values, and refreshes the preview immediately.
3. **Browse Hotkeys:** `[` and `]` keybindings adjust the live workstation split and synchronize `state.config.layout` and `preview_width` in lockstep.
4. **Atomic Save:** On `F10` or `Esc`, the configuration is validated, persisted using temporary sibling write + atomic replace, and backed up to `config.toml.bak`.

---

## 4. Visual Evidence Table

The layout engine was verified across five standardized workstation ratios on a canonical 120-column by 40-row terminal. Text screenshots were captured and stored in the `screenshots/` directory:

| Layout Ratio | Explorer Width | Preview Width | Terminal Drawer | Screenshot Artifact | Status |
|:---|:---:|:---:|:---:|:---|:---:|
| **25% / 75%** (Default) | 30 cols | 90 cols | Hidden | [layout_ratio_25_75.txt](file:///home/prajith/TermExplorer/screenshots/layout_ratio_25_75.txt) | Verified |
| **35% / 65%** (Developer Profile) | 42 cols | 78 cols | Hidden | [layout_ratio_35_65.txt](file:///home/prajith/TermExplorer/screenshots/layout_ratio_35_65.txt) | Verified |
| **40% / 60%** (Custom) | 48 cols | 72 cols | Hidden | [layout_ratio_40_60.txt](file:///home/prajith/TermExplorer/screenshots/layout_ratio_40_60.txt) | Verified |
| **50% / 50%** (Maximum Explorer) | 60 cols | 60 cols | Hidden | [layout_ratio_50_50.txt](file:///home/prajith/TermExplorer/screenshots/layout_ratio_50_50.txt) | Verified |
| **15% / 85%** (Minimum Explorer) | 18 cols | 102 cols | Hidden | [layout_ratio_15_85.txt](file:///home/prajith/TermExplorer/screenshots/layout_ratio_15_85.txt) | Verified |

All layouts cleanly allocate exact terminal columns, preserve Catppuccin Mocha borders, and maintain full text readability.

---

## 5. Automated Regression Test Suite

A dedicated integration test suite was created in [layout_ratio_tests.rs](file:///home/prajith/TermExplorer/tests/layout_ratio_tests.rs), containing 10 automated test cases:

1. `test_layout_ratio_defaults_and_sum_to_100`: Confirms default 25/75 ratio sums to 100%.
2. `test_layout_ratio_clamping_limits`: Verifies boundaries (Explorer: 15%..50%, Preview: 50%..85%).
3. `test_layout_settings_helper_synchronization`: Confirms `set_explorer_percent` and `set_preview_percent` update each other complementary.
4. `test_settings_persistence_validation`: Verifies `SettingsPersistence::validate()` rejects corrupted or invalid ratios.
5. `test_workspace_rendering_five_ratios_on_120x40`: Renders all 5 ratios on a Ratatui buffer and verifies exact column dimensions.
6. `test_responsive_terminal_resizing_preserves_ratios`: Tests dynamic window resizing from 80 to 200 columns.
7. `test_terminal_drawer_independence_from_horizontal_ratios`: Asserts toggling the terminal drawer (15% height) does not alter horizontal panel width percentages.
8. `test_dual_pane_mode_preserves_preview_allocation`: Validates dual-pane file explorer mode preserves overall preview allocation.
9. `test_layout_persistence_across_sessions`: Verifies disk export, atomic write, and re-reading across application restarts.
10. `test_migration_legacy_preview_width_without_layout_table`: Ensures older configs seamlessly migrate without layout loss.

### Quality Gate Results
- `cargo fmt --all -- --check`: **PASS (Clean)**
- `cargo check --locked --all-targets --all-features`: **PASS (0 errors)**
- `cargo clippy --locked --all-targets --all-features -- -D warnings`: **PASS (0 warnings)**
- `cargo test --locked --all-targets --all-features`: **PASS (All 38+ suites passed)**

---

## 6. Linux Packaging & Distribution Verification

Release packages were rebuilt using `target/release/termexplorer` (v1.1.0) and validated against the comprehensive packaging acceptance suites:

```text
dist/
├── termexplorer_1.1.0_amd64.deb         (5.2 MiB, Debian package)
├── termexplorer-1.1.0-1.x86_64.rpm      (5.3 MiB, RPM package)
├── termexplorer-v1.1.0-linux-x86_64.tar.gz (6.4 MiB, Portable tarball)
├── SHA256SUMS                           (Verified cryptographic hashes)
└── MANIFEST.txt                         (Distribution manifest)
```

- `bash tests/packaging_tests.sh`: **56/56 PASS (100% Passed)**
- `bash scripts/test_package_linux.sh 1.1.0 dist`: **PASS (Debian, RPM, Tarball, and Checksums verified)**

---

## 7. Atlas MCP Server Integration

The Atlas MCP server ([github.com/bloodfeast/atlas](https://github.com/bloodfeast/atlas)) was installed and integrated:
- **Repository Location:** `/home/prajith/atlas`
- **Binary Executable:** `/home/prajith/atlas/bin/atlas-x86_64-unknown-linux-gnu` (linked to `~/.local/bin/atlas`)
- **Version:** Atlas 0.7.0 (`atlas doctor` verified)
- **MCP Configuration:** Registered in `/home/prajith/.gemini/config/mcp_config.json` with stdio transport.
- **Tool Schemas:** 14 tools (`lookup`, `search`, `section`, `grep`, `symbol`, `recall`, `remember`, `journal`, `handoff`, `note`, `edit`, `discuss`, `miss`, `hook`) registered in `/home/prajith/.gemini/antigravity-cli/mcp/atlas/`.
- **Custom Skills:** `atlas` and `orchestrate` skills copied into `/home/prajith/.gemini/config/skills/`.

---

## 8. Conclusion

The layout ratio disconnect is completely resolved. The application now launches with the intended 25% File Explorer / 75% Preview / 15% Terminal Drawer layout by default, respects all user customization from Settings Studio and keyboard shortcuts, and persists settings reliably across sessions.
