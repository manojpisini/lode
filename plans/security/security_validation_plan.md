# Security Validation Plan

**Project**: lode
**Suite**: universal-project-prompt-suite v3.2.0
**Prompt**: UPPS-04
**Generated**: 2026-07-11

## Phase 1: Immediate (Run UPPS-05)

1. **UPPS-05 — Master Security Audit**: Perform full security audit of all 6 crates. All 10 threats from the threat model should be validated against actual code. Coverage:
   - ValidatedRoot implementation audit
   - Process validation audit
   - Secret scanner completeness
   - Template safety verification
   - Plugin permission model review
   - MCP input validation review
   - LSP diagnostics content review
   - Daemon IPC security review
   - CI/CD security posture
   - Supply chain controls review

## Phase 2: High Priority (Post UPPS-05)

2. **Secret Redaction Audit**: Verify that all error paths, log statements, and MCP response formatters consistently redact secrets. Audit all crates for uncovered error propagation.

3. **Advisory Ignore Review**: Investigate and resolve:
   - `RUSTSEC-2024-0436` (paste 1.0.15 unmaintained) — check if ratatui still requires it
   - `RUSTSEC-2026-0002` (lru 0.12.5 stacked borrows) — check if fix available

## Phase 3: Medium Priority

4. **Plugin Security Hardening**:
   - Add tests for plugin permission enforcement
   - Implement mandatory permission review on first run
   - Add plugin sandboxing for `execute` permission
   - Add plugin integrity verification (manifest signing)

5. **Daemon IPC Authentication**:
   - Add token-based authentication for IPC
   - Generate random token on daemon start
   - Require token for Stop/Restart commands
   - Implement rate limiting

## Phase 4: Low Priority

6. **MCP Transport Security**:
   - Add TLS support for HTTP transport
   - Document that STDIO transport has no encryption

7. **LSP Diagnostics Redaction**:
   - Add config option to redact secret values from diagnostic details
   - Default to redacted, option to show full details

8. **Software Bill of Materials**:
   - Generate SBOM as part of CI (cargo cyclonedx or similar)
   - Include in release assets

## Verification Gates

| Gate | Trigger | Verification |
|---|---|---|
| G1 | After UPPS-05 | All 10 threats validated |
| G2 | After secret redaction audit | No uncovered error paths leak secrets |
| G3 | After advisory review | No ignored advisories without documented rationale |
| G4 | After plugin hardening | Plugin permission tests pass |
| G5 | After IPC auth | Daemon rejects unauthenticated Stop/Restart |
