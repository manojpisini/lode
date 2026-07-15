# LODE Documentation

LODE is a local Rust developer tool with filesystem writes, daemon automation, child-process execution, plugins, MCP tools, and secret scanning.

## Quick Start

```bash
cargo install lode-cli
lode init my-project
lode check
```

## Documentation Sections

| Section | Description |
|---------|-------------|
| [Architecture](architecture/index.md) | System design, crate layering, security boundaries |
| [Guides](guides/index.md) | Task-oriented guides for common workflows |
| [Reference](reference/index.md) | Configuration, templates, plugins, snippets, recipes, commands |
| [Operations](operations/index.md) | Daemon, TUI, MCP server, LSP server operations |
| [User Manual](user_manual/index.md) | Complete user manual |

## Project Structure

```
crates/
  lode-core/    — Core library (config, rules, secrets, templates)
  lode-cli/     — CLI binary (40+ commands)
  lode-daemon/  — Background file watcher
  lode-mcp/     — MCP server (44 tools)
  lode-tui/     — Terminal UI (7 panes)
  lode-lsp/     — LSP server
extensions/
  vscode-lode/  — VS Code extension
  lode.nvim/    — Neovim plugin
  zed-lode/     — Zed extension
prompts/         — UPPS v5.2.0 prompt suite
```

## Key Features

- **Scaffolding:** `lode init`, `lode add`, templates, recipes, snippets
- **Validation:** `lode check`, `lode fix`, `lode scan`, `lode audit`
- **Orchestration:** `lode agent`, `lode context`, `lode plan`
- **Daemon:** Background file watching, IPC, auto-shutdown
- **MCP:** 44 tools, 9 resources, 3 prompts
- **Secrets:** Scanning, redaction, brokering
- **Template Bundles:** Capture, apply, validate, verify
