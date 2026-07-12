use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolchainConfig {
    pub rust_version: Option<String>,
    pub clippy_lints: Option<String>,
    pub rustfmt_edition: Option<String>,
    pub target: Option<String>,
}
