# Project Setup Guide

## Overview

After installing LODE and running `lode setup`, you can configure your project.

## Configuring a Project

### View Current Configuration

```bash
lode config show
```

Shows merged configuration (defaults + global + project).

### Set Configuration Values

```bash
lode config set identity.author "Your Name"
lode config set git.initial_branch main
lode config set convention.file_case snake_case
```

### Validate Configuration

```bash
lode config validate
```

Checks for schema or value issues.

### Using Profiles

```bash
lode profile list            # List available profiles
lode profile show rust        # Show profile details
lode profile use rust         # Apply profile to current project
```

## Project Types

Create projects with different profiles:

```bash
lode init my-app --profile rust-lib
lode init my-service --profile rust-cli
```

Add components to existing projects:

```bash
lode add ci
lode add docker
lode add agent
```

## Managing Environment Variables

```bash
lode env add DATABASE_URL --secret
lode env check                # Check against .env.example
lode env sync                 # Sync .env with config
```

## Next Steps

- Learn about [convention checks](quick_start.md#2-check-conventions)
- Set up [secret scanning](secrets.md)
- Configure [templates](templates.md) for your project
