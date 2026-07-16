# Quick Start Guide

## 1. Initialize a Project

```bash
lode init my-project
cd my-project
```

This creates a basic LODE project structure with `.lode/` configuration.

## 2. Check Conventions

```bash
lode check
```

Checks that all files follow naming conventions (default: `snake_case`).

## 3. Scan for Secrets

```bash
lode scan secrets
```

Scans the project for API keys, tokens, and other secrets.

## 4. Generate Context

```bash
lode context compile
```

Compiles project context for AI agent consumption.

## 5. Sync Agent Configuration

```bash
lode agent sync
```

Generates agent configuration files (AGENTS.md, CLAUDE.md).

## 6. View Project Info

```bash
lode info
```

Shows project metadata and health status.

## Next Steps

- Explore [templates](templates.md) for scaffolding
- Set up [plugins](plugins.md) for custom hooks
- Use [snippets](snippets.md) for reusable code
- Run [recipes](recipes.md) to add components
