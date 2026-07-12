# Operations

## Building

### Prerequisites

- Rust toolchain: `stable-x86_64-pc-windows-gnu`
- Windows: `$env:Path = "C:\msys64\ucrt64\bin;$env:Path"`

### Commands

```bash
# Full workspace build
cargo build --workspace

# Release binaries
cargo build --release -p lode-cli -p lode-lsp

# Individual crates
cargo build -p lode-cli
cargo build -p lode-lsp
cargo build -p lode-daemon
cargo build -p lode-mcp
cargo build -p lode-tui
```

### Alternative Toolchain (Windows)

```bash
rustup default stable-x86_64-pc-windows-msvc
```

## Testing

```bash
# Run all workspace tests
cargo test --workspace

# Per-crate tests
cargo test -p lode-core
cargo test -p lode-cli
cargo test -p lode-daemon
cargo test -p lode-mcp
cargo test -p lode-tui
cargo test -p lode-lsp
```

Current test count: 347 tests, all passing.

## Code Quality

```bash
# Lint (must pass CI)
cargo clippy --workspace -- -D warnings

# Formatting
cargo fmt --all -- --check

# Coverage
cargo install cargo-llvm-cov
cargo llvm-cov --workspace --lcov --output-path lcov.info
```

## Security Audits

```bash
# Advisory audit (requires network)
cargo audit

# License/ban/source audit
cargo deny check
```

## Release Process

Triggered by pushing a tag `v*` or via workflow_dispatch:

1. CI runs all checks on 3 platforms
2. Release workflow builds binaries for:
   - `x86_64-pc-windows-gnu` (.zip archive)
   - `x86_64-unknown-linux-gnu` (.tar.gz archive)
   - `x86_64-apple-darwin` (.tar.gz archive)
3. Archives include: `lode` + `lode-lsp` binaries
4. SHA-256 checksums generated
5. GitHub Release created with auto-generated notes

### Manual Release

```bash
lode release --bump patch    # Dry run first
lode release --bump patch    # Apply
git tag v0.2.0
git push origin v0.2.0       # Triggers release workflow
```

## Daemon

### Lifecycle

```bash
lode daemon start      # Start background daemon
lode daemon status     # Check status
lode daemon pause      # Pause file watching
lode daemon resume     # Resume file watching
lode daemon stop       # Stop daemon
lode daemon restart    # Restart daemon
lode daemon log        # View daemon log
```

### Idle Watchdog

The daemon auto-shuts down after a configurable inactivity timeout (default: 300s). Activity is defined as file system events or IPC commands.

### IPC Transport

- Linux/macOS: Unix domain socket
- Windows: TCP loopback (port derived from socket path hash)

## TUI Dashboard

```bash
lode serve
```

Keyboard navigation:
- `Tab` / `Shift+Tab`: Cycle panes
- `q`: Quit
- Number keys (1-7): Jump to pane

Panels:
1. Overview — health score ring + sub-checks
2. Metrics — sparkline trend, coverage gauge
3. Time — session table, bar chart, 4-week heatmap
4. Activity — live daemon event feed
5. Deps — audit status + outdated packages
6. Files — file tree with violation/signature styling
7. Registry — project registry table

## Troubleshooting

### Common Issues

| Issue | Likely Cause | Solution |
|---|---|---|
| `error: toolchain 'stable-x86_64-pc-windows-gnu' not installed` | Missing Rust toolchain | `rustup toolchain install stable-x86_64-pc-windows-gnu` |
| `linking with gcc failed` | MSYS2 not in PATH | `$env:Path = "C:\msys64\ucrt64\bin;$env:Path"` |
| Daemon won't start | Port conflict | Check port file at `~/.lode/daemon.sock.port` |
| MCP connection refused | Wrong transport | Use `--stdio` for stdio transport |
| LSP server not responding | Wrong content-length | Ensure `Content-Length: N\r\n\r\n` format |

### Logs

- Daemon log: `~/.lode/daemon.log`
- CI logs: GitHub Actions workflow run
- Build artifacts: `target/`

### Cleanup

```bash
lode self clean        # Clean Lode artifacts
lode self uninstall    # Remove Lode completely
cargo clean            # Clean build artifacts
```
