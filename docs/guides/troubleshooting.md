# Troubleshooting Guide

## Common Issues

### `lode` command not found

After installing via `cargo install lode-cli`, ensure `~/.cargo/bin` is in your PATH.

### `lode scan` fails

Use `lode scan secrets` instead. The `scan` command has subcommands: `secrets` and `foreign`.

### Config not found

LODE looks for configuration at `~/.lode/config.toml`. Run `lode setup` to create it, or set `$LODE_CONFIG` to point to a custom location.

### Daemon won't start

Check if another instance is running:

```bash
lode daemon status
lode doctor
```

### Secret scanning false positives

LODE scans for credential patterns. To exclude a file, add it to `.gitignore` or use allowlisted paths like `.env.example`.

### Build fails on Windows

Ensure MSYS2 ucrt64 is in PATH, or switch to the MSVC toolchain:

```bash
rustup default stable-x86_64-pc-windows-msvc
```

### Tests fail

Run with verbose output:

```bash
cargo test --workspace -- --nocapture
```

### Plugin installation fails

Plugins require a manifest with permissions. Use `lode plugin add <source>` with `--allow-unsafe` if needed.

### Getting Help

```bash
lode doctor --fix       # Auto-diagnose and fix
lode explain <topic>    # Explain a concept
lode --help             # Show CLI help
```
