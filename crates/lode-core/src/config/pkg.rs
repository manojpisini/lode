use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PkgConfig {
    pub name: String,
    pub version: String,
    pub edition: String,
    pub description: Option<String>,
    pub repository: Option<String>,
    pub publish: bool,
}

impl Default for PkgConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: "0.1.0".to_string(),
            edition: "2021".to_string(),
            description: None,
            repository: None,
            publish: false,
        }
    }
}
