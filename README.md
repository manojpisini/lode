# LODE

[![CI](https://github.com/manojpisini/lode/actions/workflows/ci.yml/badge.svg)](https://github.com/manojpisini/lode/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/lode-cli.svg)](https://crates.io/crates/lode-cli)
[![docs.rs](https://img.shields.io/docsrs/lode-core)](https://docs.rs/lode-core/latest/lode_core/)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**LODE** — an all-in-one local developer tool for Rust projects.  
Manage projects, track time, scan secrets, enforce conventions, run daemons, serve MCP/LSP protocols, manage plugins and packages, automate git workflows, and export/import portable LodePacks.

## Crates.io packages

| Crate | Version | Description |
|---|---|---|
| [lode-cli](https://crates.io/crates/lode-cli) | [![crates.io](https://img.shields.io/crates/v/lode-cli.svg)](https://crates.io/crates/lode-cli) | CLI binary — the main entry point |
| [lode-core](https://crates.io/crates/lode-core) | [![crates.io](https://img.shields.io/crates/v/lode-core.svg)](https://crates.io/crates/lode-core) | Core library: config, rules, secrets, scaffold, git |
| [lode-daemon](https://crates.io/crates/lode-daemon) | [![crates.io](https://img.shields.io/crates/v/lode-daemon.svg)](https://crates.io/crates/lode-daemon) | Background file watcher with IPC |
| [lode-mcp](https://crates.io/crates/lode-mcp) | [![crates.io](https://img.shields.io/crates/v/lode-mcp.svg)](https://crates.io/crates/lode-mcp) | MCP server: 38+ tools, 8 resources, 3 prompts |
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

| Command | Description | Key Flags |
|---|---|---|
| `setup` | Initialize lode defaults | `--defaults` |
| `init` (`new`) | Create a new project | `-p/--path`, `--profile`, `--with`, `--dry-run`, `--overwrite`, `--no-git`, `--lang`, `--preset`, `--license`, `--extra`, `--no-check`, `-y/--yes` |
| `add` | Add a component to the project | `--dry-run`, `--overwrite` |
| `sync` | Sync config/templates/agent/metrics | `--dry-run`, `--force`, `--section` |
| `info` | Show project information | `--json` |
| `config` | Configuration management | `show`, `validate`, `diff`, `set`, `reset`, `edit` |
| `template` | Template library management | `list`, `show`, `diff`, `reset`, `validate`, `edit` |
| `profile` | Profile management | `list`, `show`, `use`, `new`, `delete` |
| `recipe` | Recipe management | `list`, `show`, `apply`, `compose`, `new` |
| `commands` | Custom command macros | `list`, `show`, `add`, `remove`, `export`, `run`, `edit` |
| `plugin` | Plugin system | `list`, `search`, `add`, `remove`, `update`, `info` |
| `mcp` | MCP protocol server | `--http`, `--port`, `--list-tools`, `--list-resources`, `--list-prompts` |
| `lsp` | LSP protocol server | `--stdio`, `--capabilities` |
| `agent` | AI agent file generation | `sync`, `status`, `export`, `plan` |
| `snippet` | Code snippet management | `list`, `show`, `search`, `add`, `remove`, `insert`, `export`, `edit` |
| `task` | Track current work task | `target`, `--no-store` |
| `dev` | Run dev workflow | — |
| `build` | Run build workflow | — |
| `test` | Run test workflow | — |
| `fmt` | Run code formatter | — |
| `lint` | Run linter | — |
| `check` | Convention compliance check | `path`, `--json`, `--fix` |
| `fix` | Auto-fix conventions | `path` |
| `rename` | Rename files/folders | `path`, `--to` |
| `rules` | Convention rules management | `list`, `check`, `validate` |
| `sign` | Insert/update file signatures | `path`, `--ext`, `--force`, `--dry-run` |
| `stamp` | Insert/update license headers | `path`, `--ext`, `--license`, `--dry-run` |
| `verify` | Run project verification | — |
| `clean` | Clean build artifacts | — |
| `fresh` | Clean and full rebuild | — |
| `ship` | Verify and release | — |
| `release` | Version release management | `version`, `--bump`, `--dry-run`, `--rollback` |
| `health` / `audit` | Project health audit | — |
| `explain` | Explain a concept | — |
| `doctor` | System diagnostics | `--fix`, `--json` |
| `scan` | Scan for secrets/foreign code | `secrets`, `foreign` |
| `git` | Git workflow automation | `branch`, `commit`, `tag`, `changelog`, `install-hooks`, `uninstall-hooks`, `hooks-status`, `sign-setup`, `remote-setup` |
| `hooks` | Git hooks management | `list`, `status`, `test`, `run` |
| `env` | Environment variable management | `check`, `add`, `sync`, `use` |
| `license` | License management | `list`, `show`, `info`, `add`, `remove`, `set`, `check`, `apply` |
| `projects` | Project registry | `list`, `cd`, `register`, `remove`, `health`, `prune` |
| `toolchain` | Runtime toolchain management | `list`, `status`, `doctor`, `add`, `remove`, `use`, `pin`, `update` |
| `pkg` | Package management | `list`, `outdated`, `update`, `audit`, `why`, `info`, `lock`, `graph`, `clean` |
| `time` | Time tracking | `today`, `show`, `report`, `clear` |
| `metrics` | Project metrics | `show`, `trend`, `baseline`, `diff-baseline` |
| `workspace` | Multi-crate workspace management | `init`, `list`, `add`, `remove`, `run`, `graph` |
| `daemon` | Background file watcher daemon | `start`, `stop`, `restart`, `pause`, `resume`, `list-watchers`, `status`, `log` |
| `log` | Log management | `init`, `daemon`, `clear` |
| `export` | Export project as LodePack | `--out`, `--no-plugins`, `--no-templates`, `--no-snippets`, `--no-licenses`, `--no-recipes`, `--no-commands`, `--include-metrics` |
| `import` | Import LodePack | `path`, `--no-merge`, `--force` |
| `serve` | TUI dashboard mode | `--no-color`, `--no-live`, `--pane`, `--refresh`, `--theme` |
| `self` | Self-management | `info`, `clean`, `uninstall` |
| `upgrade` | Self-upgrade | `--check`, `--manifest`, `--dry-run`, `--rollback` |
| `completions` | Generate shell completions | `shell`, `--install`, `--dry-run`, `--out` |
| `version` | Show version | — |

## Configuration Reference

Configuration is stored at `~/.lode/config.toml` and merged with project-level `.lode/project.toml`. The config file contains these sections:

```toml
schema_version = 3
active_profile = "core/bare"       # Active profile name

[identity]                         # Author identity
author = "Your Name"
name = ""
email = "you@example.com"
org = ""
url = ""
license = "MIT OR Apache-2.0"

[convention]                       # Naming and file conventions
folder_case = "snake_case"
file_case = "snake_case"
default_case = "snake_case"
enforce = false
exclude = []
protected_prefixes = []
prefix_map = {}

[signature]                        # File signature blocks
enabled = true
auto_insert = true
auto_update_date = true
include_path = true
include_hash = false
include_license = true
separator_char = "="
section_markers.start = " --- "
section_markers.end = " --- "
comment_styles.rust = "//"
comment_styles.python = "#"
comment_styles.javascript = "//"
comment_styles.typescript = "//"

[scaffold]                         # Project scaffolding layout
always_dirs = ["src", "tests", "docs"]
always_files = []
optional = []

[git]                              # Git integration
auto_init = true
initial_branch = "main"
initial_commit = true
initial_commit_msg = "feat: initial commit"
branch_strategy = "trunk"
commit_convention = "conventional"
commit_signing = false
hooks.pre_commit = true
hooks.pre_push = true
hooks.commit_msg = true

[env]                              # Environment variables
auto_create = true
runtime_lock = true
validation.required = []
validation.warn_missing = []

[build]                            # Build configuration
generate_makefile = true
task_runner = "just"
targets = []

[daemon]                           # File watcher daemon
enabled = true
idle_timeout_s = 300
debounce_ms = 150
watch_rename = true
watch_headers = true
watch_path_sync = true
watch_env_drift = true
watch_license = true

[stack]                            # Language stack
languages = ["rust"]
indent = "4 (spaces)"
line_width = 100
comment_style = "//"

[mcp]                              # MCP protocol server
enabled = false
default_transport = "stdio"
http_port = 8080
http_host = "127.0.0.1"

[agent]                            # AI agent integration
auto_sync = true
generate_claude = true
generate_agents = true
generate_cursor = false
generate_windsurf = false
generate_mcp_json = false

[metrics]                          # Project metrics
enabled = true
auto_snapshot = true
snapshot_history = 10

[serve]                            # TUI serve mode
refresh_ms = 1000
default_pane = "status"
theme = "dark"
show_registry = true
border_style = "rounded"

[toolchain]                        # Rust toolchain settings
rust_version = ""
clippy_lints = ""
rustfmt_edition = ""
target = ""

[pkg]                              # Package metadata
version = "0.1.0"
edition = "2021"
publish = false

[license]                          # License configuration
kind = "MIT"
auto_insert = true
file_header = true

[snippets]                         # Code snippets
enabled = true

[workspace]                        # Workspace settings
shared_deps = true
shared_toolchain = true

[recipe]                           # Task recipes
recipes = []

[time]                             # Time tracking
date_format = "%Y-%m-%d"
time_format = "%H:%M:%S"
timestamp_format = "%Y-%m-%dT%H:%M:%S%.3fZ"

[prereq]                           # Prerequisite checks
auto_install = false
```
