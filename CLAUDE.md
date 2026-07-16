<!-- BEGIN MD MANAGED GUIDANCE -->
## MD prompt routing and productivity guidance

This managed section connects **CLAUDE.md** to Mission Directives **1.8.0** at `prompts`. Preserve instructions outside this block. Regenerate this block with `python prompts/tools/sync_agent_guidance.py --project-root .` instead of editing it manually.

**Scope note:** Only AGENTS.md and CLAUDE.md are managed. Other agent instruction filenames are intentionally excluded.

### When to invoke MD

- Treat the standalone keyword `MD` as an explicit request to use this suite.
- If the user supplies an exact `MD-###` or `C-###`, inspect that target with `explain` before execution.
- If `MD` is followed by ordinary words, treat those words as a lookup query. Do not guess a prompt ID from memory.
- Even without the keyword, use MD when the request clearly benefits from its evidence, authorization, skill, loop, artifact, or verification contracts.
- Use `MD-191` only for ambiguities whose answers change routing, authority, evidence lane, output medium, budget, or acceptance criteria. Do not interrogate the user about details that can be safely inferred or deferred.

### Fast lookup workflow

```bash
python prompts/tools/md.py lookup "<user terms>" --limit 8
python prompts/tools/md.py explain <MD-ID|C-ID|DEPARTMENT_PACK>
python prompts/tools/md.py plan <target> --mode <MODE> --root . --dry-run
```

Lookup before opening many prompt files. Prefer the highest-confidence exact prompt or composite scenario. If results remain ambiguous, ask one route-changing question, then rerun lookup.

### Productivity shortcuts

| Keyword after `MD` | Preferred route | Purpose |
|---|---|---|
| `clarify` | `MD-191` | ask only route-changing questions |
| `skill` | `MD-192 -> MD-196` | prove genuine skill need and execute an exact skill |
| `find skill` | `MD-193 -> MD-194 or MD-195` | discover, qualify, install, or create a missing reusable skill |
| `loop` | `MD-197 -> MD-198` | bounded repeat execution with independent exit adjudication |
| `audit fix verify` | `C-108` | convergent audit, remediation, and verification |
| `research` | `C-26` | deep research report |
| `report` | `C-95` | professional report pipeline |
| `add prompt` | `MD-199 / tools/add_prompt.py` | review, normalize, register, test, and transactionally add one prompt |
| `prompt` | `C-94` | prompt creation, optimization, evaluation, and repair |
| `feature` | `C-63` | feature delivery |
| `visual assets` | `C-109` | code-native vector, illustration, infographic, and presentation-asset production |
| `strudel` | `C-110` | Strudel composition and bounded refinement |
| `productivity` | `MD-138` | personal knowledge, productivity, and work system |

These shortcuts are defaults, not blind dispatch rules. Confirm that the route owns the requested outcome and that its authority and evidence assumptions fit.

### Adding a prompt

- Use `MD-199` when the prompt needs overlap analysis, refinement, routing decisions, or agentic review.
- Use `python prompts/tools/add_prompt.py --source <file.md> --title "<title>"` for a deterministic transactional addition.
- Never copy a prompt into the prompts directory manually; the catalog, identity registry, graph, templates, skills, fixtures, evaluations, tests, validation, and manifest must remain synchronized.

### Canonical lookup order

1. Run `python prompts/tools/md.py lookup "<terms>"`.
2. Inspect the selected route with `python prompts/tools/md.py explain <target>`.
3. Use `prompts/catalog.json` for prompt metadata and `prompts/SCENARIO_CATALOG.json` for composite workflows.
4. Use `prompts/PROMPT_EXECUTION_ORDER.md` for phase order, modes, branches, locks, and completion semantics.
5. Load only the selected bodies from `prompts/prompts/` plus their declared prerequisites.
6. Consult schemas, policies, `prompts/skill_registry.json`, `prompts/auto_prompt_policy.json`, and `prompts/loop_execution_policy.json` only when triggered.

### Efficiency and anti-bloat rules

- Select the **smallest coherent graph** that owns the observable outcome.
- **Do not load every prompt**, every department pack entry, or every skill into context.
- Load the five control prompts once per run, then only selected capabilities and required handoffs.
- Prefer a composite scenario when it already expresses the complete workflow; otherwise start from one primary prompt.
- Invoke a skill through `MD-192` and `MD-196` only when its genuine capability is needed. Installed does not mean required.
- Discover, install, or create a skill through `MD-193` to `MD-195` only when native execution cannot satisfy the acceptance criteria cleanly.
- Loop through `MD-197` and `MD-198` only for a finite queue or measurable improvement. Stop on verified success, plateau, budget exhaustion, stale evidence, lost authority, or human stop.
- Keep skill output quarantined until the routed verification step accepts the exact artifact.
- Route code-native illustrations, vectors, infographics, diagrams, and presentation assets through the exact local `visual-assets` skill when genuinely required; route Strudel music code through `strudel`.
- Do not imply publication, sending, submission, deployment, merging, purchasing, or other external action from a draft or plan.
- For every genuine planning/execution pair, present the completed plan for user review, incorporate requested changes, re-verify and re-freeze it, request review again, then ask for explicit consent to invoke only the exact execution twin declared in `paired_prompt_id`. Never substitute another executor or infer consent from the original task.

### Paired plan review workflow

- Inspect the exact reciprocal twin with `python prompts/tools/md.py pair-status <PLANNING-MD-ID> --handoff-ready --review-status approved`.
- Requested changes invalidate prior approval and consent; revise, re-freeze under a new hash, and request review again.
- Execute only after the user approves the final frozen plan and explicitly consents to the named exact execution twin.

### Core locations

- Prompt bodies: `prompts/prompts/`
- Prompt catalog: `prompts/catalog.json`
- Scenario catalog: `prompts/SCENARIO_CATALOG.json`
- Execution guide: `prompts/PROMPT_EXECUTION_ORDER.md`
- Skill registry: `prompts/skill_registry.json`
- CLI: `prompts/tools/md.py`
- Manuals: `prompts/MANUALS.md` and `prompts/docs/`

### Honest completion

A route is not complete merely because a prompt or skill ran. Completion requires the selected prompt's task-specific criteria, `=VERIFY:{id}` evidence, explicit unknowns and residuals, and the applicable human approval or external-action gate.
<!-- END MD MANAGED GUIDANCE -->
