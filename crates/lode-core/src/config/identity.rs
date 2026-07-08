use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityConfig {
    pub author: String,
    pub name: String,
    pub email: String,
    pub org: String,
    pub url: String,
    pub license: String,
}

impl Default for IdentityConfig {
    fn default() -> Self {
        Self {
            author: "Your Name".to_string(),
            name: String::new(),
            email: "you@example.com".to_string(),
            org: String::new(),
            url: String::new(),
            license: "MIT OR Apache-2.0".to_string(),
        }
    }
}
