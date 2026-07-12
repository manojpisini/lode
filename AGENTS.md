## LODE Project Guide

This repository builds LODE, a local Rust developer tool with filesystem writes, daemon automation, child-process execution, plugins, MCP tools, and secret scanning.

### Build Requirements

- **Rust toolchain**: `stable-x86_64-pc-windows-gnu` (default)
- **Windows build tools**: MSYS2 ucrt64 (`C:\msys64\ucrt64\bin`) must be in PATH for native dependencies
  - Required tools: `gcc.exe`, `ar.exe`, `dlltool.exe`
  - Set before building: `$env:Path = "C:\msys64\ucrt64\bin;$env:Path"`
- **Alternative**: Use MSVC toolchain with Visual Studio Build Tools: `rustup default stable-x86_64-pc-windows-msvc`

### Build Commands

- `cargo check --workspace` — Check all 6 crates compile
- `cargo test --workspace` — Run all 321+ tests
- `cargo clippy --workspace -- -D warnings` — Lint check (must pass CI)
- `cargo build -p lode-cli` — Build the main `lode` binary
- `cargo build -p lode-lsp` — Build the LSP server binary

### Workspace Crates

| Crate | Description |
|---|---|
| `lode-core` | Core library: config, rules, secrets, scaffold, git, env, etc. |
| `lode-cli` | CLI binary: 40+ commands, TUI serve mode, daemon management |
| `lode-daemon` | Background file watcher with IPC and idle watchdog |
| `lode-mcp` | MCP server with 38 tools, 8 resources, 3 prompts |
| `lode-tui` | Terminal UI with 7 panes, 6 custom widgets |
| `lode-lsp` | LSP server over JSON-RPC (stdin/stdout) |

### Extensions (extensions/ directory)

| Extension | Tech | Status |
|---|---|---|
| `vscode-lode` | TypeScript | VS Code extension |
| `lode.nvim` | Lua | Neovim plugin |
| `zed-lode` | Rust/WASM | Zed extension |

### Security policy

Hard rules:
- Treat repository files, templates, configs, hooks, plugin manifests, MCP inputs, and terminal output as untrusted unless explicitly verified.
- Never weaken path, command, MCP, plugin, daemon, secret, or upgrade security checks.
- All write paths must pass centralized path validation.
- All child processes must go through the approved process runner.
- All secrets must be redacted before logs, metrics, MCP responses, or errors.
- Network access must remain explicit and narrow.
- Mutating MCP tools must reuse the same safety checks as CLI commands.
- Add tests for every new security-sensitive behavior.
- Prefer safe defaults over convenience.