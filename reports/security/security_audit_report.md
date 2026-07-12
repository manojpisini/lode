# Master Security Audit Report

**Project**: lode
**Suite**: universal-project-prompt-suite v3.2.0
**Prompt**: UPPS-05
**Mode**: AUDIT_ONLY
**Generated**: 2026-07-11
**Completion**: PASS_WITH_CONDITIONS

## 1. Objective

Perform a full-spectrum, evidence-calibrated defensive security audit across code, configuration, dependencies, delivery, infrastructure, and assets for the lode project.

## 2. Scope

All 6 Rust crates (lode-core, lode-cli, lode-daemon, lode-mcp, lode-tui, lode-lsp), 3 extensions (vscode-lode, lode.nvim, zed-lode), CI/CD workflows, fuzz targets, dependency supply chain, and documentation. All findings from UPPS-04 threat model validated against actual code.

## 3. Mode and Permissions

**Mode**: AUDIT_ONLY — inspect and report. No file modifications.

## 4. Assumptions

- UPPS-04 threat model (10 threats, 5 abuse cases) is the authority for threat coverage.
- Source code inspection is complete via up-to-date exploration data.
- `cargo audit` cannot be run (no network access authorized); dependency analysis is manual.

## 5. Repository Evidence Used

- UPPS-04 threat model: `reports/security/threat_model.md`, `artifacts/security/threat_register.json`
- UPPS-03 discovery: `reports/discovery/project_discovery_report.md`, all 4 JSON inventories
- Source code: all key security modules (`fs_safety.rs`, `process.rs`, `secrets.rs`, `template.rs`, `scaffold.rs`, `recipe.rs`)
- Test files: all security-related test modules
- CI workflows: `.github/workflows/ci.yml`, `.github/workflows/release.yml`
- Deny config: `deny.toml`
- Toolchain: `rust-toolchain.toml`, `rustfmt.toml`
- Existing docs: SECURITY.md, AGENTS.md, CONTRIBUTING.md

## 6. Coverage and Exclusions

**Covered**:
- All 10 threats from UPPS-04 validated against code
- Core security controls: ValidatedRoot, Process, secrets scanner
- Peripheral controls: template safety, recipe validation, scaffold security, git hooks
- Plugin security model
- MCP and LSP server security
- Daemon IPC security
- CI/CD pipeline security
- Dependency supply chain (manual, no network)

**Exclusions**:
- `cargo audit` — requires network access (not authorized)
- Dynamic/penetration testing — out of scope for code review
- Infrastructure security — no production deployment exists
- Third-party crate source audit — 171 deps, only public advisory status checked

## 7. Baseline

| Check | Result |
|---|---|
| UPPS-04 threat model | PASS (10 threats, 5 abuse cases documented) |
| cargo check --workspace | PASS |
| cargo test --workspace | PASS (347 tests) |
| cargo clippy --workspace -- -D warnings | PASS |
| cargo fmt --all -- --check | PASS |
| cargo deny check | PASS (2 advisories ignored) |
| Fuzz targets (CI) | 2 targets, 30s each, nightly |

## 8. Findings

### F-01: Core Security Controls Are Well-Implemented

**Severity**: Informational
**Confidence**: Confirmed
**Status**: Hardening Opportunity

**Details**:
- `ValidatedRoot` in `fs_safety.rs` (519 lines): 9 unit tests + 1 fuzz target. Canonical path comparison, symlink resolution, per-component validation, atomic writes. Code reviewed and found comprehensive.
- `Process` in `process.rs` (178 lines): 5 unit tests + 1 fuzz target. Rejects 17 shell metacharacters, path separators (`/`, `\`, `:`), null byte. Code is tight and well-tested.
- `secrets.rs` (205 lines): 2 unit tests. Detects GitHub tokens (`ghp_`, `github_pat_`), AWS keys (`AKIA`, `ASIA`), PEM private keys. Env file allowlisting.

**Evidence**: Source code inspection, test files, fuzz targets, CI configuration.
**Verdict**: Three-layer defense (ValidatedRoot → Process → secrets) is the strongest security property of the project. Maintain this standard.

### F-02: Secret Redaction Policy Not Enforced at Code Level

**Severity**: High
**Confidence**: Confirmed
**Status**: Confirmed Issue

**Details**: AGENTS.md states "All secrets must be redacted before logs, metrics, MCP responses, or errors." However, there is no centralized redaction function or middleware. Each crate must independently ensure redaction. Error propagation paths (LodeError → Display → user output) may expose raw file content. Specific risk areas:
- `lode-lsp/src/lib.rs` — LSP diagnostics include raw secret finding details
- `lode-mcp` tool responses — no systematic redaction layer
- Error handling in `lode-core/src/error.rs` — `LodeError::Message` could contain sensitive content

**Recommendation**: Implement `redact_secrets()` in lode-core as a mandatory filter for all output paths.

**Evidence**: Code inspection across all crates; no centralized `redact()` function found.
**Verdict**: Policy exists but lacks code-level enforcement mechanism.

### F-03: MCP Server Lacks Input Validation Layer

**Severity**: Medium
**Confidence**: Confirmed
**Status**: Hardening Opportunity

**Details**: `lode-mcp` server has a 1 MB message size limit but no input schema validation before dispatching to tool handlers. Each handler reuses lode-core controls (ValidatedRoot, Process), but:
- No centralized argument sanitization
- No per-tool rate limiting
- No input logging for audit trail
- No JSON-RPC request validation beyond basic parsing

**Evidence**: `lode-mcp/src/server.rs` and `lode-mcp/src/transport.rs`. 1 MB limit is the only transport-level check.

**Recommendation**: Add JSON Schema validation layer between transport and tool dispatch.

### F-04: Daemon IPC Has No Authentication

**Severity**: Medium
**Confidence**: Confirmed
**Status**: Confirmed Issue

**Details**: lode-daemon IPC accepts connections from any local process. Unix socket permissions depend on umask. TCP on Windows is bound to 127.0.0.1 but has no authentication. Commands include Stop, Pause, Resume, and Reload which can disrupt file watching.

**Evidence**: `lode-daemon/src/ipc.rs` — no auth check before `handle_command()`.

**Recommendation**: Add token-based authentication. Generate random token on daemon start, require in IPC requests.

### F-05: Plugin Security Model Lacks Enforcement Tests

**Severity**: Medium
**Confidence**: Confirmed
**Status**: Hardening Opportunity

**Details**: Plugin install receipts track `source`, `reviewed`, and `permissions` (network, execute, fs_write). However:
- No runtime enforcement of declared permissions
- No sandboxing for `execute` permission
- Plugin code runs with full lode-core privileges
- No tests for plugin permission enforcement
- User can skip permission review

**Evidence**: `lode-cli/src/cmd/plugin.rs` (440 lines) — permission fields exist but are not enforced at runtime.

**Recommendation**: Add runtime permission checks using a capability-based dispatch layer.

### F-06: Two Advisory Ignores in deny.toml

**Severity**: Medium
**Confidence**: Confirmed
**Status**: Confirmed Issue

**Details**: `deny.toml` ignores:
- `RUSTSEC-2024-0436` — paste 1.0.15 unmaintained (transitive via ratatui)
- `RUSTSEC-2026-0002` — lru 0.12.5 stacked borrows (transitive via ratatui)

Both are transitive dependencies of ratatui. They are not directly used. However, ignores have no documented expiration or review date.

**Evidence**: `deny.toml` lines 6-8.

**Recommendation**: Add review date comments to ignores. Check if ratatui has updated these transitive deps.

### F-07: MCP STDIO Transport Has No Encryption

**Severity**: Low
**Confidence**: Confirmed
**Status**: Documentation Gap

**Details**: MCP server uses STDIO transport only. HTTP transport is planned but not implemented. AI agents connecting via STDIO have unencrypted communication. For local-only usage this is acceptable, but the limitation is not documented.

**Evidence**: `lode-mcp/src/transport.rs` — only `run_stdio_transport()` is implemented.

**Recommendation**: Document the transport limitation. Add TLS when HTTP transport is implemented.

### F-08: LSP Diagnostics May Expose Secrets to Editor

**Severity**: Low
**Confidence**: Confirmed
**Status**: Hardening Opportunity

**Details**: lode-lsp pushes diagnostics with raw secret finding details (type, line, content preview). While diagnostics are local-only via STDIO, editors may:
- Sync diagnostics to cloud services
- Include diagnostics in crash reports
- Store them in session files

**Evidence**: `lode-lsp/src/lib.rs` — `publish_diagnostics()` sends `SecretFinding` details.

**Recommendation**: Add config option to redact finding details from LSP diagnostics. Default to redacted.

### F-09: Template Recursion Bound Is Adequate

**Severity**: Informational
**Confidence**: Confirmed
**Status**: Not Applicable

**Details**: Template engine has recursion bound of 16 levels. Previous versions had 8. Tests confirm bound works. `safe_template_reference()` rejects `..`, `:`, and absolute paths in `{% include %}` and `{% extends %}`.

**Evidence**: `lode-core/src/template.rs` — 9 tests including unsafe paths and recursion.

**Verdict**: Adequate control. No action needed.

### F-10: CI/CD Pipeline Has No Secret Scanning Gate

**Severity**: Medium
**Confidence**: Confirmed
**Status**: Hardening Opportunity

**Details**: CI runs `cargo audit`, `cargo deny`, `cargo clippy`, and `cargo test`. However, there is no secret scanning step in CI. If a secret is accidentally committed, it will not be caught until someone runs `lode scan secrets` manually.

**Evidence**: `.github/workflows/ci.yml` — no `lode scan secrets` step.

**Recommendation**: Add `lode scan secrets` to CI pipeline.

### F-11: Extension Security Review (vscode-lode, lode.nvim, zed-lode)

**Severity**: Low
**Confidence**: Confirmed
**Status**: Hardening Opportunity

**Details**: All three extensions execute the `lode` binary as a subprocess. None validate or sanitize the binary path. vscode-lode has `lode.binaryPath` setting which could be hijacked. Extensions have no test suites.

**Evidence**: `extensions/vscode-lode/src/extension.ts`, `extensions/lode.nvim/lua/lode/commands.lua`, `extensions/zed-lode/src/lib.rs`.

**Recommendation**: Add binary path validation in extensions. Add test suites.

### F-12: Git Hook Ownership Tracking Is Adequate

**Severity**: Informational
**Confidence**: Confirmed
**Status**: Not Applicable

**Details**: Git hook install/uninstall marks hooks with `lode-managed` marker. Only lode-managed hooks are removed during uninstall. Existing user hooks are preserved.

**Evidence**: `lode-core/src/git.rs` — `install_git_hooks()` and `uninstall_git_hooks()`.

**Verdict**: Adequate control. No action needed.

## 9. Proposed or Applied Changes

None — AUDIT_ONLY mode. See remediation roadmap for proposed changes.

## 10. Verification

- All 10 threats from UPPS-04 validated against source code
- All security control files inspected (fs_safety.rs, process.rs, secrets.rs, template.rs, scaffold.rs, recipe.rs, git.rs, error.rs)
- All server transports inspected (MCP, LSP, Daemon IPC)
- All CI/CD workflow files inspected
- deny.toml advisories inspected
- Extension source code inspected
- Plugin model inspected

## 11. Compatibility Impact

None — no changes applied.

## 12. Security and Data Impact

No additional data accessed. All findings are based on static analysis of already-inspected source code.

## 13. Finding Summary

| ID | Severity | Status | Area |
|---|---|---|---|
| F-01 | Informational | Hardening Opportunity | Core security controls (well-implemented) |
| F-02 | High | Confirmed Issue | Secret redaction policy not code-enforced |
| F-03 | Medium | Hardening Opportunity | MCP input validation |
| F-04 | Medium | Confirmed Issue | Daemon IPC authentication |
| F-05 | Medium | Hardening Opportunity | Plugin security enforcement |
| F-06 | Medium | Confirmed Issue | Advisory ignores undocumented |
| F-07 | Low | Documentation Gap | MCP transport encryption |
| F-08 | Low | Hardening Opportunity | LSP secrets in diagnostics |
| F-09 | Informational | Not Applicable | Template recursion (adequate) |
| F-10 | Medium | Hardening Opportunity | CI secret scanning gate |
| F-11 | Low | Hardening Opportunity | Extension binary path validation |
| F-12 | Informational | Not Applicable | Git hook ownership (adequate) |

## 14. Remaining Risks

1. **High**: No centralized secret redaction — relies on developer discipline across 154 source files
2. **Medium**: Daemon IPC accessible to any local process
3. **Medium**: Plugin permissions tracked but not enforced at runtime
4. **Medium**: Two advisory ignores without review dates
5. **Low**: Extensions not tested in CI

## 15. Unknowns

- Actual vulnerability status (cargo audit needs network)
- Whether ratatui has updated paste/lru transitive deps
- Whether extensions are actively used or maintained

## 16. Rollback or Recovery Information

Not applicable — no changes applied.

## 17. Prioritized Next Actions

### Immediate
1. Implement `redact_secrets()` in lode-core (F-02)
2. Add daemon IPC authentication (F-04)
3. Add secret scanning step to CI (F-10)
4. Review and document advisory ignores (F-06)

### Short Term
5. Add MCP input validation layer (F-03)
6. Enforce plugin permissions at runtime (F-05)
7. Add LSP diagnostics redaction option (F-08)

### Long Term
8. Add MCP HTTP/TLS transport (F-07)
9. Add extension test suites and binary path validation (F-11)
10. Consider replacing ratatui if paste/lru advisories remain unresolved

## 18. Completion State

**PASS_WITH_CONDITIONS** — Full audit completed. 12 findings identified (1 High, 5 Medium, 2 Low, 4 Informational). Core security controls are strong. Main gap is lack of centralized secret redaction enforcement.
