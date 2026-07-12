# lode-cli

[![crates.io](https://img.shields.io/crates/v/lode-cli.svg)](https://crates.io/crates/lode-cli)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**LODE** — an all-in-one local developer tool for Rust projects.  
Manage projects, track time, scan secrets, enforce conventions, run daemons, serve MCP/LSP protocols, manage plugins and packages, automate git workflows, and export/import portable LodePacks.

## Installation

### From crates.io (recommended)

```bash
cargo install lode-cli
```

### From source

```bash
git clone https://github.com/manojpisini/lode.git
cd lode
cargo build -p lode-cli
./target/debug/lode --help
```

### System requirements

- Rust toolchain (`stable-x86_64-pc-windows-gnu` or MSVC on Windows)
- MSYS2 ucrt64 (`C:\msys64\ucrt64\bin` in PATH) on Windows for native deps, or use MSVC toolchain

## Quick start

```bash
# Initialize lode with defaults
lode setup --defaults

# Create a new project
lode init my-project --profile core/bare

# Enter the project and check its health
cd my-project
lode health
```

## Common workflows

```bash
# Project health & diagnostics
lode health          # Run health audit
lode doctor          # System diagnostics
lode scan secrets    # Scan for leaked secrets

# Code quality
lode check           # Convention compliance check
lode lint            # Run clippy
lode fmt             # Format code

# Git automation
lode git branch feat add-login
lode git commit "feat: add login page"
lode git tag 0.1.0 --push
lode git changelog

# Environment
lode env check
lode env add DATABASE_URL --secret

# Time tracking
lode time today
lode time report

# Packages
lode pkg list
lode pkg outdated
lode pkg audit

# Daemon
lode daemon start
lode daemon status

# TUI dashboard
lode serve

# MCP / LSP servers
lode mcp --list-tools
lode lsp --stdio

# Export / Import
lode export --out project.lodepack
lode import project.lodepack
```

## All commands

| Command | Description |
|---|---|
| `setup` | Initialize lode defaults |
| `init` / `new` | Create a new project |
| `add` | Add a component to the project |
| `sync` | Sync config/templates/agent/metrics |
| `info` | Show project information |
| `config` | Configuration management |
| `template` | Template library management |
| `profile` | Profile management |
| `recipe` | Task recipe management |
| `snippet` | Code snippet management |
| `commands` | Custom command macros |
| `plugin` | Plugin system |
| `mcp` | MCP protocol server |
| `lsp` | LSP protocol server |
| `agent` | AI agent file generation |
| `task` | Track current work task |
| `dev` / `build` / `test` / `fmt` / `lint` | Workflow commands |
| `check` / `fix` / `rename` | Convention tools |
| `rules` | Convention rules management |
| `sign` / `stamp` | File signatures & headers |
| `verify` / `clean` / `fresh` / `ship` | Project lifecycle |
| `release` | Version release management |
| `health` / `audit` | Project health audit |
| `explain` | Explain a concept |
| `doctor` | System diagnostics |
| `scan` | Scan for secrets/foreign code |
| `git` | Git workflow automation |
| `hooks` | Git hooks management |
| `env` | Environment variable management |
| `license` | License management |
| `projects` | Project registry |
| `toolchain` | Runtime toolchain management |
| `pkg` | Package management |
| `time` | Time tracking |
| `metrics` | Project metrics |
| `workspace` | Multi-crate workspace management |
| `daemon` | Background file watcher daemon |
| `log` | Log management |
| `export` / `import` | LodePack archive |
| `serve` | TUI dashboard mode |
| `self` | Self-management |
| `upgrade` | Self-upgrade |
| `completions` | Shell completions |
| `version` | Show version |

Use `lode <command> --help` for detailed options.

## Configuration

Configuration is stored at `~/.lode/config.toml` and merged with project-level `.lode/project.toml`. See [docs/](https://github.com/manojpisini/lode/tree/main/docs) for the full configuration reference.

## Related crates

- [lode-core](https://crates.io/crates/lode-core) — Core library
- [lode-daemon](https://crates.io/crates/lode-daemon) — File watcher daemon
- [lode-mcp](https://crates.io/crates/lode-mcp) — MCP protocol server
- [lode-tui](https://crates.io/crates/lode-tui) — Terminal UI
- [lode-lsp](https://crates.io/crates/lode-lsp) — LSP server

## Extensions

- [vscode-lode](https://github.com/manojpisini/lode/tree/main/extensions/vscode-lode) — VS Code extension
- [lode.nvim](https://github.com/manojpisini/lode/tree/main/extensions/lode.nvim) — Neovim plugin
- [zed-lode](https://github.com/manojpisini/lode/tree/main/extensions/zed-lode) — Zed extension (WASM)

## License

MIT
