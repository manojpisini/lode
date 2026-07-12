# Dependency Upgrade Plan

**Project**: lode
**Prompt**: UPPS-13
**Generated**: 2026-07-11
**Status**: PLAN — no execution (AUDIT_ONLY mode)

## Phase 1: Immediate (1-2 weeks)

1. **Document advisory ignores** — add expiry and justification to `deny.toml`
2. **Update license allowlist** — add `Unicode-DFS-2016` to `deny.toml` if not already present

## Phase 2: Short-term (1-4 weeks)

3. **Enable Dependabot** — add `.github/dependabot.yml` for weekly cargo updates
4. **Track ratatui upstream** — file issue or watch for paste/lru replacement

## Phase 3: Medium-term (1-3 months)

5. **Add SBOM generation** — `cargo cyclonedx` to release workflow
6. **Add cargo-audit to pre-commit** — block commits with new advisories
7. **Lockfile diff in CI** — detect unexpected dependency changes

## Risk Acceptance

The two advisory ignores are both transitive through ratatui, a well-maintained crate.
Acceptable to defer if ratatui plans to update within the next release cycle.
