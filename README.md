# LODE

[![CI](https://github.com/manojpisini/lode/actions/workflows/ci.yml/badge.svg)](https://github.com/manojpisini/lode/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/lode-cli.svg)](https://crates.io/crates/lode-cli)
[![docs.rs](https://img.shields.io/docsrs/lode-core)](https://docs.rs/lode-core/latest/lode_core/)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**LODE** — an all-in-one local developer tool for Rust projects.  
Manage projects, track time, scan secrets, enforce conventions, run daemons, serve MCP/LSP protocols, manage plugins and packages, automate git workflows, manage template bundles, and export/import portable LodePacks.

## Crates.io packages

| Crate | Version | Description |
|---|---|---|
| [lode-cli](https://crates.io/crates/lode-cli) | [![crates.io](https://img.shields.io/crates/v/lode-cli.svg)](https://crates.io/crates/lode-cli) | CLI binary — the main entry point |
| [lode-core](https://crates.io/crates/lode-core) | [![crates.io](https://img.shields.io/crates/v/lode-core.svg)](https://crates.io/crates/lode-core) | Core library: config, rules, secrets, scaffold, git |
| [lode-daemon](https://crates.io/crates/lode-daemon) | [![crates.io](https://img.shields.io/crates/v/lode-daemon.svg)](https://crates.io/crates/lode-daemon) | Background file watcher with IPC |
| [lode-mcp](https://crates.io/crates/lode-mcp) | [![crates.io](https://img.shields.io/crates/v/lode-mcp.svg)](https://crates.io/crates/lode-mcp) | MCP server: 38 tools, 9 resources, 3 prompts |
| [lode-tui](https://crates.io/crates/lode-tui) | [![crates.io](https://img.shields.io/crates/v/lode-tui.svg)](https://crates.io/crates/lode-tui) | Terminal UI with 7 panes |
| [lode-lsp](https://crates.io/crates/lode-lsp) | [![crates.io](https://img.shields.io/crates/v/lode-lsp.svg)](https://crates.io/crates/lode-lsp) | LSP server over JSON-RPC |

## Installation

### From crates.io (recommended)

```bash
cargo install lode-cli
```

### From source (GitHub)

```bash
git clone https://github.com/manojpisini/lode.git
cd lode
cargo build -p lode-cli
./target/debug/lode --help
```

### Requirements

- **Rust toolchain** (`stable-x86_64-pc-windows-gnu` or MSVC)
- **Windows**: MSYS2 ucrt64 (`C:\msys64\ucrt64\bin` in PATH) for native deps, or use MSVC toolchain

## Quick start

```bash
# Set up lode with defaults
lode setup --defaults

# Create a new project
lode init my-project --profile core/bare

# Add components
lode add testing

# Check project health
lode health

# View configuration
lode config show

# Manage environment variables
lode env check
lode env add DATABASE_URL --secret

# Track time
lode time today
lode time report

# Git integration
lode git commit "feat: add new feature"
lode git branch feature login-system

# Start the TUI dashboard
lode serve
```

## Documentation

See [docs/](docs/index.md) for architecture, guides, operations, and reference documentation.

## Build

```bash
cargo check --workspace   # Check all 6 crates compile
cargo test --workspace    # Run all tests
cargo build -p lode-cli   # Build the CLI binary
cargo build -p lode-lsp   # Build the LSP server
```

## License

MIT

## CLI Command Reference

| Command | Description | Key Subcommands |
|---|---|---|
| `setup` | Initialize lode defaults | `--defaults` |
| `init` / `new` | Create a new project | `--profile`, `--with`, `--dry-run`, `--lang` |
| `add` | Add a component | `--dry-run`, `--overwrite` |
| `sync` | Sync scaffold/agent/metrics | `--dry-run`, `--force`, `--section` |
| `info` | Show project info | `--json` |
| `config` | Configuration | `show`, `validate`, `diff`, `set`, `reset`, `edit` |
| `template` | Templates | `list`, `show`, `reset`, `validate`, `edit` |
| `template-bundle` | Template bundles | `capture`, `apply`, `validate`, `verify`, `show`, `preview` |
| `profile` | Profiles | `list`, `show`, `use`, `new`, `delete` |
| `recipe` | Recipes | `list`, `show`, `apply`, `compose`, `new` |
| `commands` | Custom command macros | `list`, `show`, `add`, `remove`, `run`, `edit` |
| `plugin` | Plugins | `list`, `search`, `add`, `remove`, `update`, `info` |
| `mcp` | MCP server | `--list-tools`, `--list-resources`, `--list-prompts` |
| `lsp` | LSP server | `--stdio`, `--capabilities` |
| `agent` | AI agents | `sync`, `status`, `export`, `plan`, `policy` |
| `agent-sim` | Agent simulation | `run` |
| `snippet` | Code snippets | `list`, `show`, `search`, `add`, `remove`, `export`, `edit` |
| `task` | Current task | `target`, `--no-store` |
| `dev` / `build` / `test` / `fmt` / `lint` | Workflow runners | — |
| `check` | Convention check | `path`, `--json`, `--fix` |
| `fix` | Auto-fix conventions | `path` |
| `rename` | Rename files | `path`, `--to` |
| `rules` | Convention rules | `list`, `check`, `validate` |
| `sign` | File signatures | `path`, `--ext`, `--force` |
| `stamp` | License headers | `path`, `--ext`, `--license` |
| `verify` | File verification | `--changed` |
| `clean` / `fresh` / `ship` | Build helpers | — |
| `release` | Version release | `version`, `--bump`, `--dry-run`, `--rollback` |
| `health` / `audit` | Project audit | — |
| `explain` / `doctor` | Helpers | `--fix`, `--json` |
| `scan` | Secret/code scanning | `secrets`, `foreign` |
| `git` | Git automation | `branch`, `commit`, `tag`, `changelog` |
| `hooks` | Git hooks | `list`, `status`, `test`, `run` |
| `env` | Env variables | `check`, `add`, `sync`, `use` |
| `license` | Licenses | `list`, `show`, `add`, `remove`, `set`, `check` |
| `projects` | Project registry | `list`, `cd`, `register`, `remove`, `health` |
| `toolchain` | Runtime toolchains | `list`, `status`, `add`, `remove`, `use`, `pin` |
| `pkg` | Package management | `list`, `outdated`, `update`, `audit`, `graph` |
| `time` | Time tracking | `today`, `show`, `report`, `clear` |
| `metrics` | Project metrics | `show`, `trend`, `baseline` |
| `file` | Managed files | `add`, `list`, `check`, `remove` |
| `context` | Context compilation | `compile`, `check` |
| `handoff` | Agent handoff | `generate` |
| `diagnose` | Error diagnosis | `check`, `list` |
| `docs` | Documentation | `open` |
| `dep-graph` | Dependency graph | `resolve`, `dot` |
| `cache` | Cache management | `stats`, `clear` |
| `env-snapshot` | Env snapshots | `list`, `diff` |
| `assets` | Asset catalog | `list`, `show`, `search` |
| `pack` / `plan` / `policy` | Project planning | `create`, `apply` |
| `project` | Project config | `init`, `show` |
| `lock` / `receipts` | State tracking | `list`, `verify` |
| `archetype` | Archetype resolution | `list`, `resolve` |
| `sandbox` | Sandboxed execution | `create`, `run`, `clean` |
| `secret-vault` | Secret management | `list`, `get`, `set`, `grant` |
| `migration` | Schema migrations | `check`, `run` |
| `workspace` | Workspace management | `init`, `list`, `add`, `graph` |
| `daemon` | Background daemon | `start`, `stop`, `status`, `log` |
| `log` | Log management | `init`, `daemon`, `clear` |
| `export` / `import` | LodePack transfer | `--out`, `--force` |
| `serve` | TUI dashboard | `--pane`, `--no-color` |
| `self` / `upgrade` | Self-management | `info`, `clean`, `--check` |
| `completions` | Shell completions | `shell`, `--install` |
| `version` | Show version | — |
| `mc` / `tauri` / `gha` / `cp` | Domain helpers | — |

## Configuration

See [docs/reference/config.md](docs/reference/config.md) for the full configuration reference with all 30+ sections.
