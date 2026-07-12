# lode-lsp

[![crates.io](https://img.shields.io/crates/v/lode-lsp.svg)](https://crates.io/crates/lode-lsp)
[![docs.rs](https://img.shields.io/docsrs/lode-lsp)](https://docs.rs/lode-lsp/latest/lode_lsp/)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

LSP (Language Server Protocol) server for [LODE](https://github.com/manojpisini/lode).  
Provides diagnostics over JSON-RPC via stdin/stdout.

## Installation

```toml
[dependencies]
lode-lsp = "0.1"
```

Or build from source:

```bash
git clone https://github.com/manojpisini/lode.git
cd lode
cargo build -p lode-lsp
```

## Features

- **Secret scanning diagnostics** — detects AWS keys, GitHub tokens, private keys, and other secrets in open files
- **Filename convention checking** — validates files follow the configured naming convention (snake_case, camelCase, etc.)
- **JSON-RPC over stdio** — standard LSP transport protocol
- **Initialization options** — configurable diagnostic toggles and workspace settings
- **Redaction support** — optional redaction of sensitive content in diagnostic messages

## Protocol

Implements a subset of the LSP specification:

- `initialize` / `initialized` — server capability negotiation
- `textDocument/didOpen` — register file for diagnostics
- `textDocument/didChange` — update diagnostics on edit
- `textDocument/didClose` — stop tracking file
- `shutdown` / `exit` — clean server shutdown

### Capabilities

```json
{
  "textDocumentSync": 1,
  "diagnosticsProvider": true
}
```

## Editor integration

### VS Code

Use the [vscode-lode](https://github.com/manojpisini/lode/tree/main/extensions/vscode-lode) extension.

### Neovim

Use the [lode.nvim](https://github.com/manojpisini/lode/tree/main/extensions/lode.nvim) plugin.

### Zed

Use the [zed-lode](https://github.com/manojpisini/lode/tree/main/extensions/zed-lode) extension.

## Related crates

- [lode-core](https://crates.io/crates/lode-core) — Core library
- [lode-cli](https://crates.io/crates/lode-cli) — CLI binary with LSP server command

## License

MIT
