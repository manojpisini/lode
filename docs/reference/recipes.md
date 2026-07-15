# Recipe Reference

## Recipe Format

Recipes define composable project components.

```toml
[meta]
name = "docker-setup"
version = "0.1.0"
description = "Adds Docker configuration to a project"

[dependencies]
rust = ["core"]

[[files]]
path = "Dockerfile"
content = '''
FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/{{project}} /usr/local/bin/
CMD ["{{project}}"]
'''

[[files]]
path = ".dockerignore"
content = "target/\n.git/\n"
```

## Recipe Commands

| Command | Description |
|---------|-------------|
| `lode recipe new <name>` | Create a new recipe |
| `lode recipe list` | List available recipes |
| `lode recipe apply <name>` | Apply a recipe to current project |
| `lode recipe compose <name1> <name2>` | Combine two recipes |
| `lode recipe info <name>` | Show recipe details |

## Recipe Composition

Recipes can be composed:

```
lode recipe compose docker-setup ci-workflow
```

This merges both recipes, applying files from each. Conflicts are resolved by prompting or using explicit override policies.
