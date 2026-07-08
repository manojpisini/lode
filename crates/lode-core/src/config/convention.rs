use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConventionConfig {
    pub folder_case: String,
    pub file_case: String,
    pub default_case: String,
    pub enforce: bool,
    pub exclude: Vec<String>,
    pub protected_prefixes: Vec<String>,
    pub prefix_map: HashMap<String, String>,
}

impl Default for ConventionConfig {
    fn default() -> Self {
        Self {
            folder_case: "snake_case".to_string(),
            file_case: "snake_case".to_string(),
            default_case: "snake_case".to_string(),
            enforce: false,
            exclude: Vec::new(),
            protected_prefixes: Vec::new(),
            prefix_map: HashMap::new(),
        }
    }
}
