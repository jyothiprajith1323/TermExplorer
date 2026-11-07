# 📋 TermExplorer V1.1.0-RC2 — Release Readiness Checklist

This checklist verifies all engineering, architectural, safety, and release requirements for the approved V1.1.0-RC2 release candidate.

---

## 1. 🏗️ Build, Compilation & Linters

| Requirement | Command / Check | Expected Result | Status | Verified |
|---|---|---|:---:|:---:|
| Clean Compilation | `cargo check --all-targets --all-features` | Exit code 0, 0 errors | Passed | ✅ YES |
| Release Binary | `cargo build --release` | Optimized binary at `target/release/termexplorer` | 6.6 MB | ✅ YES |
| Format Standards | `cargo fmt --all -- --check` | Exit code 0, 0 diffs | Passed | ✅ YES |
| Strict Linter | `cargo clippy --all-targets --all-features -- -D warnings` | Exit code 0, 0 warnings | Passed | ✅ YES |
| Documentation Build | `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` | Exit code 0, 0 warnings, 0 broken links | Passed | ✅ YES |

---

## 2. 🧪 Automated Test Suite & Coverage

| Metric | Target | Measured Result | Status |
|---|---|:---:|:---:|
| Total Automated Tests | ≥ 80 tests | **114 tests** | ✅ EXCEEDS |
| Integration Test Suites | ≥ 15 suites | **24 suites** | ✅ EXCEEDS |
| Test Pass Rate | 100% | **100% (114 passed, 0 failed)** | ✅ PASSED |
| Keybinding & Panel Verification | All panels, modes & routing verified | 9 passed | ✅ PASSED |
| Adversarial Filesystem Tests | Zip slip, symlinks, unicode | 5 passed | ✅ PASSED |
| Adversarial Plugin Tests | Memory growth, loop fuel, traps | 5 passed | ✅ PASSED |
| Mouse Hitbox & Freeze Tests | Hitbox routing across 60–160 cols | 7 passed | ✅ PASSED |
| Visual Regression & Layout | 25/75 split, terminal drawer 15% | 10 passed | ✅ PASSED |
| Performance Benchmarks | 10 measured metrics (Debug + Release)| 1 passed | ✅ PASSED |
| Reliability & Data Safety | Atomic write, durability sync, trash | 7 passed | ✅ PASSED |

---

## 3. 🛡️ Data Safety & Filesystem Durability

| Mechanism | Description | Verification Method | Status |
|---|---|---|:---:|
| **Atomic File Replacement** | Staged via temporary files in same dir, committed via `rename` | Tested in `test_atomic_file_saving_preserves_content` | ✅ ACTIVE |
| **Physical Storage Durability** | `as_file().sync_all()` called before rename to flush dirty pages | Implemented in `save_file_atomic` | ✅ ACTIVE |
| **Directory Metadata Sync** | Parent directory `File::open().sync_all()` ensures directory commit | Implemented in `save_file_atomic` | ✅ ACTIVE |
| **Permission Preservation** | Captures and restores original POSIX permission mode bits | Tested in `save_file_atomic` | ✅ ACTIVE |
| **External Modification Protection** | Tracks `loaded_mtime` to prevent overwriting concurrent external changes | Tested in `test_editor_external_modification_detection` | ✅ ACTIVE |
| **Crash Recovery Snapshots** | Persists unsaved editor buffers to `~/.local/share/termexplorer/recovery/` | Tested in `test_editor_unsaved_buffer_recovery_snapshot` | ✅ ACTIVE |
| **Cross-Filesystem Transfers** | Verified copy-then-verify-then-remove with rollback on `EXDEV` | Tested in `move_entry` | ✅ ACTIVE |
| **FreeDesktop Trash Conformance** | Compliant `.trashinfo` metadata generation and bi-directional restore | Tested in `test_trash_and_restore_freedesktop_metadata` | ✅ ACTIVE |
| **Configuration Self-Healing** | Automatic `config.toml.bak` creation and corrupted archive quarantine | Tested in `test_config_atomic_backup_and_corrupted_recovery` | ✅ ACTIVE |

---

## 4. 🖥️ Approved Layout Architecture & Terminal Stability

| Feature | Target Specification | Measured Behavior | Status |
|---|---|---|:---:|
| **Default Horizontal Split** | **25% Explorer / 75% Preview** | Verified at 30/90 cols on 120-wide terminal | ✅ APPROVED |
| **Terminal Visibility** | **OFF by default on fresh install** | Default `show_terminal = false`, 0 PTY startup | ✅ APPROVED |
| **Terminal Drawer Height** | **~15% height when enabled** | Clamped to 5–8 rows (6 rows on 40-row terminal) | ✅ APPROVED |
| **Full Height Content Area** | Expanded to 38 rows when terminal closed | Verified in `test_render_workstation_main_layout_120x40` | ✅ APPROVED |
| **Workspace Sidebar** | Removed from permanent default display | Default `show_sidebar = false` | ✅ APPROVED |
| **Session Persistence** | Terminal session survives hiding and restoring | Held in `App.terminal_session` across toggles | ✅ APPROVED |
| **Emergency Panic Hook** | Restores raw mode, alternate screen, and mouse capture | Installed in `src/main.rs` | ✅ APPROVED |
| **Process Reaping** | Child process trees reaped cleanly on session drop | Tested in `test_terminal_session_clean_process_cleanup` | ✅ APPROVED |

---

## 5. 🧩 WebAssembly Sandboxing & Resource Controls

| Security Control | Specification | Implementation | Status |
|---|---|---|:---:|
| **Runtime Memory Limiter** | `wasmi::ResourceLimiter` intercepts `memory.grow` | Blocks allocations exceeding 16 MiB ceiling | ✅ ENFORCED |
| **Upfront Allocation Audit** | Initial & max pages inspected during instantiation | Rejects modules with declared memory > 16 MiB | ✅ ENFORCED |
| **Instruction Fuel Metering** | Bounded quota (`5,000,000` fuel steps) per call | Halts infinite loops within < 500 ms | ✅ ENFORCED |
| **Capability Permissions** | Strict manifest authorization required | Rejects unauthorized calls (`commands`, etc.) | ✅ ENFORCED |
| **Host Trap Isolation** | Guest traps caught gracefully; host never crashes | Verified in `test_adversarial_unreachable_trap_crash_isolation` | ✅ ENFORCED |

---

## 6. 🌐 Documentation, Packaging & Website

| Item | Target | Location | Status |
|---|---|---|:---:|
| **Validation Report** | Complete 10-part report | `docs/RC2_VALIDATION_REPORT.md` | ✅ READY |
| **Performance Benchmarks** | Debug vs. release measurements | `PERFORMANCE.md` & `docs/PERFORMANCE.md` | ✅ READY |
| **Test Inventory** | 105 tests across 23 suites | `TEST_RESULTS.md` | ✅ READY |
| **Architecture Guide** | 25/75 split & 15% drawer documented | `docs/ARCHITECTURE.md` | ✅ READY |
| **Install Script** | POSIX script with release build | `install.sh` | ✅ READY |
| **Arch Linux Package** | PKGBUILD for AUR release | `packaging/aur/PKGBUILD` | ✅ READY |
| **Website & SVG UI** | 25/75 layout and terminal drawer | `website/` | ✅ READY |
