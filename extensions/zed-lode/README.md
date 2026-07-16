# LODE — Zed Extension

Zed editor extension for [LODE](https://github.com/manojpisini/lode), the Local Opinionated Development Environment.

## Features

- Language server protocol integration with `lode-lsp`
- Secret scanning diagnostics
- Convention checking
- File signature verification

## Installation

```bash
cargo install lode-lsp
```

Then add the extension in Zed settings.

## Requirements

- [LODE CLI](https://crates.io/crates/lode-cli) or [lode-lsp](https://crates.io/crates/lode-lsp) installed

## Building

```bash
cargo build --manifest-path extensions/zed-lode/Cargo.toml --release
```

The WASM binary will be at `target/wasm32-wasi/release/zed-lode.wasm`.
