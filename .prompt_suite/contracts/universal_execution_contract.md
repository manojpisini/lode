# Universal Execution Contract

**Project**: lode
**Suite**: universal-project-prompt-suite v3.2.0
**Generated**: 2026-07-11
**Mode**: AUDIT_ONLY
**Scope**: entire repository

## A. Operating Mode

**Mode**: `AUDIT_ONLY`

Inspect and report. Do not modify repository files outside of `.prompt_suite/` and `logs/`.

## B. Required Project Inputs

```yaml
PROJECT_ROOT: "."
PROJECT_NAME: "lode"
PROJECT_TYPE: "hybrid_cli_tool"
PROJECT_PURPOSE: "Local Rust developer tool with filesystem writes, daemon automation, child-process execution, plugins, MCP tools, and secret scanning"
MODE: "AUDIT_ONLY"
SCOPE: "entire repository"
PRIORITY_AREAS:
  - documentation
  - security
  - code quality
EXCLUSIONS:
  - target/ (build artifacts)
  - node_modules/
PROTECTED_PATHS:
  - .prompt_suite/
  - .github/workflows/
  - Cargo.lock
SUPPORTED_PLATFORMS:
  - x86_64-unknown-linux-gnu
  - x86_64-apple-darwin
  - x86_64-pc-windows-gnu
DEPLOYMENT_TARGETS:
  - github_releases
NETWORK_ACCESS: false
ALLOW_EXTERNAL_LOOKUPS: false
ALLOW_INSTALLS: false
ALLOW_FILE_MODIFICATION: false
ALLOW_FILE_DELETION: false
ALLOW_RENAMES_OR_MOVES: false
ALLOW_DEPENDENCY_CHANGES: false
ALLOW_DATA_MUTATION: false
ALLOW_MIGRATIONS: false
ALLOW_PUBLIC_CONTRACT_CHANGES: false
ALLOW_SERVICE_STARTUP: false
ALLOW_UNKNOWN_BINARY_EXECUTION: false
REQUIRED_COMMANDS:
  - cargo check --workspace
  - cargo test --workspace
FORBIDDEN_COMMANDS:
  - cargo publish
  - git push
  - rm -rf target/
ACCEPTANCE_CRITERIA:
  - all cargo check --workspace passes
  - all cargo test --workspace passes
  - all output files follow lowercase snake_case
ADDITIONAL_CONTEXT: ""
```

## C. Evidence Rules

1. Inspect before concluding.
2. Cite file paths, symbols, line ranges, configuration keys, command output, or reproducible observations.
3. Mark important statements as `Verified`, `Inferred`, `Proposed`, `Unknown`, or `Not Applicable` when ambiguity matters.
4. Never treat a file's existence as proof that it is active.
5. Never treat a textual reference search as complete reachability analysis.
6. Never treat a passing test as proof of total correctness.
7. Never treat comments, TODOs, examples, issue references, or roadmap notes as implemented behavior without corroboration.
8. Never claim complete coverage without an inventory that accounts for exclusions, opaque files, generated files, binaries, archives, unsupported formats, and tool failures.
9. When sources conflict, identify the conflict, rank likely authority, and preserve unresolved ambiguity.
10. Record exact commands and exact outcomes; do not paraphrase failed verification into success.

## D. Safety and Permission Rules

1. Do not access production systems unless explicitly authorized.
2. Do not access networks or external services unless explicitly authorized.
3. Do not reveal complete secrets, private keys, tokens, credentials, personal data, or sensitive connection strings.
4. Do not execute unknown binaries or untrusted scripts.
5. Do not install or upgrade dependencies unless explicitly authorized.
6. Do not delete, rename, move, or overwrite files unless explicitly authorized.
7. Do not mutate databases, indexes, queues, object stores, caches, or user data unless explicitly authorized.
8. Do not rewrite migration history unless a project-specific authority explicitly permits it.
9. Do not change public APIs, CLI behavior, configuration keys, events, schemas, serialized formats, package exports, or supported paths without explicit authorization and compatibility analysis.
10. Do not discard or overwrite pre-existing local changes.
11. Prefer reversible, reviewable, minimal changes.
12. Stop applying changes when unexpected regression, data risk, contract drift, or unexplained scope expansion appears.

## E. Repository Ground Truth

Consumed from `./.prompt_suite/context/project_context.yaml` (UPPS-00) and verified during UPPS-03.

## F. Baseline Rules

Before modifications, record:
- Current revision and branch
- Dirty, staged, modified, and untracked state
- Existing generated artifacts
- Existing build outcome
- Existing test outcome
- Existing lint, formatting, type-check, and static-analysis outcome
- Known pre-existing failures

Do not attribute pre-existing failures to new work.

## G. Change Discipline

For each proposed or applied change:
- State the objective.
- State the evidence.
- State the affected files and contracts.
- State compatibility risk.
- State data and security implications.
- State verification.
- State rollback.
- Keep mechanical changes separate from semantic changes.
- Keep dependency changes separate from cleanup.
- Keep migrations separate from unrelated refactors.
- Keep documentation corrections traceable to source evidence.
- Avoid unrelated formatting.

## H. Verification Discipline

Use the narrowest relevant check first, followed by broader available checks.

Applicable checks:
- Parser or syntax validation
- Formatting check
- Lint
- Unit tests
- Integration tests
- Fuzz/property tests
- Packaging and installation
- Documentation link validation

A skipped, unavailable, unauthorized, or failed check must be reported explicitly.

## I. Finding Classification

Impact: `Critical`, `High`, `Medium`, `Low`, `Informational`
Confidence: `Confirmed`, `High`, `Medium`, `Low`
Status: `Confirmed Issue`, `Likely Issue`, `Risk`, `Hardening Opportunity`, `Documentation Gap`, `Needs Manual Review`, `Not Applicable`, `False Positive`

## J. Completion States

`PASS`, `PASS_WITH_CONDITIONS`, `FAIL`, `INCOMPLETE`

## K. Mandatory Final Report Sections

1. Objective
2. Scope
3. Mode and permissions
4. Assumptions
5. Repository evidence used
6. Coverage and exclusions
7. Baseline
8. Findings
9. Proposed or applied changes
10. Verification
11. Compatibility impact
12. Security and data impact
13. Remaining risks
14. Unknowns
15. Rollback or recovery information
16. Prioritized next actions
17. Completion state
