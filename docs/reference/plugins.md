# Plugin Reference

## Plugin Manifest

Plugins are configured via `lode.toml` or `.loderc` in the plugin directory.

```toml
[plugin]
name = "my-plugin"
version = "0.1.0"

[permissions]
execute = true
fs_write = ["docs/", "src/generated/"]
network = false

[hooks]
pre_check = "scripts/pre_check.sh"
post_init = "scripts/post_init.sh"
```

## Plugin Commands

| Command | Description |
|---------|-------------|
| `lode plugin add <path>` | Add a plugin from path |
| `lode plugin info <name>` | Show plugin details |
| `lode plugin remove <name>` | Remove a plugin |

## Permissions

| Permission | Description |
|------------|-------------|
| `execute` | Allow running scripts/hooks |
| `fs_write` | Allow file writes in specific paths |
| `network` | Allow network access |
| `read` | Allow reading files |

## Hook Points

| Hook | When Triggered |
|------|---------------|
| `pre_check` | Before convention check |
| `post_init` | After project initialization |
| `pre_sync` | Before scaffold sync |
| `post_sync` | After scaffold sync |
