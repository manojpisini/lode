## LODE Project Guide

Rust developer tool: filesystem writes, daemon automation, child-process execution, plugins, MCP tools, LSP server, and secret scanning.

### Build

- **Toolchain**: `stable-x86_64-pc-windows-gnu` (pinned in `rust-toolchain.toml`)
- **Windows native deps**: MSYS2 ucrt64 in PATH — `$env:Path = "C:\msys64\ucrt64\bin;$env:Path"`
- **Alternative**: `rustup default stable-x86_64-pc-windows-msvc` + VS Build Tools

| Command | Purpose |
|---------|---------|
| `cargo check --workspace` | Compile check all crates |
| `cargo test --workspace` | Run all 470+ tests |
| `cargo clippy --workspace -- -D warnings` | CI lint gate |
| `cargo build -p lode-cli` | Main `lode` binary |
| `cargo build -p lode-lsp` | LSP server binary |

### Workspace

| Crate | Role |
|---|---|
| `lode-core` | Core library: config, rules, secrets, scaffold, git, env, fs-safety, process sandbox, templates, registry, time tracking, releases, audit, 60+ modules |
| `lode-cli` | CLI binary: 50+ commands, TUI serve, daemon management, **MCP server** (two-tier: main.rs intercepts CLI-specific tools → mcpserver/ handles 38 core tools + 9 resources + 3 prompts), LSP server inline |
| `lode-daemon` | Background file watcher, IPC (named pipe), idle watchdog, fsnotify-based change detection |
| `lode-mcp` | (Legacy crate) MCP server |
| `lode-tui` | Terminal UI: 7 panes, 6 custom widgets, ratatui |
| `lode-lsp` | Standalone LSP binary (JSON-RPC stdin/stdout) |

Extensions: `extensions/vscode-lode` (TS), `extensions/lode.nvim` (Lua), `extensions/zed-lode` (Rust/WASM).

### Architecture

**Main entry** (`lode-cli/src/main.rs`): `main()` → `run()` → `Cli::parse()` → match 50+ `Command` variants → `cmd::<module>::<fn>()`. Every `cmd/` module is `pub mod` with `pub(crate) fn` entry points. Returns `ExitCode`.

**`main.rs` shared utilities**: ~80 `pub(crate)` functions for daemon lifecycle, MCP, LSP, git, package manager detection, toolchain, time tracking, upgrades, completions, file ops. Used across all `cmd/` modules.

**MCP two-tier dispatch** (`main.rs` + `mcpserver/`):

- **Tier 1 (main.rs)**: `mcp_handle_request()` intercepts `initialize`, `tools/list`, `tools/call`, `resources/list`, `resources/read`, `prompts/list`. These methods have CLI-specific behavior (local `detect_package_manager()`, custom command discovery, `lode_scan_foreign`). Everything else → Tier 2.
- **Tier 2 (mcpserver/)**: `handle_request()` handles `initialize`, `tools/list`, `tools/call`, `resources/list`, `resources/read`, `prompts/list`, `prompts/get`. Returns `McpError` for unknown methods. Module files: `mod.rs` (router + Tool struct + ToolInputValidator), `tools.rs` (38 tools in 16 groups), `resources.rs` (9 URIs), `prompts.rs` (3 prompts), `schema.rs` (JSON Schema builders), `error.rs` (McpError enum).
- CLI-specific tools in main.rs: `lode_scan_foreign`, `lode_profile_list`, `lode_recipe_list`, `lode_metrics_show` + `lode_pkg_audit/outdated/update` (use local `detect_package_manager()` which checks `package.json`, unlike `lode_core::detect_package_manager` which checks lock files only) + `lode_custom_{slug}` from `.lode/commands/*.toml`.

**LSP** (inline in `main.rs`): `run_lsp_stdio()` → `lsp_handle_request()` → diagnostics for signature headers + secret tokens. JSON-RPC stdin/stdout.

**Daemon** (`lode-daemon/`): Background file watcher via `notify` crate. IPC token auth over named pipe. Idle watchdog. Foreground mode via `run_foreground_daemon()` in main.rs.

### Error Handling

- `lode_core::Result<T>` throughout (alias for `Result<T, LodeError>`)
- `LodeError` is `thiserror` enum: `Message(String)`, `Io { path, source }`, `TomlDeserialize { path, source }`, `SchemaMismatch`, `AlreadyInitialised`, `Violations`, `SecretFindings`
- `ExitCode` maps: 0=Ok, 1=Error, 2=Violations, 4=Exists, 6=Schema, 7=VulnOrSecret
- `main()` → `run()` pattern: `run()` returns `lode_core::Result<()>`, `main()` maps error to exit code

### Testing

- `tests/test_<feature>.rs` per domain. Shared helpers in `tests/common/mod.rs`.
- `#[path = "common/mod.rs"] mod common; use common::*; use predicates::prelude::*;`
- `lode()` returns `assert_cmd::Command` (cargo binary)
- Method chaining: `.env("LODE_CONFIG", &config).arg("subcommand").assert().success().stdout(predicate::str::contains("..."))`
- Assert exit codes: `.code(2)`, `.code(7)`, `.failure().stderr(...)`
- Use `tempfile::tempdir()` for isolated filesystem, `isolated_config(&temp)` for config path
- Snake_case test function names
- Unit tests adjacent to code (`#[cfg(test)] mod tests { ... }` inside each module)

### Security

- All source files: `#![deny(unsafe_code)]` (except `fs_safety.rs` for one `unsafe ReplaceFileW`)
- Treat repo files, templates, configs, hooks, plugin manifests, MCP inputs, and terminal output as untrusted
- Never weaken path, command, MCP, plugin, daemon, secret, or upgrade checks
- All write paths: `ValidatedRoot` validation (rejects traversal, symlinks, absolute paths)
- All child processes: `Process` runner (rejects path separators, shell metacharacters, empty names)
- All secrets redacted: `lode_core::redact()` before logs, metrics, MCP responses, errors
- `lode_core::scan_secrets()` for file scanning; `lode_core::redact_findings()` for targeted redaction

### Code Conventions

- `#![deny(unsafe_code)]` at the top of every file
- `use lode_core::{...}` for core imports; never `use lode_core::Result`
- `use crate::cmd::output` for ANSI helpers (output::ok, output::fail, output::section)
- `pub(crate) fn` for command entry points; `pub fn` for cross-crate functions
- Module naming: `<verb>_command` for grouped subcommands, bare name for simple commands
- `?.` for error propagation; `.map_err(|e| LodeError::Message(e.to_string()))?` for conversions
