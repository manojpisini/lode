# Template Reference

## Template Syntax

LODE uses `{{ }}` syntax for template rendering.

### Variable Substitution

```
Hello {{ name }}!
Project: {{ project }}
```

### Filters

```
{{ name | upper }}
{{ description | lower }}
{{ url | urlencode }}
{{ count | default("0") }}
{{ title | trim }}
```

### Conditionals

```
{% if feature.enabled %}
Enabled feature
{% endif %}

{% if platform == "windows" %}
Windows-specific
{% else %}
Cross-platform
{% endif %}
```

### Loops

```
{% for item in items %}
- {{ item }}
{% endfor %}
```

### Includes

```
{% include "partials/header.md" %}
```

### Extends / Blocks

```
{% extends "base.md" %}
{% block content %}
Custom content here
{% endblock %}
```

## Template Bundles

Template bundles are self-contained directories with a TOML manifest and `assets/` directory.

### Bundle Structure

```
my-template/
├── my-template.toml    # Manifest
├── assets/             # Binary/copied assets
│   ├── logo.png
│   └── config.json
└── files/              # Inline rendered files (optional)
```

### Manifest Format

```toml
[meta]
id = "my-template"
name = "My Template"
version = "0.1.0"
kind = "project"
description = "A sample project template"

[variables]
project = { type = "string", prompt = "Project name", default = "my-project" }
feature_x = { type = "bool", prompt = "Enable feature X?", default = true }

[[files]]
path = "src/main.rs"
content = '''
fn main() {
    println!("Hello, {{ project }}!");
}
'''

[[assets]]
source = "logo.png"
destination = "assets/logo.png"

[[directories]]
path = "src"
```

### Bundle Commands

```
lode template-bundle apply <path>        # Apply a bundle
lode template-bundle capture <src> <dst>  # Capture directory as bundle
lode template-bundle preview <source>     # Preview capture
lode template-bundle list                 # List bundles
lode template-bundle show <path>          # Show manifest
lode template-bundle validate <path>      # Validate bundle
lode template-bundle verify <path>        # Verify assets exist
```
