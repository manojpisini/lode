# Architecture Documentation

## Crate Layering

```
lode-core (library)
  ├── lode-cli     (binary — 40+ CLI commands)
  ├── lode-daemon  (binary — file watcher + IPC)
  ├── lode-mcp     (binary — MCP server)
  ├── lode-tui     (binary — terminal UI)
  └── lode-lsp     (binary — LSP server)
```

## Security Architecture

- **ValidatedRoot** (`crates/lode-core/src/fs_safety.rs`): All write paths pass through centralized path validation. Prevents traversal, symlink escapes.
- **Process struct** (`crates/lode-core/src/process.rs`): All child processes go through this struct. Validates program names, rejects shell metacharacters, path separators.
- **Secret scanning** (`crates/lode-core/src/secrets.rs`): Regex-based scanner for API keys, tokens, private keys. Integrated into capture and MCP output.
- **Redaction** (`crates/lode-core/src/redact.rs`): Output sanitization for logs, MCP responses, errors.
- **MCP validation** (`crates/lode-mcp/src/tools/mod.rs`): ToolInputValidator checks schema, types, required args on every tool call.

## Data Flow

```
User Input → CLI/MCP/LSP/TUI → Core Library → ValidatedRoot → Filesystem
                                         → Process struct → Child processes
                                         → Secrets scanner → Redacted output
```
