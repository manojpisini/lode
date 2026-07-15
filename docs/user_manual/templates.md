# Template Usage Guide

## Overview

LODE supports two template systems:
1. **Project templates** — Used during `lode init` for scaffolding new projects
2. **Template bundles** — Self-contained directories with manifests and assets

## Using Templates

### List Available Templates

```bash
lode template list
```

### Show Template Details

```bash
lode template show default
```

### Initialize with a Template

```bash
lode init my-project --template my-template
```

## Template Bundles

Template bundles are the modern template system. They are self-contained directories with a TOML manifest and an `assets/` directory.

### Create a Bundle from an Existing Project

```bash
lode template-bundle capture ./my-project ./bundles/my-project-bundle
```

### Apply a Bundle

```bash
lode template-bundle apply ./bundles/my-project-bundle
```

### Apply with Variables

```bash
lode template-bundle apply ./bundles/my-project-bundle project=my-app version=0.2.0
```

### Dry Run

```bash
lode template-bundle apply ./bundles/my-project-bundle --dry-run
```

### Validate a Bundle

```bash
lode template-bundle validate ./bundles/my-project-bundle
```

### Verify Bundle Assets

```bash
lode template-bundle verify ./bundles/my-project-bundle
```

## Capture Modes

| Mode | Description |
|------|-------------|
| `minimal` | Only essential configuration files |
| `source` | Source code and configuration (default) |
| `development` | Source + tools, tests, docs |
| `complete` | Everything except build artifacts |

```bash
lode template-bundle capture ./project ./bundle --mode complete
```
