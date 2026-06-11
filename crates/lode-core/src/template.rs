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
    let mut output = render_blocks(input, context);
    for (key, value) in &context.values {
        output = output.replace(&format!("{{{{ {} }}}}", key), value);
        output = output.replace(&format!("{{{{{} }}}}", key), value);
        output = output.replace(&format!("{{{{ {} }}}}", key), value);
        output = output.replace(&format!("{{{{{}}}}}", key), value);
    }
    output
}

fn render_blocks(input: &str, context: &RenderContext) -> String {
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
                output.push(render_template(&body, context));
            }
            index = next;
        } else if let Some((binding, list_name)) = for_tag(trimmed) {
            let (body, next) = collect_block(&lines, index + 1, "endfor");
            if let Some(items) = context.get_list(list_name) {
                for item in items {
                    let child = context.clone().with(binding, item);
                    output.push(render_template(&body, &child));
                }
            }
            index = next;
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
}
