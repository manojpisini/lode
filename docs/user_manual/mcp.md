# MCP Integration Guide

## Overview

LODE provides an MCP (Model Context Protocol) server with 44 tools, 9 resources, and 3 prompts for AI agent integration.

## Starting the MCP Server

```bash
lode serve
```

The MCP server runs over stdin/stdout using JSON-RPC protocol.

## MCP Tools (44 total)

### Lifecycle
- `lode_init` — Initialize a project
- `lode_add` — Add a component
- `lode_sync` — Sync project scaffold
- `lode_info` — Show project info

### Conventions
- `lode_check` — Check naming conventions
- `lode_fix` — Fix convention violations
- `lode_rename` — Rename files

### Security
- `lode_scan_secrets` — Scan for secrets

### Git
- `lode_git_branch` — Create conventional branch
- `lode_git_commit` — Create conventional commit
- `lode_git_changelog` — Generate changelog
- `lode_git_tag` — Create tag

### Environment
- `lode_env_check` — Check environment
- `lode_env_add` — Add environment variable
- `lode_env_sync` — Sync environment

### Templates
- `lode_template_list` — List templates
- `lode_template_show` — Show template
- `lode_template_bundle_list` — List bundles
- `lode_template_bundle_show` — Show bundle manifest
- `lode_template_bundle_validate` — Validate bundle
- `lode_template_bundle_preview` — Preview capture
- `lode_template_bundle_apply` — Apply bundle
- `lode_template_bundle_capture` — Capture bundle

### Package Management
- `lode_pkg_outdated` — Check outdated deps
- `lode_pkg_audit` — Audit dependencies
- `lode_pkg_update` — Update dependencies
- `lode_pkg_list` — List packages
- `lode_pkg_clean` — Clean packages

### Release
- `lode_release` — Create release

### Time
- `lode_time_today` — Today's time
- `lode_time_report` — Time report

### Registry
- `lode_projects_list` — List projects
- `lode_projects_health` — Project health

### Agent
- `lode_agent_sync` — Sync agent config
- `lode_agent_plan` — Show agent plan

### Config
- `lode_config_show` — Show config
- `lode_config_set` — Set config value
- `lode_config_validate` — Validate config

### Toolchain
- `lode_toolchain_status` — Check toolchain
- `lode_toolchain_pin` — Pin toolchain version

## MCP Resources (9)

| URI | Description |
|-----|-------------|
| `lode://config` | LODE configuration |
| `lode://registry` | Project registry |
| `lode://templates` | Available templates |
| `lode://template-bundles` | Available template bundles |
| `lode://profiles` | Scaffold profiles |
| `lode://recipes` | Available recipes |
| `lode://project/info` | Current project info |
| `lode://project/health` | Project health audit |
| `lode://project/metrics` | Project metrics |

## MCP Prompts (3)

| Name | Description |
|------|-------------|
| `lode-project-review` | Review a LODE project |
| `lode-scaffold-plan` | Generate scaffold plan |
| `lode-convention-check` | Check naming conventions |
