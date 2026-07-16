# Installation

## From crates.io

```bash
cargo install lode-cli
cargo install lode-lsp
```

## From Source

```bash
git clone https://github.com/manojpisini/lode
cd lode
cargo build --release -p lode-cli
cargo build --release -p lode-lsp
```

The binaries will be at `target/release/lode.exe` and `target/release/lode-lsp.exe`.

## Requirements

- **Rust toolchain:** `stable-x86_64-pc-windows-gnu` (default on Windows)
- **Windows build tools:** MSYS2 ucrt64 (`C:\msys64\ucrt64\bin`) in PATH
- **Alternative:** MSVC toolchain with Visual Studio Build Tools

## Setup

After installation, run setup to create the default configuration:

```bash
lode setup
```

This creates:
- `~/.lode/config.toml` — Global configuration
- `~/.local/share/lode/` — Global assets directory
