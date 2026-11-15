# Testing Strategy & Quality Gates

TermExplorer relies on a multi-layered testing methodology combining unit, integration, headless UI, and adversarial filesystem tests.

---

## 1. Testing Pyramid

```text
       ▲
      / \      Headless UI & Visual Tests (TestBackend assertions)
     /───\     
    /     \    Adversarial Filesystem & Safety Tests (symlink loops, permissions)
   /───────\   
  /         \  Subsystem Integration Tests (Git, PTY, Workflow, Plugins, Config)
 /───────────\ 
/             \ Unit Tests (Rope indexers, parsers, clamp logic, hashers)
───────────────
```

---

## 2. Integration Test Suites (`tests/`)

The repository contains 26 comprehensive integration test files:
- `config_and_first_run_tests.rs`: Schema v2, migration from v1, first-run status detection, future schema rejection, Settings Studio 14 sections.
- `plugin_tests.rs`: Wasmi sandboxing, 5M fuel limit, 16 MiB memory limit, SHA-256 integrity check, capability permission enforcement.
- `workflow_tests.rs`: Command registry, fuzzy search ranking, Task DAG cycle detection, project stack detection, variable expansion.
- `git_tests.rs`: Status queries, unified diff parsing, branch creation, commit creation, and index staging.
- `adversarial_filesystem_tests.rs`: Malformed filenames, broken symlinks, deeply nested directory hierarchies, read-only permissions.
- `terminal_tests.rs` / `terminal_edge_tests.rs`: PTY creation, non-blocking stream handling, escape sequence parsing, resize events.
- `unicode_and_emoji_tests.rs`: Multibyte UTF-8 handling, compound emojis, zero-width joiners, fullwidth CJK characters.
- `preview_tests.rs` / `preview_media_tests.rs`: Image downsampling, PDF text extraction, audio tag parsing, SQLite schema inspection.

---

## 3. Pre-Commit Quality Checklist

Before submitting code, contributors must run:
1. `cargo fmt --all -- --check`
2. `cargo check --locked --all-targets`
3. `cargo test --all-targets`
4. `cargo clippy --all-targets --all-features -- -D warnings`
5. `cargo doc --no-deps`
