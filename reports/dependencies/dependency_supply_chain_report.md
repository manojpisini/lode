# Dependency and Supply Chain Review

**Project**: lode
**Suite**: universal-project-prompt-suite v3.2.0
**Prompt**: UPPS-13
**Mode**: AUDIT_ONLY
**Generated**: 2026-07-11
**Completion**: PASS_WITH_CONDITIONS

## 1. Objective

Review all third-party code, dependency resolution, build-time execution, vulnerabilities, licensing, and reproducibility for the lode project.

## 2. Scope

171 packages across 6 workspace crates + CI tooling (cargo-audit, cargo-deny, cargo-llvm-cov, cargo-fuzz).

## 3. Mode and Permissions

**Mode**: AUDIT_ONLY — inspect and report. No dependency changes. No network access.

## 4. Assumptions

- `cargo metadata` output is the authoritative dependency tree.
- `deny.toml` is the current license and advisory policy.
- `cargo audit` cannot be run (no network). Manual advisory review only.

## 5. Dependency Inventory

### Workspace Crates (6)
lode-core, lode-cli, lode-daemon, lode-mcp, lode-tui, lode-lsp

### Direct Dependencies by Crate

| Crate | Direct Deps | Category |
|---|---|---|
| lode-core | 8 | camino, dirs, serde, serde_json, thiserror, toml, regex, sha2 |
| lode-cli | 9 | camino, clap, crossterm, lode-core, ratatui, serde, serde_json, sha2, toml |
| lode-daemon | 8 | camino, clap, lode-core, notify, serde, serde_json, thiserror, tokio |
| lode-mcp | 5 | camino, clap, lode-core, serde_json, toml |
| lode-tui | 6 | camino, crossterm, lode-core, ratatui, serde, serde_json |
| lode-lsp | 3 | camino, lode-core, serde, serde_json |

### Total: 171 resolved packages (including transitive)

## 6. Supply Chain Findings

### F-SC-01: License Compliance

**Severity**: Informational
**Confidence**: Confirmed
**Status**: Verified

- `deny.toml` allows MIT, Apache-2.0, and "MIT OR Apache-2.0"
- `cargo deny check` passes
- All 171 packages comply with license policy
- No copyleft licenses detected

### F-SC-02: Source Integrity

**Severity**: Informational
**Confidence**: Confirmed
**Status**: Verified

- `deny.toml` bans unknown-registry and unknown-git sources
- All packages from crates.io (the default registry)
- No git dependencies or path dependencies from external sources
- `Cargo.lock` pinned for reproducible builds

### F-SC-03: Advisory Ignores Need Review

**Severity**: Medium
**Confidence**: Confirmed
**Status**: Confirmed Issue

Two advisories explicitly ignored in `deny.toml`:
1. **RUSTSEC-2024-0436**: paste 1.0.15 — unmaintained (transitive via ratatui)
2. **RUSTSEC-2026-0002**: lru 0.12.5 — stacked borrows UB (transitive via ratatui)

Both are transitive through ratatui 0.28.1. The paste crate is unmaintained but unlikely to be a direct attack vector (it's a proc macro for `paste!` syntax). The lru stacked borrows issue can cause UB but is a soundness issue in a cache implementation.

**Recommendation**: Review and add expiry dates. Check if ratatui has updated.

### F-SC-04: No Wildcard Dependencies

**Severity**: Informational
**Confidence**: Confirmed
**Status**: Verified

`deny.toml` denies wildcard dependencies. Cargo.toml uses exact or caret versions. `Cargo.lock` provides deterministic builds.

### F-SC-05: No Automated Dependency Updates

**Severity**: Low
**Confidence**: Confirmed
**Status**: Hardening Opportunity

No Dependabot, Renovate, or similar automated update mechanism. CI runs `cargo audit` and `cargo deny` manually via workflow steps.

**Recommendation**: Enable Dependabot for monthly dependency update PRs.

### F-SC-06: No Software Bill of Materials (SBOM)

**Severity**: Low
**Confidence**: Confirmed
**Status**: Documentation Gap

No SBOM generation in CI. No `cargo cyclonedx` or SPDX output in release artifacts.

**Recommendation**: Add SBOM generation to release workflow.

### F-SC-07: Build-Time Code Execution Risk

**Severity**: Low
**Confidence**: Confirmed
**Status**: Hardening Opportunity

Several crates have `build.rs` scripts and proc macros (clap_derive, serde_derive, wit-bindgen-rust-macro, etc.). These execute arbitrary code during build. All are from trusted registries.

## 7. Upgrade Impact Matrix

No pending upgrades evaluated — `cargo outdated` not available without network.

| Dependency | Current | Latest Known | Risk | Priority |
|---|---|---|---|---|
| paste (transitive via ratatui) | 1.0.15 | unmaintained | Low | Review |
| lru (transitive via ratatui) | 0.12.5 | unknown | Low | Review |

## 8. Recommendations

1. **Address advisory ignores** — review and document expiry
2. **Enable Dependabot** — monthly automated dependency PRs
3. **Add SBOM generation** — include in release workflow (cargo cyclonedx)
4. **Monitor ratatui updates** — track progress on replacing paste/lru
5. **Consider lockfile auditing** — add `cargo audit` to pre-commit hook

## 9. Completion State

**PASS_WITH_CONDITIONS** — supply chain review complete. 7 findings. Key action: resolve advisory ignores.
