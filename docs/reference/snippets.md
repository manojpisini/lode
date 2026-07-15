# Snippet Reference

## Snippet Format

Snippets are stored as individual files in the snippets directory.

```
.lode/snippets/
├── rust/
│   ├── hello_world.md
│   └── fibonacci.md
├── python/
│   └── flask_app.md
└── common/
    └── git_ignore.md
```

### Snippet Content

```markdown
---
title: Hello World in Rust
description: A simple hello world program
language: rust
tags: [beginner, hello-world]
---

fn main() {
    println!("Hello, world!");
}
```

## Snippet Commands

| Command | Description |
|---------|-------------|
| `lode snippet add` | Add a new snippet |
| `lode snippet list` | List all snippets |
| `lode snippet search <query>` | Search snippets by keyword |
| `lode snippet search --language rust` | Filter by language |
| `lode snippet search --tag beginner` | Filter by tag |
| `lode snippet export --format vscode` | Export to VS Code format |
| `lode snippet export --format zed` | Export to Zed format |
| `lode snippet remove <name>` | Remove a snippet |

## Export Formats

### VS Code

```json
{
  "Hello World in Rust": {
    "prefix": "hello",
    "body": ["fn main() {", "    println!(\"Hello, world!\");", "}"],
    "description": "A simple hello world program"
  }
}
```

### Zed

```json
{
  "hello_world": {
    "prefix": "hello",
    "body": ["fn main() {", "    println!(\"Hello, world!\");", "}"],
    "description": "A simple hello world program"
  }
}
```
