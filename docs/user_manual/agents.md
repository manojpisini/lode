# Agent Configuration Guide

## Overview

LODE helps configure AI agents for project assistance.

## Syncing Agent Configuration

```bash
lode agent sync
```

Generates:
- `AGENTS.md` — Agent guidance document
- `CLAUDE.md` — Claude-specific configuration
- Agent policy files

## Loading an Agent Plan

```bash
lode agent plan
```

Displays the current agent execution plan.

## Generating Agent Policy

```bash
lode agent policy
```

Generates canonical agent policy files defining what the agent can and cannot do.

## Context Compilation

```bash
lode context compile
```

Compiles project context (file tree, config, rules) into a single document for agent consumption.

```bash
lode context compile --budget 4000
```

Limits output to approximately 4000 tokens.

## Agent Simulation

```bash
lode agent-sim build --intent "Add tests for new module"
```

Simulates an agent executing an intent against the project, producing a trace of actions.
