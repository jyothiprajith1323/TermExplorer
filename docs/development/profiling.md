# Profiling & Performance Engineering

TermExplorer is engineered for low latency, smooth 60 FPS rendering, and minimal memory usage.

---

## 1. Real-Time Telemetry Overlay (`F12`)

Press `F12` during execution to display the live performance overlay:
- **Render Latency**: Time spent constructing the Ratatui frame (target: <2,000 $\mu\text{s}$).
- **Instantaneous & Rolling FPS**: Ensures terminal diffing does not drop below 60 FPS.
- **Estimated Heap Allocation**: Tracks active buffer memory and cache capacity.

---

## 2. CPU Profiling with `samply` and `flamegraph`

To profile CPU hotspots during heavy operations (such as opening 50,000-line files or walking large directories):

```bash
# Profile with samply (interactive web-based profiler)
cargo install samply
samply record ./target/release/termexplorer

# Profile with cargo-flamegraph
cargo install flamegraph
cargo flamegraph --bin termexplorer -- /path/to/large/project
```

The resulting `flamegraph.svg` reveals function call durations across `syntect`, `ropey`, and `crossterm`.

---

## 3. Allocation Profiling with DHAT & Heaptrack

To inspect heap allocation bottlenecks and check for memory leaks:
```bash
# Run with heaptrack
heaptrack ./target/release/termexplorer
heaptrack_gui heaptrack.termexplorer.*.gz
```
Verify that long-running terminal sessions or repeatedly opening previews does not lead to monotonic heap growth.
