# Release Action Plan

**Project**: lode
**Target Version**: v0.1.0
**Prompt**: UPPS-23
**Generated**: 2026-07-11
**Decision**: CONDITIONAL_GO

## Pre-Release Blockers (Must Complete)

| # | Action | Effort | Owner | UPPS Ref |
|---|---|---|---|---|
| 1 | Implement `redact_secrets()` in lode-core and wire to all output boundaries | 3-5 days | — | F-02 (UPPS-05) |
| 2 | Draft v0.1.0 CHANGELOG.md with latest commits | 0.5 day | — | UPPS-23 |
| 3 | Add cargo-cyclonedx SBOM generation to release.yml | 1 day | — | F-SC-06 (UPPS-13) |

## Release Day Checklist

- [ ] `cargo check --workspace` passes
- [ ] `cargo test --workspace` passes (347 tests)
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo deny check` passes
- [ ] Version bumped from 0.1.0 if any changes made
- [ ] CHANGELOG reviewed and finalized
- [ ] Tag created (`git tag v0.1.0 && git push --tags`)
- [ ] Release workflow triggered (tag push or workflow_dispatch)
- [ ] Verify binaries download and run on all 3 platforms

## Post-Release (v0.1.1 / v0.2.0)

| Priority | Action | Effort | Target |
|---|---|---|---|
| P1 | Daemon IPC token-based authentication | 1-2 days | v0.1.1 |
| P1 | Document advisory ignores with expiry | 0.5 day | v0.1.1 |
| P1 | Add lode scan secrets to CI | 0.5 day | v0.1.1 |
| P2 | MCP input validation layer | 2-3 days | v0.2.0 |
| P2 | Plugin permission runtime enforcement | 2-3 days | v0.2.0 |
| P2 | Enable Dependabot | 0.5 day | v0.2.0 |
| P3 | Metrics/tracing layer | 3-5 days | v0.2.0 |
| P3 | Incident response plan | 1 day | v0.2.0 |
| P3 | Code of conduct | 0.5 day | Before public announcement |
| P3 | security.txt | 0.5 day | Before public announcement |
