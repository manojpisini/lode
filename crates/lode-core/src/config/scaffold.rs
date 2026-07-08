use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateMapping {
    pub template: String,
    pub dest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionalComponent {
    pub name: String,
    pub description: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScaffoldConfig {
    pub always_dirs: Vec<String>,
    pub always_files: Vec<TemplateMapping>,
    pub optional: Vec<OptionalComponent>,
}

impl Default for ScaffoldConfig {
    fn default() -> Self {
        Self {
            always_dirs: vec!["src".to_string(), "tests".to_string(), "docs".to_string()],
            always_files: Vec::new(),
            optional: Vec::new(),
        }
    }
}
