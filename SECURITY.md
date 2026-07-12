# Security Policy

LODE is pre-release. Treat all project files, templates, hooks, plugin manifests, MCP inputs, and terminal output as untrusted.

## Reporting

Open a private report with:

- affected command or crate
- reproduction steps
- expected and actual impact
- whether filesystem writes, process execution, MCP, plugin, daemon, or secret handling is involved

## Security Boundaries

- Filesystem mutations must use `ValidatedRoot`.
- Child processes must use `Process`.
- Secrets must not be logged or returned through MCP/TUI/LSP without redaction.
- Mutating MCP tools require the same validation as CLI commands.
- Network access must be explicit and narrow.