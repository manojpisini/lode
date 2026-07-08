use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackConfig {
    pub languages: Vec<String>,
    pub framework: Option<String>,
    pub indent: String,
    pub line_width: usize,
    pub comment_style: String,
    pub package_manager: Option<String>,
}

impl Default for StackConfig {
    fn default() -> Self {
        Self {
            languages: vec!["rust".to_string()],
            framework: None,
            indent: "4".to_string(),
            line_width: 100,
            comment_style: "//".to_string(),
            package_manager: None,
        }
    }
}
