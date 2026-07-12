# Phased Improvement Plan

**Project**: lode
**Prompt**: UPPS-24
**Generated**: 2026-07-11
**Status**: PLAN — no execution (AUDIT_ONLY mode)

## Phase 1: Security Fundamentals (1 week)

1. **P0** Centralize secret redaction in lode-core (3-5 days)
2. **P1** Document advisory ignores with expiry dates (0.5 day)
3. **P1** Add MCP input length limits (1 day)

## Phase 2: Dependency Hardening (1-2 weeks)

4. **P2** Enable Dependabot (0.5 day)
5. **P2** Add SBOM generation to release CI (1 day)
6. **P2** Submit security.txt (0.5 day)

## Phase 3: Operational Maturity (2-4 weeks)

7. **P3** Define incident response plan (1 day)
8. **P3** Evaluate and decide on metrics/tracing (decision + 3-5 days)
9. **P3** Daemon IPC auth review and implementation (2 days)

## Effort Summary

| Phase | Items | Total Effort |
|---|---|---|
| Phase 1 | 3 | 5-7 days |
| Phase 2 | 3 | 2 days |
| Phase 3 | 3 | 6-8 days |
| **Total** | **9** | **13-17 days** |
