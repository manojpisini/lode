use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RenderContext {
    values: BTreeMap<String, String>,
    lists: BTreeMap<String, Vec<String>>,
}

impl RenderContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.values.insert(key.into(), value.into());
        self
    }

    pub fn with_list<I, S>(mut self, key: impl Into<String>, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.lists
            .insert(key.into(), values.into_iter().map(Into::into).collect());
        self
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    pub fn get_list(&self, key: &str) -> Option<&[String]> {
        self.lists.get(key).map(Vec::as_slice)
    }
}

pub fn render_template(input: &str, context: &RenderContext) -> String {
    render_template_with_resolver(input, context, &|_| None)
}

pub fn render_template_with_resolver<'a>(
    input: &str,
    context: &RenderContext,
    resolver: &(dyn Fn(&str) -> Option<String> + 'a),
) -> String {
    render_template_inner(input, context, resolver, 0)
}

fn render_template_inner<'a>(
    input: &str,
    context: &RenderContext,
    resolver: &(dyn Fn(&str) -> Option<String> + 'a),
    include_depth: usize,
) -> String {
    if include_depth < 16 {
        if let Some(parent) = extends_template(input) {
            if let Some(parent_source) = resolver(parent) {
                let blocks = extract_template_blocks(input);
                let inherited = apply_block_overrides(&parent_source, &blocks);
                return render_template_inner(&inherited, context, resolver, include_depth + 1);
            }
        }
    }
    render_variables(
        &render_blocks(input, context, resolver, include_depth),
        context,
    )
}

fn render_variables(input: &str, context: &RenderContext) -> String {
    let mut output = String::new();
    let mut rest = input;
    while let Some(start) = rest.find("{{") {
        output.push_str(&rest[..start]);
        let after_start = &rest[start + 2..];
        let Some(end) = after_start.find("}}") else {
            output.push_str(&rest[start..]);
            return output;
        };
        let expression = after_start[..end].trim();
        output.push_str(&render_expression(expression, context));
        rest = &after_start[end + 2..];
    }
    output.push_str(rest);
    output
}

fn render_expression(expression: &str, context: &RenderContext) -> String {
    let mut parts = expression.split('|').map(str::trim);
    let Some(key) = parts.next() else {
        return String::new();
    };
    let mut value = context.get(key).unwrap_or_default().to_string();
    for filter in parts {
        value = apply_filter(&value, filter);
    }
    value
}

fn apply_filter(value: &str, filter: &str) -> String {
    match filter {
        "upper" => value.to_ascii_uppercase(),
        "lower" => value.to_ascii_lowercase(),
        "snake" | "snake_case" => slug_to_ident(value),
        "kebab" | "kebab_case" => slug_to_ident(value).replace('_', "-"),
        "pascal" | "pascal_case" => slug_to_class(value),
        "camel" | "camel_case" => {
            let pascal = slug_to_class(value);
            let mut chars = pascal.chars();
            match chars.next() {
                Some(first) => first.to_ascii_lowercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        }
        "title" => value
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
        "urlencode" | "url_encode" => url_encode(value),
        _ => value.to_string(),
    }
}

fn url_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn render_blocks<'a>(
    input: &str,
    context: &RenderContext,
    resolver: &(dyn Fn(&str) -> Option<String> + 'a),
    include_depth: usize,
) -> String {
    let lines = input.lines().collect::<Vec<_>>();
    let mut output = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let trimmed = lines[index].trim();
        if let Some(name) = block_tag_value(trimmed, "if") {
            let (body, next) = collect_block(&lines, index + 1, "endif");
            if context
                .get(name)
                .map(|value| !value.is_empty() && value != "false")
                .unwrap_or(false)
            {
                output.push(render_template_inner(
                    &body,
                    context,
                    resolver,
                    include_depth,
                ));
            }
            index = next;
        } else if let Some((binding, list_name)) = for_tag(trimmed) {
            let (body, next) = collect_block(&lines, index + 1, "endfor");
            if let Some(items) = context.get_list(list_name) {
                for item in items {
                    let child = context.clone().with(binding, item);
                    output.push(render_template_inner(
                        &body,
                        &child,
                        resolver,
                        include_depth,
                    ));
                }
            }
            index = next;
        } else if let Some(include) = include_tag(trimmed) {
            if include_depth < 16 {
                if let Some(source) = resolver(include) {
                    output.push(render_template_inner(
                        &source,
                        context,
                        resolver,
                        include_depth + 1,
                    ));
                }
            }
            index += 1;
        } else {
            output.push(lines[index].to_string());
            index += 1;
        }
    }
    let mut rendered = output.join("\n");
    if input.ends_with('\n') {
        rendered.push('\n');
    }
    rendered
}

fn block_tag_value<'a>(trimmed: &'a str, name: &str) -> Option<&'a str> {
    let inner = trimmed.strip_prefix("{%")?.strip_suffix("%}")?.trim();
    inner.strip_prefix(name)?.trim().split_whitespace().next()
}

fn for_tag(trimmed: &str) -> Option<(&str, &str)> {
    let inner = trimmed.strip_prefix("{%")?.strip_suffix("%}")?.trim();
    let rest = inner.strip_prefix("for")?.trim();
    let (binding, list_name) = rest.split_once(" in ")?;
    Some((binding.trim(), list_name.trim()))
}

fn include_tag(trimmed: &str) -> Option<&str> {
    let inner = trimmed.strip_prefix("{%")?.strip_suffix("%}")?.trim();
    let include = inner.strip_prefix("include")?.trim();
    quoted_tag_value(include)
}

fn extends_template(input: &str) -> Option<&str> {
    input
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .and_then(extends_tag)
}

fn extends_tag(trimmed: &str) -> Option<&str> {
    let inner = trimmed.strip_prefix("{%")?.strip_suffix("%}")?.trim();
    let parent = inner.strip_prefix("extends")?.trim();
    quoted_tag_value(parent)
}

fn quoted_tag_value(value: &str) -> Option<&str> {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
}

fn extract_template_blocks(input: &str) -> BTreeMap<String, String> {
    let lines = input.lines().collect::<Vec<_>>();
    let mut blocks = BTreeMap::new();
    let mut index = 0;
    while index < lines.len() {
        let trimmed = lines[index].trim();
        if let Some(name) = block_tag_value(trimmed, "block") {
            let (body, next) = collect_block(&lines, index + 1, "endblock");
            blocks.insert(name.to_string(), body);
            index = next;
        } else {
            index += 1;
        }
    }
    blocks
}

fn apply_block_overrides(parent: &str, blocks: &BTreeMap<String, String>) -> String {
    let lines = parent.lines().collect::<Vec<_>>();
    let mut output = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let trimmed = lines[index].trim();
        if let Some(name) = block_tag_value(trimmed, "block") {
            let (default_body, next) = collect_block(&lines, index + 1, "endblock");
            output.push(blocks.get(name).cloned().unwrap_or(default_body));
            index = next;
        } else {
            output.push(lines[index].to_string());
            index += 1;
        }
    }
    let mut rendered = output.join("\n");
    if parent.ends_with('\n') {
        rendered.push('\n');
    }
    rendered
}

fn collect_block(lines: &[&str], start: usize, end_tag: &str) -> (String, usize) {
    let mut depth = 0usize;
    let mut body = Vec::new();
    let mut index = start;
    while index < lines.len() {
        let trimmed = lines[index].trim();
        if trimmed.starts_with("{% if ") || trimmed.starts_with("{% for ") {
            depth += 1;
        }
        if trimmed == format!("{{% {end_tag} %}}") {
            if depth == 0 {
                return (body.join("\n"), index + 1);
            }
            depth -= 1;
        }
        body.push(lines[index].to_string());
        index += 1;
    }
    (body.join("\n"), index)
}

pub fn slug_to_ident(slug: &str) -> String {
    let mut ident = String::new();
    for ch in slug.chars() {
        if ch.is_ascii_alphanumeric() {
            ident.push(ch.to_ascii_lowercase());
        } else if !ident.ends_with('_') {
            ident.push('_');
        }
    }
    ident.trim_matches('_').to_string()
}

pub fn slug_to_class(slug: &str) -> String {
    let mut class = String::new();
    let mut upper_next = true;
    for ch in slug.chars() {
        if ch.is_ascii_alphanumeric() {
            if upper_next {
                class.push(ch.to_ascii_uppercase());
                upper_next = false;
            } else {
                class.push(ch);
            }
        } else {
            upper_next = true;
        }
    }
    if class.is_empty() {
        "App".to_string()
    } else {
        class
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_conditionals_and_loops() {
        let context = RenderContext::new()
            .with("project", "demo")
            .with("has_ci", "true")
            .with_list("dirs", ["src", "tests"]);

        let output = render_template(
            "# {{ project }}\n{% if has_ci %}\nci=yes\n{% endif %}\n{% for dir in dirs %}\n- {{ dir }}\n{% endfor %}\n",
            &context,
        );

        assert!(output.contains("# demo"));
        assert!(output.contains("ci=yes"));
        assert!(output.contains("- src"));
        assert!(output.contains("- tests"));
    }

    #[test]
    fn skips_false_conditionals() {
        let context = RenderContext::new().with("enabled", "false");

        let output = render_template(
            "before\n{% if enabled %}\nhidden\n{% endif %}\nafter\n",
            &context,
        );

        assert!(output.contains("before"));
        assert!(output.contains("after"));
        assert!(!output.contains("hidden"));
    }

    #[test]
    fn renders_filter_pipelines() {
        let context = RenderContext::new().with("project", "My Demo App");

        let output = render_template(
            "{{ project | upper }}\n{{ project | snake }}\n{{ project | kebab }}\n{{ project | pascal }}\n{{ project | camel }}\n",
            &context,
        );

        assert!(output.contains("MY DEMO APP"));
        assert!(output.contains("my_demo_app"));
        assert!(output.contains("my-demo-app"));
        assert!(output.contains("MyDemoApp"));
        assert!(output.contains("myDemoApp"));
    }

    #[test]
    fn renders_urlencode_filter() {
        let context = RenderContext::new().with("license", "MIT OR Apache-2.0");

        let output = render_template(
            "https://img.shields.io/badge/license-{{ license | urlencode }}-blue.svg",
            &context,
        );

        assert_eq!(
            output,
            "https://img.shields.io/badge/license-MIT%20OR%20Apache-2.0-blue.svg"
        );
    }

    #[test]
    fn renders_includes_with_resolver() {
        let context = RenderContext::new().with("project", "demo");

        let output = render_template_with_resolver(
            "before\n{% include \"partials/name.txt\" %}\nafter\n",
            &context,
            &|name| match name {
                "partials/name.txt" => Some("project={{ project | upper }}".to_string()),
                _ => None,
            },
        );

        assert!(output.contains("before"));
        assert!(output.contains("project=DEMO"));
        assert!(output.contains("after"));
    }

    #[test]
    fn include_recursion_is_bounded() {
        let context = RenderContext::new();

        let output =
            render_template_with_resolver("{% include \"loop\" %}\nend\n", &context, &|name| {
                (name == "loop").then(|| "{% include \"loop\" %}".to_string())
            });

        assert!(output.lines().count() < 20);
    }

    #[test]
    fn renders_extends_and_blocks() {
        let context = RenderContext::new().with("project", "demo");

        let output = render_template_with_resolver(
            "{% extends \"base.md\" %}\n{% block title %}\n{{ project | title }}\n{% endblock %}\n{% block body %}\nchild body\n{% endblock %}\n",
            &context,
            &|name| match name {
                "base.md" => Some("#\n{% block title %}\nUntitled\n{% endblock %}\n{% block body %}\ndefault\n{% endblock %}\nfooter\n".to_string()),
                _ => None,
            },
        );

        assert!(output.contains("Demo"));
        assert!(output.contains("child body"));
        assert!(output.contains("footer"));
        assert!(!output.contains("default"));
    }

    #[test]
    fn extends_keeps_parent_default_blocks() {
        let context = RenderContext::new();

        let output = render_template_with_resolver(
            "{% extends \"base.md\" %}\n{% block title %}\nChild\n{% endblock %}\n",
            &context,
            &|name| {
                match name {
                "base.md" => Some("{% block title %}\nParent\n{% endblock %}\n{% block body %}\ndefault body\n{% endblock %}\n".to_string()),
                _ => None,
            }
            },
        );

        assert!(output.contains("Child"));
        assert!(output.contains("default body"));
    }
}
