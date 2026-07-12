# Threat Model

**Project**: lode
**Suite**: universal-project-prompt-suite v3.2.0
**Prompt**: UPPS-04
**Mode**: AUDIT_ONLY
**Generated**: 2026-07-11
**Completion**: PASS

## 1. Objective

Identify assets, trust boundaries, threat actors, abuse cases, existing controls, residual risks, and security validation priorities for the lode project.

## 2. Scope

All 6 workspace crates, 3 extensions, CI/CD pipeline, CLI/MCP/LSP/daemon/TUI interfaces, and build/release process.

## 3. Assets

### Primary Assets

| Asset | Description | Sensitivity | Location |
|---|---|---|---|
| Source code | All Rust, TypeScript, Lua source | High | Repository, working tree |
| User config | `~/.lode/config.toml`, project `.lode/` | Medium | Filesystem |
| Project data | Scaffolds, templates, snippets, recipes | Low | `~/.lode/`, project dir |
| User secrets | API keys, tokens, creds discovered by scanner | High | Source files (scanned) |
| Daemon state | IPC socket, state files | Medium | `~/.lode/` |
| Registry data | Project registry JSON | Low | `~/.lode/` |
| Time tracking | Session logs | Low | `~/.lode/` |
| Build artifacts | Compiled binaries | Medium | `target/` |
| Release archives | Published binaries, checksums | High | GitHub Releases |

### Secondary Assets

| Asset | Description | Sensitivity |
|---|---|---|
| System PATH | Executable lookup directories | High |
| Git repository | `.git/` history, hooks | Medium |
| Environment files | `.env`, `.env.*` | High (may contain secrets) |
| Plugin code | Installed plugins | Medium |

## 4. Trust Boundaries

```
┌──────────────────────────────────────────────────────────────────┐
│                     Untrusted Zone (User's System)                │
│                                                                  │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────────────┐ │
│  │ Terminal User │   │ AI Agent     │   │ Editor (VS Code,     │ │
│  │ (shell)       │   │ (MCP Client) │   │  Neovim, Zed)       │ │
│  └──────┬───────┘   └──────┬───────┘   └──────────┬───────────┘ │
│         │                  │                       │             │
├─────────┼──────────────────┼───────────────────────┼─────────────┤
│         │     Boundary A: CLI Input                │             │
│         ▼                  ▼                       ▼             │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    TRUSTED ZONE (lode)                    │   │
│  │                                                          │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │   │
│  │  │ lode-cli │ │ lode-mcp │ │ lode-lsp │ │ lode-tui │   │   │
│  │  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘   │   │
│  │       │             │            │             │         │   │
│  │       └──────┬──────┴────────────┴─────────────┘         │   │
│  │              │                                           │   │
│  │              ▼                                           │   │
│  │  ┌──────────────────┐   ┌──────────────────┐             │   │
│  │  │   lode-core      │   │  lode-daemon     │             │   │
│  │  │  (ValidatedRoot, │◄──│  (file watcher,  │             │   │
│  │  │   Process,       │   │   IPC server)    │             │   │
│  │  │   secrets scanner)│  └────────┬─────────┘             │   │
│  │  └──────────────────┘           │                        │   │
│  │                                 │                        │   │
│  └─────────────────────────────────┼────────────────────────┘   │
│                                    │                            │
│                    Boundary B: Filesystem                       │
│                                    ▼                            │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │               FILESYSTEM (Untrusted)                      │   │
│  │  source code, config, templates, plugins, .git, .env     │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│                    Boundary C: Child Processes                   │
│                                    │                            │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  git, cargo, npm, make, rustup, system tools            │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│                    Boundary D: Network (Optional)                │
│                                    │                            │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  GitHub (releases, git remote)                           │   │
│  └──────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
```

### Boundary Descriptions

**Boundary A — CLI/MCP/LSP Input**: User and AI agent inputs via CLI args, MCP JSON-RPC, or LSP JSON-RPC. Primary entry point for command injection, path traversal, and untrusted data.

**Boundary B — Filesystem**: All file read/write operations. lode-core enforces `ValidatedRoot` (traversal prevention, symlink escape detection, atomic writes). Templates, plugins, config, and hooks are untrusted content read from disk.

**Boundary C — Child Processes**: Git, cargo, npm, make, and other tools spawned via `Process` (shell metacharacter rejection, path separator rejection). Tool output is untrusted.

**Boundary D — Network**: GitHub for release publishing and git operations (optional, explicit opt-in). No other network access.

## 5. Threat Actors

| Actor | Motivation | Access Level | Capability |
|---|---|---|---|
| **Malicious Local User** | Escalate privileges, access other user's data | Same OS user | Run lode commands, modify source/config |
| **Supply Chain Attacker** | Inject malicious code via dependencies | Network | Compromise upstream crate |
| **Plugin Author** | Abuse plugin permissions | Filesystem | Execute arbitrary code via hooks/plugins |
| **AI Agent** (via MCP) | Unintended operations | MCP tool access | Call any of 38 MCP tools |
| **Template Attacker** | Path traversal via template paths | Filesystem | Craft malicious `{% include %}` or `{% extends %}` |
| **Git Hook Attacker** | Code execution via git hooks | Filesystem | Modify `.git/hooks/` pre-existing hooks |

## 6. Threat Register

### T-01: Command Injection via Process

| Attribute | Value |
|---|---|
| ID | T-01 |
| Title | Command injection via child process execution |
| Threat Actor | Malicious Local User, AI Agent |
| Attack Vector | Crafted program name containing shell metacharacters or path separators |
| Target | `Process::new()` in lode-core |
| Existing Control | `Process::validate_program()` rejects `/`, `\`, `:`, null byte, all shell metacharacters (17 characters) |
| Existing Control Confidence | High — tested with 5 fuzz + unit tests, fuzz target in CI |
| Bypass Risk | Low — validation is comprehensive, fuzzed |
| Residual Risk | Low |
| Impact | High — arbitrary command execution |
| Likelihood | Low |

### T-02: Path Traversal via ValidatedRoot

| Attribute | Value |
|---|---|
| ID | T-02 |
| Title | Path traversal via ValidatedRoot |
| Threat Actor | Malicious Local User, Template Attacker, Plugin Author |
| Attack Vector | Relative path with `../`, absolute path, symlink to outside root |
| Target | `ValidatedRoot::resolve()` in lode-core |
| Existing Control | Canonical path comparison, symlink resolution, component validation |
| Existing Control Confidence | High — 9 unit tests + fuzz target in CI |
| Bypass Risk | Low |
| Residual Risk | Low |
| Impact | High — arbitrary file write |
| Likelihood | Low |

### T-03: Template Engine Abuse

| Attribute | Value |
|---|---|
| ID | T-03 |
| Title | Template engine abuse (include/extends traversal, recursion) |
| Threat Actor | Template Attacker |
| Attack Vector | `{% include "../outside.txt" %}`, `{% extends "/absolute/path" %}`, nested recursion |
| Target | `template.rs` in lode-core |
| Existing Control | `safe_template_reference()` rejects `..`, `:`, absolute paths. Recursion bound of 16. |
| Existing Control Confidence | High — 9 unit tests covering unsafe paths, recursion |
| Bypass Risk | Low |
| Residual Risk | Low |
| Impact | Medium — file read/write outside expected paths |
| Likelihood | Low |

### T-04: Secret Exposure in Logs/Errors

| Attribute | Value |
|---|---|
| ID | T-04 |
| Title | Secrets leaked in logs, error messages, or MCP responses |
| Threat Actor | Malicious Local User |
| Attack Vector | Trigger error that includes file content with secrets |
| Target | All error handling, logging, MCP response formatting |
| Existing Control | AGENTS.md policy: all secrets redacted before logs, metrics, MCP responses, or errors |
| Existing Control Confidence | Medium — policy exists, enforcement depends on consistent code review |
| Bypass Risk | Medium — relies on developer discipline |
| Residual Risk | Medium |
| Impact | High — credential disclosure |
| Likelihood | Medium |

### T-05: Malicious Plugin or Hook

| Attribute | Value |
|---|---|
| ID | T-05 |
| Title | Malicious plugin or git hook execution |
| Threat Actor | Plugin Author |
| Attack Vector | Install plugin from untrusted source, install git hook with malicious commands |
| Target | `plugin.rs`, `hooks.rs` in lode-cli / lode-core |
| Existing Control | Plugin install receipts track source, permissions (network, execute, fs_write). Hooks discovered from expected directories. `Process` validates command names. |
| Existing Control Confidence | Medium — plugin security model exists but permissions rely on user review during install |
| Bypass Risk | Medium |
| Residual Risk | Medium |
| Impact | High — arbitrary code execution |
| Likelihood | Medium |

### T-06: MCP Input Injection

| Attribute | Value |
|---|---|
| ID | T-06 |
| Title | Injection via MCP JSON-RPC input |
| Threat Actor | AI Agent, Malicious Network Actor |
| Attack Vector | Crafted tool call with malicious arguments (path traversal, command injection, oversized payload) |
| Target | `lode-mcp` server, transport layer |
| Existing Control | Max message size 1 MB. Tools reuse lode-core security controls (ValidatedRoot, Process). STDIO transport only. |
| Existing Control Confidence | Medium — input validation exists but is delegated to downstream functions |
| Bypass Risk | Medium — depends on each tool handler validating inputs |
| Residual Risk | Medium |
| Impact | High — depends on the tool |
| Likelihood | Medium |

### T-07: LSP Diagnostics Data Leak

| Attribute | Value |
|---|---|
| ID | T-07 |
| Title | Secret data leaked via LSP diagnostics |
| Threat Actor | Editor plugin, Malicious Local User |
| Attack Vector | `textDocument/publishDiagnostics` sends secret findings to editor, which may store or transmit them |
| Target | `lode-lsp` server, LSP client |
| Existing Control | LSP diagnostic messages contain finding details for visibility. Secrets scanner has `.env` allowlist. |
| Existing Control Confidence | Medium — no redaction layer in LSP diagnostics (by design, for user visibility) |
| Bypass Risk | Medium — diagnostics are local-only in STDIO transport |
| Residual Risk | Low |
| Impact | Medium — credential visibility in editor |
| Likelihood | Low |

### T-08: Supply Chain — Compromised Dependency

| Attribute | Value |
|---|---|
| ID | T-08 |
| Title | Compromised upstream dependency |
| Threat Actor | Supply Chain Attacker |
| Attack Vector | Publish malicious version of a dependency (171 total, 2 with ignored advisories) |
| Target | `Cargo.lock`, `Cargo.toml` dependency declarations |
| Existing Control | `cargo audit` in CI (advisory checks). `cargo deny` (license, ban, source checks). Dependabot alerts (if configured). Two advisories explicitly ignored. |
| Existing Control Confidence | Medium — CI catches known advisories but ignores some. No automated dependency update workflow. |
| Bypass Risk | Medium |
| Residual Risk | Medium |
| Impact | High — compromised build chain |
| Likelihood | Low |

### T-09: Daemon IPC Injection

| Attribute | Value |
|---|---|
| ID | T-09 |
| Title | Unauthorized IPC commands to daemon |
| Threat Actor | Malicious Local User (different process) |
| Attack Vector | Connect to daemon IPC socket and send crafted commands |
| Target | lode-daemon IPC server (Unix socket or TCP) |
| Existing Control | Platform-adaptive IPC (Unix socket has file permissions, TCP bound to 127.0.0.1). Commands are limited (Status, Stop, Pause, Resume, ListWatchers, Reload). |
| Existing Control Confidence | Medium — no authentication on IPC. Unix socket permissions depend on umask. |
| Bypass Risk | Medium — any local process can connect |
| Residual Risk | Medium |
| Impact | Medium — daemon disruption, file watching paused |
| Likelihood | Medium |

### T-10: Symlink Attack on Atomic Writes

| Attribute | Value |
|---|---|
| ID | T-10 |
| Title | Symlink race during atomic write |
| Threat Actor | Malicious Local User |
| Attack Vector | Create symlink at temporary path before atomic rename |
| Target | `ValidatedRoot::write_atomic()` in lode-core |
| Existing Control | Atomic write uses `std::fs::rename` (atomic on same filesystem). Temp file created in same directory as target. |
| Existing Control Confidence | Medium — symlink escape is tested, but TOCTOU race between temp creation and rename is theoretically possible |
| Bypass Risk | Low |
| Residual Risk | Low |
| Impact | High — arbitrary file overwrite |
| Likelihood | Low |

## 7. Abuse Case Register

### AC-01: Plugin Installed Without Review

User adds a plugin, skips reviewing permissions, plugin executes malicious commands. Control: Plugin install receipts track `reviewed` flag. Mitigation: Require explicit `allow_unsafe` flag for high-permission plugins.

### AC-02: Config Merge Override Attack

Malicious project `.lode/config.toml` overrides security-critical global config values. Control: Merge chain is defaults < global < project. This is by design but a user may unknowingly weaken security by running `lode init` in an attacker-controlled directory.

### AC-03: Git Hook Pre-Commit Secrets Bypass

User disables or bypasses pre-commit secret scanning hook. Control: Hooks are optional (user must install). Risk accepted: `lode scan secrets` remains available as an explicit command.

### AC-04: Template Include Path Traversal (Non-Control)

Before the `safe_template_reference` control was added, `{% include %}` could reference paths outside the template directory. Now blocked. Verification: Tests confirm rejection.

### AC-05: MCP Agent Prompt Injection

AI agent prompt instructs MCP client to call dangerous tool with malicious args. Control: All tools reuse lode-core security controls. Tool arguments validated by downstream functions.

## 8. Controls Assessment

| Control | Location | Strength | Tested |
|---|---|---|---|
| ValidatedRoot (canonical path enforcement) | `lode-core/src/fs_safety.rs` | Strong | 9 unit + 1 fuzz target |
| Process (metacharacter rejection) | `lode-core/src/process.rs` | Strong | 5 unit + 1 fuzz target |
| Secrets scanner | `lode-core/src/secrets.rs` | Medium | 2 unit tests |
| Template safety | `lode-core/src/template.rs` | Strong | 9 unit tests |
| Recipe destination validation | `lode-core/src/recipe.rs` | Strong | 4 unit tests |
| Scaffold traversal rejection | `lode-core/src/scaffold.rs` | Strong | 6 unit tests |
| Git hook ownership | `lode-core/src/git.rs` | Medium | 4 unit tests |
| Plugin permissions | `lode-cli/src/cmd/plugin.rs` | Medium | 0 direct tests |
| MCP message size limit | `lode-mcp/src/transport.rs` | Medium | 0 direct tests |
| LSP diagnostics (local only) | `lode-lsp/src/lib.rs` | Medium | 10 unit tests |
| Exit code for security findings | `lode-core/src/error.rs` | Strong | Implicit |

## 9. STRIDE per Component

| Component | Spoofing | Tampering | Repudiation | Info Disclosure | DoS | Elevation |
|---|---|---|---|---|---|---|
| lode-cli CLI parser | N/A | Low (clap) | Low (logging) | Low (error msgs) | Low | N/A |
| lode-core ValidatedRoot | N/A | Strong | N/A | N/A | Low | N/A |
| lode-core Process | N/A | Strong | N/A | N/A | Low | Low |
| lode-core secrets scanner | N/A | Medium | N/A | Low (diagnostics) | N/A | N/A |
| lode-mcp server | Low | Medium (input) | Low (logging) | Medium (responses) | Medium (size limit) | Medium (tool calls) |
| lode-lsp server | Low | Medium (input) | N/A | Medium (diagnostics) | Low | Low |
| lode-daemon IPC | Low (no auth) | Medium | Low (logging) | Low | Medium (stop/restart) | Low |
| lode-tui | N/A | Low | N/A | Low (display) | Low | N/A |
| CI/CD pipeline | Low | Medium (ignore advisories) | High | Low | Low | Low |

## 10. Residual Risks

| Risk | Mitigation | Owner | Priority |
|---|---|---|---|
| Secret exposure in error/log output | Code review + automated secret redaction checks | Team | High |
| Plugin security relies on user review | Add plugin permission audit and sandboxing | Team | Medium |
| Daemon IPC has no authentication | Add token-based IPC auth for stop/restart | Team | Medium |
| No MCP transport encryption | Add TLS support for HTTP transport | Team | Low |
| LSP diagnostics may expose secrets in editor | Add redaction option to LSP config | Team | Low |
| Two advisory ignores unverified | Review paste/lru status and remove ignores | Team | Medium |
| No automated dependency updates | Enable Dependabot or Renovate | Team | Low |

## 11. Security Validation Priorities

1. **HIGH**: Run UPPS-05 (Master Security Audit) to validate all controls against actual code
2. **HIGH**: Verify secret redaction is consistently applied across all crates
3. **MEDIUM**: Review plugin permission model and add tests
4. **MEDIUM**: Audit the two ignored advisories (paste, lru)
5. **MEDIUM**: Add IPC authentication to daemon
6. **LOW**: Add MCP HTTP transport security (TLS)

## 12. Completion State

**PASS** — Threat model covers all identified assets, trust boundaries, and actors across 10 documented threats and 5 abuse cases. STRIDE analysis per component. Security validation priorities established for UPPS-05.
