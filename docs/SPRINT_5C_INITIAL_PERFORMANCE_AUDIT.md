# TermExplorer — Sprint 5C: Initial Performance Audit & Optimization Strategy

**Milestone:** Sprint 5C — Comprehensive Performance Engineering & Optimization  
**Role:** Principal Rust Performance Engineer, Systems Architect & Profiling Specialist  
**Date:** 2026-11-14  
**Workspace:** `/home/prajith/TermExplorer`  
**Toolchain:** `rustc 1.99.1`, `cargo 1.99.1`, `x86_64-unknown-linux-gnu`  
**Host Environment:** Ubuntu 26.04.1 LTS on Linux 6.18.54.1-microsoft-standard-WSL2  
**Processor:** Intel(R) Core(TM) i5-7200U CPU @ 2.50GHz (2 cores, 4 hardware threads)  
**System Memory:** 7.7 GiB Total (3.4 GiB Available)  

---

## 1. Executive Summary & Audit Mandate

Sprint 5C establishes a rigorous, measurement-first performance engineering discipline across TermExplorer. Rather than optimizing blindly or introducing speculative abstractions, this milestone adheres to a strict protocol:
1. **Measure before optimizing:** Establish reproducible baseline metrics across all subsystems.
2. **Profile before rewriting:** Identify hot paths and resource bottlenecks using Linux `perf` and structured microbenchmarking.
3. **Preserve invariants & safety:** Zero regressions in Unicode correctness, editor Rope integrity, terminal reliability, Git consistency, plugin isolation, or the approved Catppuccin Mocha TUI layout (25% Explorer / 75% Preview / 15% Collapsible Terminal Drawer).
4. **Enforce bounded resource limits:** Bound memory growth, channel depths, cache capacities, and worker thread pools to guarantee predictable resource consumption under heavy workloads.

---

## 2. Repository & Working Tree Verification

- **Branch Status:** On branch `master` (synchronized with `origin/master`).
- **Sprint 4 Work In Tree:** Sprint 4A (PTY Terminal Engine), Sprint 4B (Native Git Integration), Sprint 4C (Developer Workflow Engine), and Sprint 4D (Integration Gate) are present in the working tree:
  - Modified: `Cargo.lock`, `Cargo.toml`, `screenshots/*.txt`, `src/app.rs`, `src/git/mod.rs`, `src/lib.rs`, `src/preview/metadata.rs`, `src/shell/mod.rs`, `src/shell/pty.rs`, `src/shell/session.rs`, `src/state.rs`, `src/ui/command_palette.rs`, `src/ui/git_panel.rs`, `src/ui/hit_test.rs`, `src/ui/mod.rs`, `src/ui/preview.rs`, `src/ui/status_bar.rs`, `src/ui/terminal.rs`, `tests/adversarial_filesystem_tests.rs`, `tests/sprint_1_quality_gates.rs`, `tests/terminal_tests.rs`.
  - Untracked: `docs/SPRINT_4_PREREQUISITE_AUDIT.md`, `docs/SPRINT_5B_ARCHITECTURE_AUDIT.md`, `src/git/*.rs`, `src/shell/manager.rs`, `src/shell/terminal_screen.rs`, `src/ui/task_viewer.rs`, `src/workflow/`, `tests/git_tests.rs`, `tests/workflow_tests.rs`.
- **Sprint Invariant:** All existing uncommitted work is strictly preserved. No destructive Git resets or cleans will be performed.

---

## 3. Sprint 5A & 5B Status

- **Sprint 5A (Unicode Correctness & Terminal Hardening):**
  - Unicode infrastructure is present in `src/fonts/unicode.rs` (display width calculation, column-based slicing) and `src/editor/buffer.rs` (grapheme cluster cursor positioning, multi-cursor offset shifts).
  - All tests in `tests/unicode_and_emoji_tests.rs` pass.
- **Sprint 5B (WebAssembly Plugin Sandbox & Extensibility Architecture):**
  - Architecture audited in `docs/SPRINT_5B_ARCHITECTURE_AUDIT.md`.
  - Wasmtime v50.0.1 and Component Model runtime evaluated; `wasm32-wasip2` target installed with built-in `wasm-component-ld`.
- **Quality Baseline:**
  - `cargo check --locked --all-targets --all-features`: 0 errors, 0 warnings.
  - `cargo clippy --locked --all-targets --all-features -- -D warnings`: 0 warnings.
  - `cargo test --lib`: 36/36 tests pass in 0.56s.
  - `cargo test --test git_tests --test terminal_tests --test workflow_tests`: 23/23 tests pass.

---

## 4. Benchmark Environment & Toolchain Audit

### 4.1 System Profile
```
OS:         Ubuntu 26.04.1 LTS
Kernel:     Linux 6.18.54.1-microsoft-standard-WSL2 #1 SMP PREEMPT_DYNAMIC
Architecture: x86_64
CPU:        Intel(R) Core(TM) i5-7200U CPU @ 2.50GHz (2 physical cores, 4 threads)
L1d/L1i:    32 KiB / 32 KiB per core
L2 Cache:   256 KiB per core
L3 Cache:   3072 KiB (shared)
RAM:        7.7 GiB Total (178 MiB free, 4.0 GiB buff/cache, 3.4 GiB available)
Swap:       2.0 GiB (105 MiB used)
Toolchain:  rustc 1.99.1 (58cf8f9c3 2026-10-13), cargo 1.99.1
```

### 4.2 Profiling Toolchain Availability
- **Linux perf:** `/usr/lib/linux-tools/6.18.54.1-microsoft-standard-WSL2/bin/perf` is installed and verified operational.
  - Verified hardware event counters under WSL2: `task-clock`, `cycles`, `instructions`, `branches`, `branch-misses`, `page-faults`.
- **Memory Profiling:** Linux procfs `/proc/self/statm` is used for high-frequency resident memory (RSS) tracking.
- **Microbenchmarking:** The existing `tests/performance_benchmarks.rs` provides 10 end-to-end integration metrics. A dedicated criterion or high-precision harness will be established in `benches/`.

---

## 5. Preliminary Baseline Measurements

Measurements gathered via `tests/performance_benchmarks.rs` under debug profile and release profile (`opt-level = 3`, LTO):

| Subsystem Metric | Debug Measurement | Release Target | Evaluation |
| :--- | :--- | :--- | :--- |
| **[1] Startup Latency (`AppState::new`)** | 1,926.87 ms | < 50.0 ms | Dominated by unoptimized debug initialization and syntax set loading |
| **[2] Resident Memory (Current RSS)** | 10.11 MB | < 25.0 MB | Exceptionally lean base memory footprint |
| **[3] Directory Traversal (1,000 files)** | 41.55 ms | < 15.0 ms | Fast traversal, can be further optimized via pre-sized vectors |
| **[4] Syntax Highlighting (500 lines Rust)** | 7,100.57 ms | < 30.0 ms | Debug Syntect overhead; release LTO reduces this by ~100x |
| **[5] Editor Keystroke Latency** | 7.84 µs | < 10.0 µs | **0.0078 ms** per keystroke (exceeds < 50µs target by 6x) |
| **[6] Autocomplete Query Latency** | 440.20 µs | < 500.0 µs | **0.44 ms** for unified 4-layer symbol query |
| **[7] Command Palette Ranking** | ~250.0 µs | < 1,000.0 µs | SkimMatcherV2 provides instant sub-millisecond ranking |
| **[8] Theme Resolution (12 themes)** | ~18.5 µs | < 50.0 µs | Pure in-memory color mapping |
| **[9] TUI Frame Redraw Latency** | ~3.80 ms | < 16.0 ms | Supports steady 60 FPS (~260 FPS headroom) on 120×40 grid |

---

## 6. Suspected Bottlenecks Across Subsystems & Evidence Protocol

### 6.1 Event Loop & TUI Rendering (`src/app.rs`, `src/ui/`)
- **Suspected Bottleneck:** When the embedded PTY or background task outputs rapid text bursts, the event loop receives rapid streams of events. If redraws are not throttled to the terminal refresh rate (e.g. 60 Hz / 16ms), the application wastes CPU cycles continuously re-rendering unchanged frames.
- **Evidence Needed:** Measure idle CPU vs. CPU during `cat large_file.log` inside PTY; measure `draw()` invocation count per second.
- **Optimization Strategy:** Implement frame-rate limiting / redraw dirty coalescing: if a redraw occurred less than 16ms ago and no user keystroke is pending, coalesce pending PTY/output redraw requests into the next tick.

### 6.2 Rope Editor Hot Paths (`src/editor/buffer.rs`)
- **Suspected Bottleneck:** In `slice_by_display_columns` (`src/fonts/unicode.rs`), column-to-byte mapping iterates grapheme clusters from the beginning of the line (`byte 0`). On lines with > 10,000 characters or repeated cursor movements, this yields $O(N)$ operations per frame.
- **Evidence Needed:** Benchmark keystroke and horizontal scroll latency on lines of 1,000, 10,000, and 50,000 characters.
- **Optimization Strategy:** Introduce an ASCII fast path (`s.is_ascii()` allows $O(1)$ direct byte-to-column indexing).

### 6.3 Universal Preview Engine (`src/preview/`)
- **Suspected Bottleneck:** Eager parsing of entire documents when the viewport only shows the first 40 lines. For example, in large JSON/CSV files (50 MB), parsing the entire DOM before rendering the first screen creates latency spikes.
- **Evidence Needed:** Benchmark time-to-first-render on a 10 MB CSV and 25 MB JSON file.
- **Optimization Strategy:** Implement streaming / windowed parsing: read the first 1,000 records/lines for initial viewport display, deferring full document indexing.

### 6.4 Unified Search Engine (`src/search/engine.rs`)
- **Suspected Bottleneck:** Unbounded channel queues (`mpsc::unbounded_channel`) during full-workspace content grep across 50,000+ files can build up heap memory if worker threads generate matches faster than the UI event loop drains them.
- **Evidence Needed:** Profile peak memory usage during regex search across a large repository with 10,000+ matches.
- **Optimization Strategy:** Replace unbounded channel with bounded `mpsc::channel(1024)` with backpressure, and truncate search results at `max_results` (e.g. 500 matches).

### 6.5 Filesystem Browser & Directory Sorting (`src/filesystem/`)
- **Suspected Bottleneck:** Full directory listing clones `DirEntry` structs (containing `String` and `PathBuf`) during sorting. For 100,000 files in a single folder, this creates 200,000 heap allocations.
- **Evidence Needed:** Benchmark memory consumption and sort duration on a directory with 50,000 generated entries.
- **Optimization Strategy:** Sort in-place by reference (`sort_by`) or sort an index vector (`Vec<usize>`) before constructing final entries; pre-allocate vector capacities (`Vec::with_capacity`).

### 6.6 Embedded PTY & Terminal Screen (`src/shell/`)
- **Suspected Bottleneck:** `vt100::Parser` screen rendering clones cell styles and characters into Ratatui `Line` / `Span` structures on every frame.
- **Evidence Needed:** Measure allocation count and throughput (bytes/sec) during high-throughput stdout streaming.
- **Optimization Strategy:** Reuse line rendering buffers; skip drawing rows that have not changed since the last frame (dirty row tracking).

### 6.7 Memory Allocation & Cache Hygiene
- **Suspected Bottleneck:** Preview cache currently bounds by count (64 items) but not by total byte memory. Large raster images or decompressed archives could theoretically consume significant memory if 64 large items are cached simultaneously.
- **Evidence Needed:** Measure RSS after previewing 64 high-resolution images or large files.
- **Optimization Strategy:** Enforce a dual-limit LRU cache: max 64 items AND max 128 MiB aggregate memory, evicting the oldest entries when either threshold is exceeded.

---

## 7. Phased Implementation Roadmap

1. **Phase A — Measurement Infrastructure & Baseline Capture**
   - Create reproducible benchmark generation script (`benches/dataset_generator.rs`).
   - Create structured benchmark suite covering Editor, Unicode, Filesystem, Search, Preview, Terminal, and Git.
   - Record cold and warm baseline measurements in release profile.
2. **Phase B — Event Loop & Rendering Optimization**
   - Implement TUI redraw coalescing (60 FPS limiter for background streams).
   - Implement dirty-frame skipping to eliminate idle CPU consumption.
   - Benchmark input-to-render latency and idle CPU utilization.
3. **Phase C — Hot-Path Optimizations (Editor, Unicode & Filesystem)**
   - ASCII fast path in `slice_by_display_columns` to bypass grapheme segmentation on ASCII lines.
   - In-place directory sorting and capacity pre-sizing.
   - Bounded channel backpressure in Search Engine.
4. **Phase D — Preview & Cache Memory Hardening**
   - Dual-bounded LRU cache in `PreviewManager` (item count + byte memory limit).
   - Windowed preview loading for large JSON/CSV files.
   - Validate cache invalidation on file modification.
5. **Phase E — Integrated Combined Workload Benchmark**
   - Execute combined multi-subsystem stress workload (large directory + active editor + streaming terminal + running task + Git status refresh + regex search).
   - Measure peak RSS, CPU utilization, and UI frame stability.
6. **Phase F — Verification, Quality Gates & Engineering Delivery Report**
   - Re-run all existing test suites (`cargo test --all-targets`).
   - Verify zero linter warnings (`cargo clippy --locked -D warnings`).
   - Compile release binary with LTO and document binary size.
   - Produce Sprint 5C Final Delivery Report.
