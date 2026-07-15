# Snippet Management Guide

## Overview

Snippets are reusable code fragments stored in `.lode/snippets/`.

## Adding a Snippet

```bash
lode snippet add
```

This opens an editor where you can define:
- Title
- Description
- Language
- Tags
- Content

## Listing Snippets

```bash
lode snippet list
```

## Searching Snippets

```bash
lode snippet search "hello world"
lode snippet search --language rust
lode snippet search --tag beginner
```

## Exporting Snippets

Export to editor-native formats:

```bash
lode snippet export --format vscode
lode snippet export --format zed
```

This writes snippets to the format expected by your editor.

## Removing a Snippet

```bash
lode snippet remove hello_world
```

## Snippet Organization

Snippets are organized by language:

```
.lode/snippets/
├── rust/
├── python/
├── javascript/
└── common/
```
