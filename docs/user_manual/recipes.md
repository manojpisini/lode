# Recipe Management Guide

## Overview

Recipes define reusable project components (Docker setup, CI config, etc.) that can be applied to any project.

## Creating a Recipe

```bash
lode recipe new docker-setup
```

This creates a recipe template in `.lode/recipes/docker-setup/`.

## Listing Recipes

```bash
lode recipe list
```

## Applying a Recipe

```bash
lode recipe apply docker-setup
```

This renders the recipe's files into the current project.

## Composing Recipes

Combine multiple recipes into one:

```bash
lode recipe compose docker-setup ci-workflow
```

The composed recipe merges files from both sources.

## Recipe Structure

```
.lode/recipes/
└── my-recipe/
    ├── recipe.toml     # Recipe metadata and file definitions
    └── files/          # Template files (rendered with {{ }} syntax)
        └── Dockerfile
```
