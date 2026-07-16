# LSP Reference

## Overview

LODE provides a Language Server Protocol (LSP) implementation that delivers editor diagnostics for secrets, file signatures, and filename conventions. It can be run as a standalone binary or inline through the `lode` CLI.

## Running the LSP Server

```bash
# Standalone (after cargo install lode-lsp)
lode-lsp

# Inline via lode CLI
lode lsp stdio
```

## Protocol

JSON-RPC 2.0 over stdin/stdout with `Content-Length` headers. Max message size: 4 MB.

## Capabilities

| Capability | Details |
|------------|---------|
| Text sync | `didOpen`, `didChange`, `didSave` |
| Diagnostics | Secrets, signature headers, filename conventions |
| Completion | Rust keywords + TOML snippets |
| Code actions | Quickfix for secrets and convention violations |
| Document symbols | Rust symbols (fn, struct, enum, trait, impl, mod) |
| Hover | Identifier type information |

## Diagnostics

### Secret Detection

Scans files for credentials on open, change, and save:

- GitHub tokens (`ghp_`, `github_pat_`)
- AWS access keys (`AKIA`, `ASIA`, etc.)
- Private keys (`-----BEGIN...PRIVATE KEY-----`)
- Credential assignments (`api_key`, `secret`, `password`, `token`)

Uses `lode_core::scan_content()` for in-memory scanning with per-URI caching to avoid redundant scans.

### Filename Convention Check

Checks filenames against configured naming convention (default: `snake_case`).

Uses `lode_core::normalize_name()`.

### Signature Header Check

Verifies that supported file types have `@file` and `@project` markers in the first 20 lines.

Supported extensions: `.rs`, `.ts`, `.js`, `.py`, `.go`, `.java`, `.c`, `.cpp`, `.h`, `.hpp`

## Configuration

Optional `initializationOptions`:

```json
{
  "redactDiagnostics": true
}
```

When `redactDiagnostics` is enabled, diagnostic messages are redacted through `lode_core::redact()`.

## Editor Integration

### VS Code

The `vscode-lode` extension connects to the LSP server. Configure in VS Code settings:

```json
{
  "lode.lsp.path": "lode-lsp"
}
```

### Neovim

Configure via `lspconfig`:

```lua
require('lspconfig').lode.setup {
  cmd = { "lode-lsp" },
}
```

### Zed

The `zed-lode` extension includes LSP integration. Configure in Zed settings.

## Methods

| Method | Direction | Description |
|--------|-----------|-------------|
| `initialize` | Client → Server | Returns server info and capabilities |
| `initialized` | Client → Server | No-op |
| `shutdown` | Client → Server | Returns null |
| `exit` | Client → Server | Exits process |
| `textDocument/didOpen` | Client → Server | Stores document, publishes diagnostics |
| `textDocument/didChange` | Client → Server | Updates document, publishes diagnostics |
| `textDocument/didSave` | Client → Server | Re-publishes diagnostics |
| `textDocument/documentSymbol` | Client → Server | Returns Rust symbols |
| `textDocument/completion` | Client → Server | Returns completions |
| `textDocument/hover` | Client → Server | Returns hover info |
| `textDocument/codeAction` | Client → Server | Returns quickfix actions |
| `textDocument/publishDiagnostics` | Server → Client | Sends diagnostic results |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Normal shutdown |
| 1 | Error during processing |
