# lode-core

[![crates.io](https://img.shields.io/crates/v/lode-core.svg)](https://crates.io/crates/lode-core)
[![docs.rs](https://img.shields.io/docsrs/lode-core)](https://docs.rs/lode-core/latest/lode_core/)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Core library for [LODE](https://github.com/manojpisini/lode) — the all-in-one local developer tool.

## Installation

```toml
[dependencies]
lode-core = "0.1"
```

Or build from source:

```bash
git clone https://github.com/manojpisini/lode.git
cd lode
cargo build -p lode-core
```

## Features

- **Configuration** — layered TOML config (global + project), schema validation, migrations
- **Convention rules** — filename/directory casing checks, custom regex rules
- **Secret scanning** — detect AWS keys, GitHub tokens, private keys, and suspicious assignments
- **Secret redaction** — centralized `redact()` function to mask secrets in logs, errors, and output
- **Scaffolding** — project initialization from templates with `init` and `sync`
- **Git integration** — conventional commits, branch naming, changelog generation, hook management
- **Environment variables** — `.env` lockfile management with drift detection
- **Template engine** — extensible template rendering with blocks, includes, filters, and conditionals
- **Filesystem safety** — validated path resolution, symlink protection, atomic writes, traversal prevention
- **Process runner** — approved process execution with shell metacharacter validation
- **Project registry** — track and manage multiple lode projects
- **Plugin system** — permission-based plugin installation with `PluginSecurity` and `PluginInstallReceipt`
- **Release management** — version bumping, dry-run, rollback
- **Time tracking** — session-based time logging with reporting
- **Workspace support** — multi-crate workspace discovery and management
- **Agent integration** — AI agent plan/task management and context file sync
- **License management** — embedded license templates and header application
- **Audit scoring** — project health scoring system

## Related crates

- [lode-cli](https://crates.io/crates/lode-cli) — CLI binary
- [lode-daemon](https://crates.io/crates/lode-daemon) — File watcher daemon
- [lode-mcp](https://crates.io/crates/lode-mcp) — MCP protocol server
- [lode-tui](https://crates.io/crates/lode-tui) — Terminal UI
- [lode-lsp](https://crates.io/crates/lode-lsp) — LSP server

## License

MIT
