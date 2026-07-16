# Template Bundle Reference

## Overview

Template bundles are multi-file template sets with metadata, variables, and validation. They can be captured from existing directories or authored manually.

## Bundle Manifest Format

A template bundle is a directory containing a TOML manifest and template files:

```
my-bundle/
├── my-bundle.toml          # Manifest
├── templates/
│   ├── src/
│   │   └── main.rs.hbs     # Handlebars template
│   └── Cargo.toml.hbs
└── assets/
    └── logo.png            # Static assets
```

### Manifest Structure

```toml
schema_version = 1
name = "my-bundle"
version = "1.0.0"
description = "My project template"
kind = "project"             # file, bundle, feature, project, overlay, organization

[variables]
  [variables.name]
  type = "string"
  required = true
  description = "Project name"

  [variables.author]
  type = "string"
  required = true
  description = "Author name"

[directories]
  [directories.src]
  path = "src"
  mode = "create"

[files]
  [files.main]
  path = "src/main.rs"
  content = "templates/src/main.rs.hbs"
  render = true

  [files.config]
  path = "Cargo.toml"
  content = "templates/Cargo.toml.hbs"
  render = true

[assets]
  [assets.logo]
  source = "assets/logo.png"
  destination = "logo.png"
```

### Template Kinds

| Kind | Description |
|------|-------------|
| `file` | Single file template |
| `bundle` | Multi-file template set |
| `feature` | Additive feature (e.g., CI, Docker) |
| `project` | Complete project scaffold |
| `overlay` | Layer over existing project |
| `organization` | Organization-wide template pack |

### Ownership Types

| Owner | Behavior |
|-------|----------|
| `seed` | Written once at init; never updated |
| `managed` | Overwritten on sync |
| `merged` | Merged with existing content |
| `derived` | Generated from other files |
| `protected` | Never modified |
| `ephemeral` | Not persisted (generated at runtime) |
| `vendored` | Third-party, overwritten on update |

### Overwrite Policies

| Policy | Behavior |
|--------|----------|
| `error` | Fail if destination exists |
| `skip` | Keep existing, skip write |
| `prompt` | Ask user what to do |
| `replace` | Always overwrite |
| `merge` | Merge with existing content |
| `three_way` | Three-way merge |
| `backup` | Backup existing, then write |
| `version` | Create numbered version |

## Commands

```bash
# Capture a directory as a template bundle
lode template-bundle capture ./source ./bundle --mode project

# Preview what would be captured
lode template-bundle preview ./source --mode project

# Apply a template bundle
lode template-bundle apply ./bundle --variables "name=myapp,author=me"

# List available bundles
lode template-bundle list

# Show bundle manifest
lode template-bundle show ./bundle

# Validate bundle manifest
lode template-bundle validate ./bundle

# Verify bundle assets exist
lode template-bundle verify ./bundle
```

## Template Syntax

Uses Handlebars-style `{{ variable }}` substitution. Supports:

- `{{ name }}` — variable substitution
- `{{#if condition}}...{{/if}}` — conditionals
- `{{#each list}}...{{/each}}` — iteration
- `{{ uppercase name }}` — filter helpers
