use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolchainConfig {
    pub rust_version: Option<String>,
    pub clippy_lints: Option<String>,
    pub rustfmt_edition: Option<String>,
    pub target: Option<String>,
}

impl Default for ToolchainConfig {
    fn default() -> Self {
        Self {
            rust_version: None,
            clippy_lints: None,
            rustfmt_edition: None,
            target: None,
        }
    }
}
