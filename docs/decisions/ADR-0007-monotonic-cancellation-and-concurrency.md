# ADR-0007: Monotonic Generation Tokens for Async Cancellation

- **Status**: Accepted
- **Context**: Reconstructed from Current Implementation (Sprint 5C)

---

## 1. Context & Problem Statement

In interactive developer tools, users rapidly trigger background tasks (scrolling through directory listings, typing search queries, opening preview tabs). If previous asynchronous tasks continue running to completion, they waste CPU cycles, exhaust memory, and potentially emit stale results that overwrite newer state.

## 2. Decision

TermExplorer employs **Monotonic Generation Tokens** (`Arc<AtomicU64>`):
- A central counter is incremented on every user interaction that supersedes previous work.
- Workers are assigned the generation value at task creation.
- Inside processing loops (directory traversal, regex scanning, image downsampling), workers periodically compare their assigned generation against the atomic counter.
- If the generation has advanced, the worker yields immediately and terminates.

## 3. Alternatives Considered

1. **`tokio::task::JoinHandle::abort()`**: Aborts tasks abruptly, but does not allow workers to cleanly close open file descriptors or release temporary files.
2. **Standard Boolean Flags (`AtomicBool`)**: Only supports single active tasks; cannot distinguish between multiple queued or overlapping generations.

## 4. Consequences

- **Positive**: Low-overhead cancellation check; prevents stale-result race conditions; eliminates thread pool starvation.
- **Negative**: Workers must explicitly yield and check tokens at loop boundaries.
