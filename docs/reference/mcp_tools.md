# MCP Tools Reference

## Overview

LODE exposes 38 core tools via the Model Context Protocol (MCP), organized into 16 groups. Tools allow AI agents to interact with LODE projects.

## Transport

Start the MCP server:

```bash
lode mcp stdio              # JSON-RPC over stdin/stdout
lode mcp --list-tools       # List all available tools
lode mcp --list-resources   # List all available resources
lode mcp --list-prompts     # List all available prompts
```

Protocol version: `2024-11-05`

## Tool Groups

### Lifecycle (4 tools)

| Tool | Description | Key Arguments |
|------|-------------|---------------|
| `lode_init` | Initialize a new LODE project | `name`, `path`, `author`, `org`, `profile`, `components` |
| `lode_add` | Add a component to existing project | `path`, `name`, `component` |
| `lode_sync` | Synchronise scaffold with filesystem | `path` |
| `lode_info` | Show project info from manifest | `path` |

### Convention (3 tools)

| Tool | Description |
|------|-------------|
| `lode_check` | Check project for convention violations |
| `lode_fix` | Automatically fix convention violations |
| `lode_rename` | Rename a file/directory to match conventions |

### Signature (2 tools)

| Tool | Description |
|------|-------------|
| `lode_sign` | Compute content hash and show signature header |
| `lode_stamp` | Write a signature header into a file |

### Environment (3 tools)

| Tool | Description |
|------|-------------|
| `lode_env_check` | Check env variables for drift or missing values |
| `lode_env_add` | Add a new env variable to the .env config |
| `lode_env_sync` | Synchronise .env file with the env config |

### Git (4 tools)

| Tool | Description |
|------|-------------|
| `lode_git_branch` | Generate a conventional branch name |
| `lode_git_commit` | Stage all changes and create a conventional commit |
| `lode_git_changelog` | Generate a changelog from git log |
| `lode_git_tag` | Create a git tag for current HEAD |

### Health (2 tools)

| Tool | Description |
|------|-------------|
| `lode_audit` | Run a project health audit |
| `lode_metrics` | Show project metrics |

### Package Management (5 tools)

| Tool | Description |
|------|-------------|
| `lode_pkg_outdated` | List outdated dependencies |
| `lode_pkg_audit` | Audit dependencies for vulnerabilities |
| `lode_pkg_update` | Update dependencies |
| `lode_pkg_list` | Detect the package manager |
| `lode_pkg_clean` | Show clean command for detected package manager |

### Secrets (1 tool)

| Tool | Description |
|------|-------------|
| `lode_scan_secrets` | Scan project files for leaked secrets |

### Release (1 tool)

| Tool | Description |
|------|-------------|
| `lode_release` | Bump version and prepare a release |

### Time Tracking (2 tools)

| Tool | Description |
|------|-------------|
| `lode_time_today` | Show today's time tracking summary |
| `lode_time_report` | Show time tracking sessions |

### Registry (2 tools)

| Tool | Description |
|------|-------------|
| `lode_projects_list` | List all registered LODE projects |
| `lode_projects_health` | Show health status for registered projects |

### Agent (2 tools)

| Tool | Description |
|------|-------------|
| `lode_agent_sync` | Show agent config sync status |
| `lode_agent_plan` | Generate an execution plan for a task |

### Config (3 tools)

| Tool | Description |
|------|-------------|
| `lode_config_show` | Show the default LODE configuration |
| `lode_config_set` | Set a config value in project.toml |
| `lode_config_validate` | Validate project configuration against schema |

### Template (2 tools)

| Tool | Description |
|------|-------------|
| `lode_template_list` | List available project template paths |
| `lode_template_show` | Show details of a specific template |

### Template Bundle (6 tools)

| Tool | Description |
|------|-------------|
| `lode_template_bundle_list` | List available template bundles |
| `lode_template_bundle_show` | Show TOML manifest of a template bundle |
| `lode_template_bundle_validate` | Validate a template bundle manifest |
| `lode_template_bundle_preview` | Preview a directory capture |
| `lode_template_bundle_apply` | Apply/render a template bundle |
| `lode_template_bundle_capture` | Capture a directory as a template bundle |

### Toolchain (2 tools)

| Tool | Description |
|------|-------------|
| `lode_toolchain_status` | Show installed toolchain versions |
| `lode_toolchain_pin` | Pin a specific tool version |

## Resources

9 resources available via `resources/read`:

| URI | Description | MIME Type |
|-----|-------------|-----------|
| `lode://config` | Default LODE configuration template | TOML |
| `lode://registry` | All registered LODE projects | JSON |
| `lode://templates` | Available project template paths | JSON |
| `lode://template-bundles` | Available template bundles | JSON |
| `lode://profiles` | Available scaffold profiles | JSON |
| `lode://recipes` | Available component recipes | JSON |
| `lode://project/info` | Current project config and metadata | JSON |
| `lode://project/health` | Current project health audit | JSON |
| `lode://project/metrics` | Current project metrics | JSON |

## Prompts

3 prompts available via `prompts/get`:

| Prompt | Description | Arguments |
|--------|-------------|-----------|
| `lode-project-review` | Review a LODE project | `path` (required) |
| `lode-scaffold-plan` | Generate a scaffold plan | `path` (required), `recipe` (optional) |
| `lode-convention-check` | Check naming conventions | `path` (required) |

## CLI-Specific Tools

In addition to the 38 core tools, the following CLI-specific tools are available when running MCP through the `lode` binary:

- `lode_scan_foreign` — Analyse a non-LODE project
- `lode_profile_list` — List available profiles
- `lode_recipe_list` — List available recipes
- `lode_metrics_show` — Show latest metrics snapshot
- `lode_pkg_audit` — Package audit (local detection)
- `lode_pkg_outdated` — Package outdated check (local detection)
- `lode_pkg_update` — Package update plan (local detection)
- `lode_custom_{slug}` — Custom command macros

## Redaction

All MCP responses are automatically redacted. Secrets, tokens, and high-entropy strings are replaced with `[REDACTED]` before returning results to the client.
