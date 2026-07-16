# Commands Cheatsheet

## Project Lifecycle

| Command | Description |
|---------|-------------|
| `lode init <name>` | Initialize a new LODE project |
| `lode init --profile rust` | Init with Rust profile |
| `lode add <component>` | Add a component to existing project |
| `lode sync` | Sync scaffold with template updates |

## Conventions

| Command | Description |
|---------|-------------|
| `lode check` | Check naming conventions |
| `lode fix` | Fix convention violations |
| `lode rename <old> <new>` | Rename a file/directory |

## Secrets & Security

| Command | Description |
|---------|-------------|
| `lode scan secrets` | Scan for secrets |
| `lode scan secrets --quiet` | Quiet mode (exit code only) |
| `lode scan secrets --staged` | Scan staged files only |
| `lode scan foreign` | Scan for non-LODE projects |

## Templates

| Command | Description |
|---------|-------------|
| `lode template list` | List available templates |
| `lode template show <name>` | Show template details |
| `lode template-bundle apply <path>` | Apply a template bundle |
| `lode template-bundle capture <src> <dst>` | Capture directory as bundle |
| `lode template-bundle preview <source>` | Preview a capture |
| `lode template-bundle list` | List template bundles |
| `lode template-bundle show <path>` | Show bundle manifest |
| `lode template-bundle validate <path>` | Validate bundle |
| `lode template-bundle verify <path>` | Verify bundle assets |

## Configuration

| Command | Description |
|---------|-------------|
| `lode config show` | Show config |
| `lode config set <key> <value>` | Set config value |
| `lode config reset <key>` | Reset to default |
| `lode config validate` | Validate config |

## Environment

| Command | Description |
|---------|-------------|
| `lode env add <key>` | Add env variable |
| `lode env check` | Check env against .env.example |
| `lode env sync` | Sync env files |

## Git

| Command | Description |
|---------|-------------|
| `lode git branch <kind> <description>` | Create conventional branch |
| `lode git commit [message] --type <kind> --scope <scope>` | Create conventional commit |
| `lode git changelog` | Generate changelog |
| `lode git tag <version>` | Create version tag |

## Agent & Context

| Command | Description |
|---------|-------------|
| `lode agent sync` | Sync agent configuration |
| `lode agent plan` | Load/display agent plan |
| `lode agent policy` | Generate agent policy |
| `lode context compile` | Compile project context |
| `lode context compile --budget <tokens>` | With token budget |

## Plugins

| Command | Description |
|---------|-------------|
| `lode plugin add <path>` | Add a plugin |
| `lode plugin info <name>` | Show plugin info |
| `lode plugin remove <name>` | Remove a plugin |

## Snippets

| Command | Description |
|---------|-------------|
| `lode snippet add` | Add snippet |
| `lode snippet list` | List snippets |
| `lode snippet search <query>` | Search snippets |
| `lode snippet export --format vscode` | Export snippets |
| `lode snippet remove <name>` | Remove snippet |

## Recipes

| Command | Description |
|---------|-------------|
| `lode recipe new <name>` | Create recipe |
| `lode recipe list` | List recipes |
| `lode recipe apply <name>` | Apply recipe |
| `lode recipe compose <a> <b>` | Compose recipes |

## License

| Command | Description |
|---------|-------------|
| `lode license set <type>` | Set project license |
| `lode license check` | Check license presence |
| `lode license add` | Add license file |

## Rules

| Command | Description |
|---------|-------------|
| `lode rules list` | List rules |
| `lode rules check <name>` | Check a name against rules |
| `lode rules validate` | Validate rule config |

## Release

| Command | Description |
|---------|-------------|
| `lode release <version>` | Create a release |
| `lode release --dry-run` | Dry run release |
| `lode release rollback` | Rollback pending release |

## Daemon

| Command | Description |
|---------|-------------|
| `lode daemon start` | Start daemon |
| `lode daemon status` | Check daemon status |
| `lode daemon stop` | Stop daemon |
| `lode daemon log` | Show daemon log |
| `lode daemon log --tail` | Follow log |

## Health

| Command | Description |
|---------|-------------|
| `lode doctor` | System health check |
| `lode audit` | Project audit |
| `lode metrics` | Project metrics |

## Time Tracking

| Command | Description |
|---------|-------------|
| `lode time today` | Show today's time |
| `lode time show` | Show time log |
| `lode time report` | Generate time report |
| `lode time clear` | Clear time log |

## Projects

| Command | Description |
|---------|-------------|
| `lode projects list` | List registered projects |
| `lode projects health` | Show project health |
| `lode projects prune` | Remove missing projects |

## Signature

| Command | Description |
|---------|-------------|
| `lode sign` | Sign headers |
| `lode stamp` | Stamp files |

## Workspace

| Command | Description |
|---------|-------------|
| `lode workspace init <name>` | Init workspace |
| `lode workspace add <path>` | Add member |
| `lode workspace list` | List members |
| `lode workspace graph` | Show dependency graph |

## Other

| Command | Description |
|---------|-------------|
| `lode info` | Show project info |
| `lode handoff` | Generate handoff document |
| `lode export` | Export project as LodePack |
| `lode self update` | Self-update |
| `lode setup` | Setup LODE environment |
