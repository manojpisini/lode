use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpConfig {
    pub enabled: bool,
    pub default_transport: String,
    pub http_port: u16,
    pub http_host: String,
    pub auth_token_env: Option<String>,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_transport: "stdio".to_string(),
            http_port: 8080,
            http_host: "127.0.0.1".to_string(),
            auth_token_env: None,
        }
    }
}
