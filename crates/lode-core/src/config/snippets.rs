use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snippet {
    pub prefix: String,
    pub body: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnippetsConfig {
    pub enabled: bool,
    pub dir: Option<String>,
    pub snippets: HashMap<String, Snippet>,
}

impl Default for SnippetsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            dir: None,
            snippets: HashMap::new(),
        }
    }
}
