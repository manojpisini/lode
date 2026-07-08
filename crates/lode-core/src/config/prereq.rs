use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrereqCheck {
    pub name: String,
    pub command: String,
    pub min_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrereqConfig {
    pub checks: Vec<PrereqCheck>,
    pub auto_install: bool,
}

impl Default for PrereqConfig {
    fn default() -> Self {
        Self {
            checks: Vec::new(),
            auto_install: false,
        }
    }
}
