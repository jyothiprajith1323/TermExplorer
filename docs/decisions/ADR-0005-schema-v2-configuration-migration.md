# ADR-0005: Schema Version 2 Configuration & Backward-Compatible Migration Engine

- **Status**: Accepted
- **Date**: 2026-11-14 (Sprint 6B)

---

## 1. Context & Problem Statement

Prior to Sprint 6B, TermExplorer used an unversioned `config.toml` structure. As new subsystems were added (PTY terminal, Git integration, Workflow Task Runner, WebAssembly plugins), settings became fragmented across ad-hoc structs. Users upgrading from earlier versions risked losing customizations, and corrupted files caused startup crashes.

## 2. Decision

TermExplorer establishes **Configuration Schema Version 2** (`src/config/`):
- Explicit `schema_version = 2` top-level integer key.
- 9 typed tables: `[general]`, `[layout]`, `[theme]`, `[editor]`, `[terminal]`, `[search]`, `[developer]`, `[monitor]`, `[keybindings]`.
- **Automatic Migration**: Unversioned files are classified as Version 1; `ConfigMigrator::migrate_v1_to_v2()` preserves user customizations, creates `config.toml.v1.bak`, and fills new sections with defaults.
- **Future Version Rejection**: Files with `schema_version > 2` are rejected to prevent older binaries from corrupting newer configurations.
- **Atomic Persistence & Backup**: Writes to `.tmp`, flushes buffers with `sync_all()`, copies original to `.bak`, and renames atomically.
- **Corruption Recovery**: Malformed files are archived to `config.toml.corrupted.<timestamp>` and clean defaults are loaded without crashing.

## 3. Alternatives Considered

1. **Silent Fallback to Defaults**: Wipes user customizations when parsing fails (unacceptable data loss).
2. **Untyped TOML Value Map**: Allows arbitrary keys but removes compile-time type safety and validation clamping.

## 4. Consequences

- **Positive**: Zero-loss upgrades for existing users; guaranteed valid settings via `validate_and_clamp()`; corruption recovery.
- **Negative**: Adding new configuration tables in the future requires updating the migration engine and tests.
