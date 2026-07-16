# Configuration Reference

## Global Configuration

Location: `~/.lode/config.toml`

```toml
[identity]
name = "your-name"
email = "your-email@example.com"

[convention]
default_case = "snake_case"

[features]
auto_register = true

[git]
sign_commits = false
remote = "origin"

[install]
global_dir = "~/.local/share/lode"
```

## Project Configuration

Location: `.lode/project.toml`

```toml
schema_version = "1.0"

[project]
name = "my-project"
version = "0.1.0"
description = "My project description"

[project.language]
primary = "rust"
version = "2021"

[features]
enabled = ["core", "rust"]

[git]
init = true

[scaffold]
template = "default"
```

## Config Commands

| Command | Description |
|---------|-------------|
| `lode config show` | Show current configuration |
| `lode config show --section identity` | Show specific section |
| `lode config show --project` | Show project config |
| `lode config set key value` | Set a config value |
| `lode config reset key` | Reset to default |
| `lode config validate` | Validate configuration |

## Environment Configuration

LODE reads `.env` files for environment variables:

```
DATABASE_URL=postgres://localhost/mydb
API_KEY=sk-...
```

Use `lode env add` to manage environment entries:

```
lode env add DATABASE_URL --value "postgres://..."
lode env check
lode env sync
```
