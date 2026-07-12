# Changelog

## v0.1.0 — 2026-07-11

### Added

- Centralized `redact_secrets()` module in lode-core for output-boundary secret filtering.
- Daemon IPC token-based authentication (random 32-byte token on daemon start).
- `lode scan secrets` step in CI pipeline.
- SBOM generation via `cargo-cyclonedx` in release workflow.
- Dependabot configuration for weekly cargo and monthly GitHub Actions updates.
- `security.txt` (RFC 9116) and `CODE_OF_CONDUCT.md`.
- LSP diagnostics redaction config option (`lsp.redact_diagnostics`, default `true`).
- MCP tool input validation layer with JSON Schema enforcement.
- Plugin permission runtime enforcement (execute/network/fs_write gates).

### Security

- All MCP tool functions accepting a `path` argument now validate through `ValidatedRoot` (26 functions across 11 modules).
- `.env` values are sanitized to reject newlines, carriage returns, and null bytes.
- MCP transport enforces a 1 MB input size limit.
- Version strings in `lode_toolchain_pin` are validated (rejects path separators, null bytes, invalid chars).
- Hardcoded `sh` in hooks replaced with platform-appropriate shell selection (`cmd /c` for `.bat`/`.cmd` on Windows).
- Redundant `return` and unnecessary borrow fixed across the codebase.

### CI

- CI matrix expanded to `ubuntu-latest`, `macos-latest`, and `windows-latest`.
- MSYS2 setup added for Windows test runner (GNU toolchain).
- `rust-toolchain.toml` enables auto-detection of components.
- `cargo clippy` passes with `-D warnings`.

### Testing

- **lode-daemon**: 3 → 22 tests (state machine, IPC parsing, handler file I/O).
- **lode-mcp**: 3 → 28 tests (error codes, schema builders, tool dispatch invariants).
- **lode-lsp**: 2 → 10 tests (secret scanning edge cases, filename conventions).

### Dependencies

- Tokio narrowed from `features = ["full"]` to explicit feature set.
- `sha2` deduplicated to workspace level.
- Minimum versions pinned: serde >=1.0.200, serde_json >=1.0.120, thiserror >=1.0.60, regex >=1.10, notify >=6.1.

## Unreleased

### Security

- All MCP tool functions accepting a `path` argument now validate through `ValidatedRoot` (26 functions across 11 modules).
- `.env` values are sanitized to reject newlines, carriage returns, and null bytes.
- MCP transport enforces a 1 MB input size limit.
- Version strings in `lode_toolchain_pin` are validated (rejects path separators, null bytes, invalid chars).
- Hardcoded `sh` in hooks replaced with platform-appropriate shell selection (`cmd /c` for `.bat`/`.cmd` on Windows).
- Redundant `return` and unnecessary borrow fixed across the codebase.

### CI

- CI matrix expanded to `ubuntu-latest`, `macos-latest`, and `windows-latest`.
- MSYS2 setup added for Windows test runner (GNU toolchain).
- `rust-toolchain.toml` enables auto-detection of components.
- `cargo clippy` passes with `-D warnings`.

### Testing

- **lode-daemon**: 3 → 22 tests (state machine, IPC parsing, handler file I/O).
- **lode-mcp**: 3 → 28 tests (error codes, schema builders, tool dispatch invariants).
- **lode-lsp**: 2 → 10 tests (secret scanning edge cases, filename conventions).

### Dependencies

- Tokio narrowed from `features = ["full"]` to explicit feature set.
- `sha2` deduplicated to workspace level.
- Minimum versions pinned: serde >=1.0.200, serde_json >=1.0.120, thiserror >=1.0.60, regex >=1.10, notify >=6.1.

### Known Release Blockers

- CLI monolith extraction (10,274-line `main.rs`).
- Full Windows IPC verification.
- Broader TUI test coverage.