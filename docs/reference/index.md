# Reference

## CLI Reference

The `lode` binary provides 60+ commands organized into groups.

### Global Options

```
--help       Print help
--version    Print version
```

### Core Lifecycle

| Command | Description |
|---|---|
| `lode setup` | Initialize Lode global configuration |
| `lode init [name]` | Initialize a new project (alias: `new`) |
| `lode add <component>` | Add a component to the project |
| `lode sync` | Sync scaffold, templates, agent, metrics |
| `lode info` | Show project and Lode information |

### Development

| Command | Description |
|---|---|
| `lode dev` | Run `make dev` |
| `lode build` | Run `make build` |
| `lode test` | Run `make test` |
| `lode fmt` | Run `make fmt` |
| `lode lint` | Run `make lint` |
| `lode check [--json] [--fix]` | Check naming conventions |
| `lode fix [path]` | Auto-fix naming conventions |
| `lode verify` | Run `make verify` |
| `lode clean` | Run `make clean` |
| `lode fresh` | Clean + rebuild |
| `lode ship` | Verify + prepare release |
| `lode task [target]` | Run task runner target |

### Configuration

| Command | Description |
|---|---|
| `lode config show` | Show config |
| `lode config validate` | Validate config against schema |
| `lode config diff` | Show config differences |
| `lode config set <key>=<value>` | Set config value |
| `lode config reset` | Reset config to defaults |
| `lode config edit` | Open config in editor |

### Template Library

| Command | Description |
|---|---|
| `lode template list` | List templates |
| `lode template show <name>` | Show template details |
| `lode template reset` | Reset templates to defaults |
| `lode template edit <name>` | Edit template |

### Profile Management

| Command | Description |
|---|---|
| `lode profile list` | List profiles |
| `lode profile show <name>` | Show profile |
| `lode profile use <name>` | Activate profile |
| `lode profile new <name>` | Create profile |
| `lode profile delete <name>` | Delete profile |

### Recipe Management

| Command | Description |
|---|---|
| `lode recipe list` | List recipes |
| `lode recipe show <name>` | Show recipe |
| `lode recipe apply <name>` | Apply recipe |
| `lode recipe compose <names...>` | Compose recipes |
| `lode recipe new <name>` | Create recipe |

### Snippet Management

| Command | Description |
|---|---|
| `lode snippet list` | List snippets |
| `lode snippet show <name>` | Show snippet |
| `lode snippet search <query>` | Search snippets |
| `lode snippet add <lang> <name>` | Add snippet |
| `lode snippet remove <lang> <name>` | Remove snippet |
| `lode snippet insert <name> <file>` | Insert snippet into file |
| `lode snippet export <format>` | Export snippets |
| `lode snippet edit <name>` | Edit snippet |

### Custom Commands

| Command | Description |
|---|---|
| `lode commands list` | List custom commands |
| `lode commands add <name>` | Add command macro |
| `lode commands remove <name>` | Remove command macro |
| `lode commands run <name>` | Run command macro |

### Plugins

| Command | Description |
|---|---|
| `lode plugin list` | List plugins |
| `lode plugin search <query>` | Search plugins |
| `lode plugin add <source>` | Install plugin |
| `lode plugin remove <name>` | Remove plugin |
| `lode plugin update <name>` | Update plugin |
| `lode plugin info <name>` | Plugin details |

### Server Modes

| Command | Description |
|---|---|
| `lode mcp [--http] [--port N]` | Start MCP server |
| `lode lsp [--stdio] [--capabilities]` | Start LSP server |
| `lode serve [--no-color] [--no-live]` | Start TUI dashboard |

### Security

| Command | Description |
|---|---|
| `lode scan secrets [path]` | Scan for secrets |
| `lode scan foreign [path]` | Scan for foreign projects |
| `lode sign [path]` | Compute file content hash |
| `lode stamp [path]` | Write signature header |

### Git

| Command | Description |
|---|---|
| `lode git branch <kind> <desc>` | Create conventional branch |
| `lode git commit <message>` | Stage all and commit |
| `lode git tag <tag>` | Create git tag |
| `lode git changelog` | Generate changelog |
| `lode git hooks install` | Install git hooks |
| `lode git hooks uninstall` | Uninstall git hooks |
| `lode hooks list` | List hooks |
| `lode hooks status` | Check hook status |
| `lode hooks test` | Test hooks |

### Environment

| Command | Description |
|---|---|
| `lode env check` | Check environment drift |
| `lode env add <key> <value>` | Add environment variable |
| `lode env sync` | Synchronize .env file |
| `lode env use <profile>` | Use environment profile |

### License

| Command | Description |
|---|---|
| `lode license list` | List licenses |
| `lode license show <id>` | Show license text |
| `lode license add <id>` | Add license to project |
| `lode license remove <id>` | Remove license |
| `lode license check` | Check header consistency |
| `lode license apply` | Apply license headers |

### Projects

| Command | Description |
|---|---|
| `lode projects list` | List registered projects |
| `lode projects cd <name>` | Navigate to project |
| `lode projects register <name> <path>` | Register project |
| `lode projects remove <name>` | Remove project |
| `lode projects health` | Check all projects |
| `lode projects prune` | Remove unreachable projects |

### Toolchain

| Command | Description |
|---|---|
| `lode toolchain list` | List toolchain runtimes |
| `lode toolchain status` | Show installed versions |
| `lode toolchain pin <runtime> <version>` | Pin runtime version |

### Package Management

| Command | Description |
|---|---|
| `lode pkg list` | Detect package manager |
| `lode pkg outdated` | List outdated packages |
| `lode pkg update [name]` | Update packages |
| `lode pkg audit` | Vulnerability audit |
| `lode pkg why <dep>` | Why is this dep needed |

### Time Tracking

| Command | Description |
|---|---|
| `lode time today` | Today's summary |
| `lode time show` | Show all sessions |
| `lode time report` | Generate report |
| `lode time clear` | Clear history |

### Workspace

| Command | Description |
|---|---|
| `lode workspace init` | Initialize workspace |
| `lode workspace list` | List workspace members |
| `lode workspace add <path>` | Add workspace member |
| `lode workspace remove <name>` | Remove workspace member |
| `lode workspace run <cmd>` | Run command across members |
| `lode workspace graph` | Show dependency graph |

### Daemon

| Command | Description |
|---|---|
| `lode daemon start` | Start daemon |
| `lode daemon stop` | Stop daemon |
| `lode daemon restart` | Restart daemon |
| `lode daemon pause` | Pause daemon |
| `lode daemon resume` | Resume daemon |
| `lode daemon status` | Daemon status |
| `lode daemon log` | Show daemon log |

### Other

| Command | Description |
|---|---|
| `lode health` | Project health audit (alias: `audit`) |
| `lode doctor` | System diagnostics |
| `lode rename <path> [to]` | Rename file/directory |
| `lode release [--bump TYPE]` | Bump version and prepare release |
| `lode rules list` | List custom rules |
| `lode rules check` | Check rules against project |
| `lode rules validate` | Validate rule definitions |
| `lode rename <path>` | Rename file/directory |
| `lode export [out]` | Export LodePack |
| `lode import <path>` | Import LodePack |
| `lode self info` | Show self info |
| `lode self clean` | Clean self artifacts |
| `lode self uninstall` | Uninstall Lode |
| `lode upgrade [--check]` | Self-upgrade |
| `lode completions <shell>` | Generate shell completions |
| `lode explain` | Lode overview help |

## Configuration Reference

Configuration uses TOML. Merge chain: defaults < global config < project config.

### Global Config (`~/.lode/config.toml`)

```toml
schema_version = 3

[agent]
auto_sync = true
generate_claude = true

[build]
command = "cargo build"

[convention]
case = "snake_case"

[daemon]
enabled = true
idle_timeout_s = 300

[git]
initial_branch = "main"
signing = false

[license]
default_license = "MIT"
enforce_headers = true

[mcp]
enabled = true

[release]
tag_prefix = "v"
update_changelog = true
conventional_bump = true

[scaffold]
template_source = "embedded"

[time]
enabled = true
idle_threshold_s = 300
```

## MCP Tools Reference

All 38 tools available via `lode mcp`:

| Tool | Description |
|---|---|
| `lode_init` | Initialize a new LODE project |
| `lode_add` | Add component to existing project |
| `lode_sync` | Synchronize scaffold state |
| `lode_info` | Show project info from manifest |
| `lode_check` | Check convention violations |
| `lode_fix` | Auto-fix convention violations |
| `lode_rename` | Rename file/directory to match conventions |
| `lode_sign` | Compute content hash / show signature |
| `lode_stamp` | Write signature header into file |
| `lode_env_check` | Check env drift/missing values |
| `lode_env_add` | Add env variable to config |
| `lode_env_sync` | Synchronize .env file |
| `lode_git_branch` | Generate conventional branch name |
| `lode_git_commit` | Stage all and create conventional commit |
| `lode_git_changelog` | Generate changelog from git log |
| `lode_git_tag` | Create git tag |
| `lode_audit` | Run project health audit |
| `lode_metrics` | Show project metrics |
| `lode_pkg_outdated` | List outdated dependencies |
| `lode_pkg_audit` | Audit dependencies for vulnerabilities |
| `lode_pkg_update` | Update dependencies |
| `lode_pkg_list` | Detect package manager |
| `lode_pkg_clean` | Show clean command |
| `lode_scan_secrets` | Scan for leaked secrets |
| `lode_release` | Bump version and prepare release |
| `lode_time_today` | Today's time tracking summary |
| `lode_time_report` | Show time tracking sessions |
| `lode_projects_list` | List all registered projects |
| `lode_projects_health` | Health status of all registered projects |
| `lode_agent_sync` | Agent config sync status |
| `lode_agent_plan` | Generate execution plan |
| `lode_config_show` | Show default LODE config |
| `lode_config_set` | Set config value via dot notation |
| `lode_config_validate` | Validate project config against schema |
| `lode_template_list` | List available templates |
| `lode_template_show` | Show template details |
| `lode_toolchain_status` | Show installed toolchain versions |
| `lode_toolchain_pin` | Pin specific tool version |

### MCP Resources

| URI | Description |
|---|---|
| `lode://config` | LODE Configuration (TOML) |
| `lode://registry` | Project Registry (JSON) |
| `lode://templates` | Templates (JSON) |
| `lode://profiles` | Profiles (JSON) |
| `lode://recipes` | Recipes (JSON) |
| `lode://project/info` | Project Info (JSON) |
| `lode://project/health` | Project Health (JSON) |
| `lode://project/metrics` | Project Metrics (JSON) |

### MCP Prompts

| Prompt | Description |
|---|---|
| `lode-project-review` | Review a LODE project: config, conventions, health, suggestions |
| `lode-scaffold-plan` | Generate scaffold plan for new/existing project |
| `lode-convention-check` | Check naming conventions and suggest fixes |

## LSP Protocol

The LSP server (`lode-lsp`) communicates over STDIN/STDOUT using JSON-RPC 2.0 with Content-Length headers.

**Capabilities**:
- `textDocumentSync`: openClose + incremental changes + save with includeText
- `completionProvider`: Trigger characters `.` and `/`
- `codeActionProvider`: true
- `documentSymbolProvider`: true
- `hoverProvider`: true

**Diagnostics**:
- Secret scanning via `lode_core::scan_content()` — detects API keys, tokens, private keys
- Filename convention checking via `lode_core::normalize_name()`
- Diagnostics pushed on document open, change, and save

**Code Actions**:
- Add secrets to `.env.example`
- Fix naming convention
- Rename file to match convention
