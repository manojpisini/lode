# lode-mcp

[![crates.io](https://img.shields.io/crates/v/lode-mcp.svg)](https://crates.io/crates/lode-mcp)
[![docs.rs](https://img.shields.io/docsrs/lode-mcp)](https://docs.rs/lode-mcp/latest/lode_mcp/)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

MCP (Model Context Protocol) server for [LODE](https://github.com/manojpisini/lode).  
Exposes 38+ tools, 8 resources, and 3 prompts over stdio or HTTP transport.

## Installation

```toml
[dependencies]
lode-mcp = "0.1"
```

Or build from source:

```bash
git clone https://github.com/manojpisini/lode.git
cd lode
cargo build -p lode-mcp
```

## Features

### Tools (38+)

| Category | Tools |
|---|---|
| Lifecycle | `initialize`, `shutdown` |
| Config | `lode_config_show`, `lode_config_set`, `lode_config_validate` |
| Project | `lode_init`, `lode_health`, `lode_info` |
| Git | `lode_git_branch`, `lode_git_commit`, `lode_git_tag`, `lode_git_changelog` |
| Conventions | `lode_check`, `lode_fix`, `lode_rename`, `lode_rules_list`, `lode_rules_validate` |
| Secrets | `lode_scan_secrets` |
| Env | `lode_env_check`, `lode_env_add` |
| Time | `lode_time_today`, `lode_time_report` |
| Release | `lode_release` |
| Packages | `lode_pkg_list`, `lode_pkg_outdated`, `lode_pkg_update`, `lode_pkg_audit` |
| Toolchain | `lode_toolchain_list`, `lode_toolchain_add`, `lode_toolchain_remove` |
| Agent | `lode_agent_sync`, `lode_agent_plan` |

### Resources

- `config://defaults` — default configuration
- `config://current` — active configuration
- `project://health` — project health report
- `project://metrics` — project metrics
- `git://status` — git status summary
- `env://status` — environment status
- `secrets://scan` — secret scan results
- `time://today` — today's time log

### Prompts

- `analyze_project` — structured project analysis
- `review_security` — security review guidance
- `plan_release` — release planning

### Input validation

All tool inputs are validated through `ToolInputValidator` with JSON Schema validation before dispatch. Responses are redacted of secrets before returning.

## Related crates

- [lode-core](https://crates.io/crates/lode-core) — Core library
- [lode-cli](https://crates.io/crates/lode-cli) — CLI binary with MCP server command

## License

MIT
