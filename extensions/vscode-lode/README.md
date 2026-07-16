# LODE — VS Code Extension

VS Code extension for [LODE](https://github.com/manojpisini/lode), the Local Opinionated Development Environment.

## Features

- **Check conventions** — `LODE: Check Conventions` command
- **Scan secrets** — `LODE: Scan Secrets` command
- **Initialize projects** — `LODE: Initialize Project` command
- **Sync templates** — `LODE: Sync Templates` command
- **Status display** — `LODE: Show Status` command
- **Diagnostics** — Convention violation highlighting in `.lode/config.toml`
- **Decorations** — Inline highlights for convention violations

## Requirements

- [LODE CLI](https://crates.io/crates/lode-cli) installed and available in PATH

## Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `lode.binaryPath` | `lode` | Path to the lode CLI binary |
| `lode.enableDiagnostics` | `true` | Enable LODE diagnostics |
| `lode.enableDecorations` | `true` | Highlight convention violations |

## Commands

- `LODE: Check Conventions` — Run `lode check` on current project
- `LODE: Scan Secrets` — Run `lode scan secrets`
- `LODE: Initialize Project` — Run `lode init`
- `LODE: Sync Templates` — Run `lode sync`
- `LODE: Show Status` — Show LODE status
