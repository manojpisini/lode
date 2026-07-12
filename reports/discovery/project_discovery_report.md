# Project Discovery Report

**Project**: lode
**Suite**: universal-project-prompt-suite v3.2.0
**Prompt**: UPPS-03
**Mode**: AUDIT_ONLY
**Generated**: 2026-07-11
**Completion**: PASS

## 1. Objective

Build an authoritative, evidence-based map of the lode repository to ground all later specialist reviews.

## 2. Scope

Entire repository: 6 Rust workspace crates, 3 IDE extensions, fuzz targets, CI/CD, documentation.

## 3. Mode and Permissions

**Mode**: AUDIT_ONLY — inspect and report. No files modified outside prompt output directories.
**Permissions**: Builds and tests may run. No network access, no dependency changes, no file mutations.

## 4. Assumptions

- The repository is self-contained with no submodules or nested git repos.
- Cargo workspace resolver v2 is the build system authority.
- The workspace builds on stable Rust with `stable-x86_64-pc-windows-gnu` toolchain.
- All paths are UTF-8 (camino convention).

## 5. Repository Evidence Used

- `Cargo.toml` — workspace manifest, 6 member crates
- All `Cargo.toml` files in each crate — dependency declarations
- All `src/` directories — source file enumeration
- `lib.rs`, `main.rs` files — entry points and public API
- `README.md`, `AGENTS.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CHANGELOG.md` — documentation
- `rust-toolchain.toml`, `rustfmt.toml`, `deny.toml` — configuration
- `.github/workflows/ci.yml`, `.github/workflows/release.yml` — CI/CD
- `extensions/` — VS Code, Neovim, Zed extension sources
- `fuzz/` — fuzz target sources
- `git log --oneline -5`, `git status --short` — version control state
- `cargo check --workspace`, `cargo test --workspace` — build/test verification

## 6. Coverage and Exclusions

**Covered**: All 6 workspace crates, all 46 CLI source files, all 56 lode-core source files, all 52 remaining crate source files, 3 extensions, 2 CI workflows, fuzz targets.
**Exclusions**: `target/` (build artifacts), `node_modules/` (not committed), `Cargo.lock` (generated), `.git/` (git internals), binary/archive files.

## 7. Baseline

### Version Control

```
Last 5 commits (newest first):
  fd07125 fix: address critical audit findings
  0c976ec feat: complete lode project surface
  baa6e90 feat: validate command file mutations
  0283ad1 feat: expand validated filesystem coverage
  7fe33e8 feat: enforce validated project operations

Working tree: ~135 modified files, ~25 untracked files
```

### Build Outcome

```
cargo check --workspace: PASS (0 errors, 0 warnings)
```

### Test Outcome

```
cargo test --workspace: PASS (347 tests, 0 failed)

Breakdown:
  lode-core unit tests:          115 passed
  lode-core integration tests:    11 passed (12 tests, 1 is a doc-test)
  lode-cli unit tests:            13 passed
  lode-cli integration tests:    116 passed
  lode-daemon unit tests:         29 passed
  lode-mcp unit tests:            28 passed
  lode-tui unit tests:            19 passed
  lode-lsp unit tests:            10 passed
  Cross-crate integration tests:   6 passed
```

### Lint Outcome

```
cargo clippy --workspace -- -D warnings: PASS
```

### Format Outcome

```
cargo fmt --all -- --check: PASS
```

## 8. Findings

### Finding 8.1 — Repository Identity

| Attribute | Value | Evidence |
|---|---|---|
| Name | lode | Cargo.toml workspace package name |
| Type | Hybrid CLI + TUI + LSP + MCP + Daemon + Library | Cargo.toml member declarations |
| License | MIT | Cargo.toml `license = "MIT"`, LICENSE file |
| Version | 0.1.0 | Cargo.toml `version = "0.1.0"` |
| Repository | https://github.com/lode-rs/lode | Cargo.toml |
| Language | Rust (edition 2021) | Cargo.toml, rust-toolchain.toml |
| Status | Active development | Multiple untracked files, recent commits |
| Platforms | Linux, macOS, Windows | CI matrix, release workflow |

### Finding 8.2 — Workspace Architecture

```
lode/
  Cargo.toml       # Workspace root (resolver v2)
  crates/
    lode-core/     # Library — shared foundation
    lode-cli/      # Binary — CLI tool (lode)
    lode-daemon/   # Binary+Library — background daemon
    lode-mcp/      # Binary — MCP server
    lode-tui/      # Binary — Terminal UI
    lode-lsp/      # Binary — LSP server
  extensions/
    vscode-lode/   # VS Code extension (TypeScript)
    lode.nvim/     # Neovim plugin (Lua)
    zed-lode/      # Zed extension (Rust/WASM)
```

All crates depend on `lode-core`. No crate depends on other sibling crates at compile time. The daemon is depended on by CLI integration tests only.

### Finding 8.3 — Source Line Counts

| Crate | Source Files | Lines of Rust |
|---|---|---|
| lode-core | 56 | 9,072 |
| lode-cli | 46 | 10,208 |
| lode-daemon | 7 | 1,242 |
| lode-mcp | 24 | 2,721 |
| lode-tui | 19 | 1,847 |
| lode-lsp | 2 | 627 |
| **Total** | **154** | **25,717** |

Plus extensions: ~700 lines TypeScript (vscode-lode), ~360 lines Lua (lode.nvim), ~77 lines Rust/WASM (zed-lode).

### Finding 8.4 — Entry Points

| Crate | Binary Name | Entry File | Purpose |
|---|---|---|---|
| lode-cli | `lode` | `src/main.rs` | CLI dispatch, 60+ commands |
| lode-daemon | `lode-daemon` | `src/main.rs` | Background file watcher |
| lode-mcp | `lode-mcp` | `src/main.rs` | MCP protocol server |
| lode-tui | `lode-tui` | `src/main.rs` | Terminal dashboard |
| lode-lsp | `lode-lsp` | `src/main.rs` | LSP protocol server |

Plus library entry: `lode-core/src/lib.rs` (re-exports all public API).

### Finding 8.5 — CLI Commands (61 total variants, 44 named + 17 alias/shorthand)

All defined via `clap` derive in `lode-cli/src/cmd/types.rs`.

**Core lifecycle**: Setup, Init, Add, Sync, Info
**Dev workflow**: Build, Test, Fmt, Lint, Check, Fix, Verify, Clean, Fresh, Ship
**Config**: Config, Template, Profile, Recipe, Snippet, Commands, Plugin
**Protocol servers**: Mcp, Lsp
**Agent/AI**: Agent, Task
**Scaffolding shortcuts**: Dev, Build, Test, Fmt, Lint, Verify, Clean, Fresh, Ship, Mc, Tauri, Gha, Cp
**Security**: Scan (Secrets/Foreign), Sign, Stamp
**Git**: Git (Branch/Commit/Tag/Changelog/Hooks), Hooks
**Env**: Env (Check/Add/Sync/Use)
**License**: License (List/Show/Info/Add/Remove/Set/Check/Apply)
**Project**: Projects (List/Cd/Register/Remove/Health/Prune)
**Tools**: Toolchain, Pkg, Time, Metrics
**Workspace**: Workspace (Init/List/Add/Remove/Run/Graph)
**Daemon**: Daemon (Start/Stop/Restart/Pause/Resume/ListWatchers/Status/Log)
**Other**: Rules, Rename, Release, Health, Doctor, Export, Import, Serve, Self, Upgrade, Completions, Version, Explain

### Finding 8.6 — Dependencies (171 packages total)

**Key dependencies by category:**

| Category | Crates |
|---|---|
| CLI | clap 4.6, clap_derive |
| Serialization | serde 1.0, serde_json 1.0, toml 0.8 |
| Async | tokio 1.52 |
| TUI | ratatui 0.28, crossterm 0.28 |
| File watching | notify 6.1 |
| Security | sha2 0.10, regex 1.12 |
| Error handling | thiserror 1.0, anyhow 1.0 |
| Filesystem | camino 1.2, tempfile 3.27, dirs 5.0 |
| Testing | assert_cmd 2.2, predicates 3.1 |
| Logging | log 0.4 |

**Deny configuration**: MIT/Apache-2.0 licenses allowed. Two advisories ignored (paste 1.0.15 unmaintained, lru 0.12.5 stacked borrows — both transitive via ratatui). Wildcard dependencies denied.

### Finding 8.7 — Security Model

Three-layer security architecture:

1. **`ValidatedRoot`** (`lode-core/src/fs_safety.rs`): Central path validation. All filesystem writes pass through this. Prevents path traversal, absolute paths, symlink escapes. Atomic writes.

2. **`Process`** (`lode-core/src/process.rs`): Validated process runner. Rejects all shell metacharacters, path separators, null bytes. Only bare program names accepted.

3. **`secrets.rs`** (`lode-core/src/secrets.rs`): Scans for API keys (GitHub tokens, AWS keys), private keys (PEM). Allowlists env files. Skips binaries.

Additional: Template include/extends recursion bound (16), scaffold traversal rejection, recipe destination validation, git hook ownership tracking, dedicated exit codes for security findings.

### Finding 8.8 — MCP Server (38 tools, 8 resources, 3 prompts)

**Tool categories**:
- Lifecycle (4): init, add, sync, info
- Conventions (3): check, fix, rename
- Signatures (2): sign, stamp
- Environment (3): env_check, env_add, env_sync
- Git (4): branch, commit, changelog, tag
- Health (2): audit, metrics
- Package (5): pkg_outdated, pkg_audit, pkg_update, pkg_list, pkg_clean
- Secrets (1): scan
- Release (1): release
- Time (2): time_today, time_report
- Registry (2): projects_list, projects_health
- Agent (2): agent_sync, agent_plan
- Config (3): config_show, config_set, config_validate
- Template (2): template_list, template_show
- Toolchain (2): toolchain_status, toolchain_pin

Transport: STDIO (HTTP not yet implemented). Max message size: 1 MB.

### Finding 8.9 — LSP Server

JSON-RPC 2.0 over stdin/stdout. Capabilities: incremental text sync, completions (Rust keywords, TOML snippets), hover (word-at-cursor analysis), document symbols (fn/struct/enum/trait/impl/mod), code actions (secrets to .env.example, fix naming, rename file). Diagnostics push for secrets and convention violations.

### Finding 8.10 — Daemon

Background file watcher with platform-adaptive IPC (Unix socket on Linux/macOS, TCP on Windows). Idle watchdog for auto-shutdown. Commands: Status, Stop, Pause, Resume, ListWatchers, Reload. Events: Create, Modify, Rename, Delete. Handlers apply signature stamps and verify file signatures.

### Finding 8.11 — TUI

7 panes: Overview, Metrics, Time, Activity, Deps, Files, Registry. 6 custom widgets: Heatmap, BarChart, ScoreRing, StatusBar, Sparkline, Theme. Communicates with daemon via IPC for live event feed. Uses ratatui 0.28 with crossterm backend.

### Finding 8.12 — CI/CD

**CI** (push/PR to main, 10 jobs):
- check, build, audit (cargo audit), test (3-platform matrix), clippy, fmt, deny (cargo-deny), coverage (llvm-cov), fuzz (nightly, 2 targets x 30s), extensions (vscode-lode compile)

**Release** (tag v\* or workflow_dispatch):
- Builds 3 platform binaries, archives (zip/tar.gz), SHA-256 checksums, publishes to GitHub Releases

### Finding 8.13 — Documentation Inventory

| File | Quality | Gaps |
|---|---|---|
| README.md | Good overview, CLI reference | No architecture diagram, no MCP protocol details |
| AGENTS.md | Concise build/security guide | No crate dependency graph |
| CONTRIBUTING.md | Pre-commit checklist | No extension development guide |
| SECURITY.md | Policy and reporting | No threat model, no security boundaries map |
| CHANGELOG.md | Unreleased entries only | No versioned releases yet |
| LODE_DESIGN_FINAL.md | Design document | May be stale |
| rustfmt.toml | Active | N/A |

### Finding 8.14 — Extensions

| Extension | Language | Lines | Features |
|---|---|---|---|
| vscode-lode | TypeScript | ~700 | Diagnostics, status bar, decorations, LSP client |
| lode.nvim | Lua | ~360 | Async diagnostics, floating preview, keymaps |
| zed-lode | Rust/WASM | ~77 | Slash commands (check, scan, status, init) |

### Finding 8.15 — Fuzz Testing

2 targets: `validated_root` (ValidatedRoot::new + resolve with random bytes), `process_validation` (Process::new with random bytes). Both run for 30s each in CI on nightly Rust.

## 9. Proposed or Applied Changes

None (AUDIT_ONLY mode).

## 10. Verification

- `Verified`: All source files enumerated via `Get-ChildItem`
- `Verified`: All Cargo.toml files read and dependencies extracted
- `Verified`: `cargo check --workspace` passes
- `Verified`: `cargo test --workspace` passes (347/347)
- `Verified`: `cargo clippy --workspace -- -D warnings` passes
- `Verified`: `cargo fmt --all -- --check` passes
- `Verified`: git state matches `git log` and `git status`
- `Verified`: All 38 MCP tools enumerated from source
- `Verified`: All 8 MCP resources and 3 prompts identified

## 11. Compatibility Impact

None — no changes proposed or applied.

## 12. Security and Data Impact

None — no data accessed, no secrets exposed, no files modified.

## 13. Remaining Risks

1. ~135 modified and ~25 untracked files in working tree — pre-existing state, not related to this audit.
2. 171 dependencies is a large supply chain — UPPS-13 recommended for deeper review.
3. No dependency audit (cargo audit) results captured — permission not granted for network access.
4. Two cargo-deny advisory ignores exist (paste, lru) — should be reviewed periodically.
5. Reference to `LODE_DESIGN_FINAL.md` may be stale vs. current implementation.
6. Extensions (vscode-lode, lode.nvim) have no test suites in CI.
7. Fuzz coverage is limited to 2 targets at 30s each.

## 14. Unknowns

- Actual vulnerability status (requires `cargo audit` with network access)
- Git commits beyond last 5
- Whether `prompts/` directory should be tracked or gitignored (currently tracked)
- Whether `LODE_DESIGN_FINAL.md` reflects current architecture
- Whether `outputs/` directory has any content (gitignored)
- Whether MSVC toolchain alternative is tested in CI

## 15. Rollback or Recovery Information

Not applicable — no changes made.

## 16. Prioritized Next Actions

1. **High**: UPPS-07 — generate comprehensive project documentation (architecture, CLI reference, security model, MCP tools, contributor guide)
2. **Medium**: UPPS-15 — review CLI and MCP public contracts for completeness
3. **Medium**: UPPS-13 — dependency and supply chain review
4. **Low**: UPPS-24 — project health completeness audit after specialist reviews
5. **Low**: UPPS-23 — release readiness if a release is planned

## 17. Completion State

**PASS** — All discovery objectives met:
- Repository inventory complete (6 crates, 154 source files, 25,717 lines)
- Entry point map complete (5 binaries + 1 library, 61 CLI command variants)
- Component map complete (all modules with descriptions)
- Command inventory complete (all CI, build, test, lint, format commands)
- Baseline captured (git state, build outcome, test outcome, lint outcome)
- All required artifacts written (reports + 4 JSON inventories)
