use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RenderContext {
    values: BTreeMap<String, String>,
}

impl RenderContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.values.insert(key.into(), value.into());
        self
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }
}

pub fn render_template(input: &str, context: &RenderContext) -> String {
    let mut output = input.to_string();
    for (key, value) in &context.values {
        output = output.replace(&format!("{{{{ {} }}}}", key), value);
        output = output.replace(&format!("{{{{{} }}}}", key), value);
        output = output.replace(&format!("{{{{ {} }}}}", key), value);
        output = output.replace(&format!("{{{{{}}}}}", key), value);
    }
    output
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
