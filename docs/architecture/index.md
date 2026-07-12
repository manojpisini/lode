# Architecture

## Workspace Structure

```
lode/
  Cargo.toml                # Workspace root (resolver v2)
  crates/
    lode-core/              # Shared library foundation
    lode-cli/               # CLI binary (lode)
    lode-daemon/            # Background daemon (binary + library)
    lode-mcp/               # MCP protocol server
    lode-tui/               # Terminal UI dashboard
    lode-lsp/               # LSP protocol server
  extensions/
    vscode-lode/            # VS Code extension (TypeScript)
    lode.nvim/              # Neovim plugin (Lua)
    zed-lode/               # Zed extension (Rust/WASM)
  fuzz/                     # Fuzz targets
```

All crates depend on `lode-core`. No compile-time cross-dependency between siblings.

## Crate Architecture

### lode-core (9,072 lines, 56 source files)

The shared foundation. Every module is a self-contained capability:

| Module | Purpose |
|---|---|
| `fs_safety` | ValidatedRoot — central path validation, traversal prevention, symlink escape detection |
| `process` | Validated child process runner — shell metacharacter rejection |
| `secrets` | Secret scanning — API keys, tokens, private keys |
| `config` | Unified LodeConfig with 22 sub-configs, schema validation, merge chain |
| `scaffold` | Project init, add component, sync with lock, template rendering |
| `template` | Template engine with variables, filters, conditionals, extends |
| `git` | Git integration — init, commit, tag, hooks, conventional commits |
| `release` | Version detection, semantic bump, file updates |
| `audit` | Project health scoring |
| `env` | Environment file generation, validation, drift detection |
| `convention` | File naming convention checking and fixing |
| `rules` | Custom regex-based convention rules |
| `install` | Global workspace setup and config management |
| `agent` | AI agent context sync |
| `recipe` | Capability bundles with template files and steps |
| `snippet` | Code snippet management |
| `hooks` | Plugin-like hook system |
| `registry` | Project registry CRUD |
| `pkg` | Package manager detection (cargo, npm, pip, etc.) |
| `license` | License text management and header consistency |
| `signature` | File signature headers |
| `template_sync` | Stale detection and reconciliation |
| `toolchain` | Runtime toolchain version management |
| `workspace` | Multi-crate workspace discovery |
| `time_tracker` | Session-based time tracking |
| `test_history` | Test run persistence |
| `commands` | LodePack import/export |
| `task` | Task runner detection |
| `prereq` | Prerequisite tool checking |
| `ipc` | Socket port hashing for daemon IPC |
| `error` | Unified error types and exit codes |
| `assets` | Embedded static assets |

### lode-cli (10,208 lines, 46 source files)

The main `lode` binary. Built with `clap` derive for argument parsing.

- Entry point: `src/main.rs` — `fn run()` parses `Cli` and dispatches to command handlers
- Command types: `src/cmd/types.rs` — all 60+ command variants, 23 subcommand enums
- Command handlers: `src/cmd/*.rs` — 43 module files, each handling one command group

### lode-daemon (1,242 lines, 7 source files)

Background file watcher with platform-adaptive IPC.

- **IPC transport**: Unix domain socket (Linux/macOS), TCP loopback fallback (Windows)
- **Protocol**: Line-delimited JSON over stream
- **Commands**: Status, Stop, Pause, Resume, ListWatchers, Reload
- **Events**: Create, Modify, Rename, Delete
- **Idle watchdog**: Auto-shutdown after configurable inactivity timeout
- **Handlers**: Signature stamps on create/modify, signature verification

### lode-mcp (2,721 lines, 24 source files)

Model Context Protocol server — 38 tools, 8 resources, 3 prompts.

- **Transport**: STDIO (line-delimited JSON-RPC), HTTP planned
- **Max message size**: 1 MB
- **Tool categories**: Lifecycle, Conventions, Signatures, Environment, Git, Health, Package, Secrets, Release, Time, Registry, Agent, Config, Template, Toolchain

### lode-tui (1,847 lines, 19 source files)

Terminal UI dashboard with `ratatui` and `crossterm`.

- **7 panes**: Overview, Metrics, Time, Activity, Deps, Files, Registry
- **6 custom widgets**: Heatmap, BarChart, ScoreRing, StatusBar, Sparkline
- **Daemon IPC**: Live event feed from daemon

### lode-lsp (627 lines, 2 source files)

Language Server Protocol implementation.

- **Transport**: STDIN/STDOUT with Content-Length headers (JSON-RPC 2.0)
- **Capabilities**: Incremental text sync, completions, hover, document symbols, code actions
- **Diagnostics**: Secret scanning + filename convention violations pushed to editor

## Security Model

Three-layer defense enforced at the core:

```
┌─────────────────────────────────────────┐
│              lode-cli, lode-mcp,         │
│           lode-daemon, lode-tui,         │
│                lode-lsp                  │
├─────────────────────────────────────────┤
│                                         │
│  ┌──────────────┐  ┌───────────────┐   │
│  │ ValidatedRoot│  │   Process     │   │
│  │ (fs_safety)  │  │  (process)    │   │
│  │              │  │               │   │
│  │ - traversal  │  │ - metachar    │   │
│  │   prevention │  │   rejection   │   │
│  │ - symlink    │  │ - path sep    │   │
│  │   escape     │  │   rejection   │   │
│  │   detection  │  │ - null byte   │   │
│  │ - atomic     │  │   rejection   │   │
│  │   writes     │  │               │   │
│  └──────┬───────┘  └──────┬────────┘   │
│         │                 │            │
│  ┌──────┴─────────────────┴────────┐   │
│  │         secrets.rs              │   │
│  │  - API key detection            │   │
│  │  - Token scanning               │   │
│  │  - Private key detection        │   │
│  │  - .env file allowlisting       │   │
│  └─────────────────────────────────┘   │
│                                         │
│  + Template recursion bound (16)        │
│  + Recipe destination validation        │
│  + Scaffold traversal rejection         │
│  + Git hook ownership tracking           │
│  + ExitCode::VulnOrSecret (7)           │
│                                         │
└─────────────────────────────────────────┘
```

**Rules**:
- All filesystem writes pass through `ValidatedRoot`
- All child processes pass through `Process`
- All secrets redacted from logs, errors, and MCP responses
- Network access must be explicit and narrow
- Mutating MCP tools reuse same safety checks as CLI

## Data Flow

```
User (Terminal)          AI Agent (MCP Client)     Editor (LSP Client)
       │                         │                       │
       │ stdio                   │ stdio                  │ stdio
       ▼                         ▼                       ▼
   lode-cli                  lode-mcp                 lode-lsp
       │                         │                       │
       └──────────┬──────────────┴───────────┬───────────┘
                  │                          │
                  ▼                          ▼
            lode-core                   lode-daemon
                                          │
                                     ┌────┴────┐
                                     │  File   │
                                     │  System │
                                     └─────────┘

lode-tui ←→ lode-daemon (IPC socket, live events)
```

## IPC Protocol (Daemon)

- **Connection**: Unix socket or TCP (platform adaptive)
- **Format**: Line-delimited JSON
- **Client sends**: `{"command": "Status" | "Stop" | "Pause" | "Resume" | "ListWatchers" | "Reload"}`
- **Server responds**: `{"success": bool, "message": string, "data": ...}`
- **Port discovery**: Deterministic from socket path via SHA-256, with `.port` sidecar file

## MCP Protocol

- **Transport**: STDIO, line-delimited JSON-RPC 2.0
- **Tools**: 38 tools registered via `tools/list`, dispatched by `tools/call`
- **Resources**: 8 resources served via `resources/list` and `resources/read`
- **Prompts**: 3 prompts served via `prompts/list` and `prompts/get`
- **Max input**: 1 MB

## LSP Protocol

- **Transport**: STDIN/STDOUT with `Content-Length` headers
- **Initialize response** declares: text sync (incremental), completions, hover, document symbols, code actions
- **Diagnostics** pushed for: secrets found (scan_content), convention violations (normalize_name)
- **Max content length**: 4 MB
