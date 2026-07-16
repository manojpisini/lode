# Secret Scanning Guide

## Overview

LODE scans for secrets (API keys, tokens, private keys) in your project files.

## Scanning

```bash
lode scan secrets
```

Scans the entire project for secrets.

### Quiet Mode

```bash
lode scan secrets --quiet
```

Only returns an exit code (0 = no secrets found, 7 = secrets found).

### Staged Files Only

```bash
lode scan secrets --staged
```

Only scans files staged in git.

### Foreign Projects

```bash
lode scan foreign
```

Scans for non-LODE projects and reports migration actions.

## What Gets Detected

- AWS access keys (`AKIA...`)
- GitHub personal access tokens (`ghp_...`)
- GitHub OAuth tokens (`gho_...`)
- Private keys (`-----BEGIN ... PRIVATE KEY-----`)
- Suspicious assignments (`password = "...", secret = "..."`)
- Generic tokens and credentials

## What Gets Skipped

- `.env` files (real env files are skipped; `.env.example` is scanned)
- Files with `example` or `changeme` in their name
- Binary files
- Files in `target/`, `.git/`, `node_modules/`, `.venv/`

## Redaction

LODE automatically redacts secrets from:
- MCP server responses
- Template bundle captures (with `--redact-secrets` flag)
- Log output
- Error messages

## Template Bundle Integration

When capturing a template bundle, secrets are detected and redacted:

```bash
lode template-bundle capture ./source ./bundle --redact-secrets
```

Use `--no-redact` to disable redaction during capture.
