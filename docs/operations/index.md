# Operations

## Daemon

The LODE daemon runs in the background to watch files, handle IPC, and auto-shutdown on idle.

```
lode daemon start
lode daemon status
lode daemon stop
lode daemon log --tail
```

See [Daemon Guide](../user_manual/daemon.md) for full details.

## MCP Server

The MCP server provides 44 tools, 9 resources, and 3 prompts over JSON-RPC (stdin/stdout).

```
lode serve
```

See [MCP Integration](../user_manual/mcp.md).

## TUI Dashboard

The terminal UI provides 7 panes for project monitoring.

```
lode serve --tui
```

## LSP Server

The LSP server provides diagnostics for filename conventions and secret scanning.

```
lode-lsp
```

## Health Checks

```
lode doctor     — System health check
lode audit      — Project audit
lode metrics    — Project metrics
```

## CI/CD

GitHub Actions workflow runs `cargo check`, `cargo test`, `cargo clippy` on every push.
