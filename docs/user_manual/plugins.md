# Plugin Management Guide

## Overview

Plugins extend LODE with custom behavior through hooks and permissions.

## Adding a Plugin

```bash
lode plugin add ./path/to/plugin
```

The plugin directory must contain a manifest file (`lode.toml` or `.loderc`).

## Viewing Plugin Info

```bash
lode plugin info my-plugin
```

## Removing a Plugin

```bash
lode plugin remove my-plugin
```

## Plugin Permissions

When adding a plugin, LODE validates its declared permissions:

| Permission | What It Allows |
|------------|---------------|
| `execute` | Run shell scripts/hooks |
| `fs_write` | Write files to declared paths |
| `network` | Access network resources |
| `read` | Read files in scope |

If a plugin requests unsafe permissions, LODE will warn and require confirmation.

## Hook Execution

Plugins can declare hooks that run at specific lifecycle points:

- `pre_check` — Before convention checking
- `post_init` — After project initialization
- `pre_sync` — Before scaffold synchronization
- `post_sync` — After scaffold synchronization

Hooks run with the plugin's declared permissions.
