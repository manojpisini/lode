# LODE Documentation

**LODE** is a local Rust developer tool providing project scaffolding, convention enforcement, secret scanning, daemon automation, TUI dashboard, MCP server, and LSP integration.

## Quick Links

| Section | Description |
|---|---|
| [Architecture](architecture/index.md) | Workspace structure, crate design, data flow |
| [Guides](guides/index.md) | Getting started, development workflow, extensions |
| [Reference](reference/index.md) | CLI commands, configuration, MCP tools, LSP protocol |
| [Operations](operations/index.md) | Daemon, TUI, build, release, troubleshooting |

## Project Overview

- **Language**: Rust (edition 2021)
- **License**: MIT
- **Repository**: github.com/lode-rs/lode
- **Status**: Active development
- **Platforms**: Linux, macOS, Windows

### What LODE Provides

| Capability | Description |
|---|---|
| Scaffolding | Initialize projects, add components, sync templates |
| Convention Enforcement | File/folder naming rules, custom rule engine |
| Secret Scanning | Detect API keys, tokens, private keys in source |
| Background Daemon | File watcher with IPC, idle watchdog, signature auto-stamp |
| Terminal UI | 7-pane dashboard with widgets (heatmap, score ring, sparklines) |
| MCP Server | 38 tools, 8 resources, 3 prompts for AI agent integration |
| LSP Server | Diagnostics, completions, hover, code actions for editors |
| CLI | 60+ commands covering the full development lifecycle |

## Documentation Map

```
docs/
  index.md              -- YOU ARE HERE
  architecture/
    index.md            -- Crate architecture, data flow, security model
  guides/
    index.md            -- Getting started, development, extension guides
  reference/
    index.md            -- CLI reference, config, MCP tools, LSP protocol
  operations/
    index.md            -- Daemon, TUI, build, release, troubleshooting
```
