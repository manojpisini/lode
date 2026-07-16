# Changelog

## v0.1.1 — 2026-07-17

### Added

- **Template bundles**: `lode template-bundle` with `capture`, `apply`, `validate`, `verify`, `show`, `preview` subcommands
- **MCP**: 6 new template bundle tools (`lode_template_bundle_list`, `lode_template_bundle_show`, `lode_template_bundle_validate`, `lode_template_bundle_preview`, `lode_template_bundle_apply`, `lode_template_bundle_capture`) + `lode://template-bundles` resource (44 tools, 9 resources total)
- **CLI commands**: `file`, `context`, `handoff`, `diagnose`, `docs`, `dep-graph`, `cache`, `env-snapshot`, `assets`, `pack`, `plan`, `policy`, `project`, `lock`, `receipts`, `archetype`, `agent-sim`, `sandbox`, `secret-vault`, `migration`, `mc`, `tauri`, `gha`, `cp`
- **Search index**: persistent local search index with inverted index (FTS5-style)
- **Asset catalog**: lifecycle states and quality ratings
- **Context compiler**: token-budgeted context compilation (`lode context compile --budget`)
- **Managed file ownership**: `file-manifest.json` tracking system
- **Agent policy generator**: `lode agent policy` generates canonical agent policy files
- **Change-aware verification**: `lode verify --changed`
- **Documentation**: 22 docs covering architecture, guides, reference, operations, user manual
- **CI**: `cargo audit` and `cargo deny check` jobs added to CI pipeline
- **Fuzz testing**: CI fuzz jobs for `ValidatedRoot` and `Process` validation

### Changed

- **Cleanup**: removed ~765 lines of dead code (unused IPC/formatting/git abstractions, 3 duplicated `load_config` functions consolidated)
- **TUI**: replaced custom `Sparkline` widget with ratatui built-in, removed dead IPC module (protocol mismatch)
- **Daemon**: removed `StateFile` wrapper, dead methods, unused `Default` impls
- **MCP**: removed unused `Default for McpServer`, deduplicated `load_config` into shared util

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

_No unreleased changes._
