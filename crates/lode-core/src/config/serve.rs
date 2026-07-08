use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServeConfig {
    pub refresh_ms: u64,
    pub default_pane: String,
    pub theme: String,
    pub show_registry: bool,
    pub border_style: String,
}

impl Default for ServeConfig {
    fn default() -> Self {
        Self {
            refresh_ms: 1000,
            default_pane: "status".to_string(),
            theme: "dark".to_string(),
            show_registry: true,
            border_style: "rounded".to_string(),
        }
    }
}
