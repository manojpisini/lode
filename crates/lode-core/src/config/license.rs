use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicenseConfig {
    pub kind: String,
    pub copyright_holder: Option<String>,
    pub year: Option<u32>,
    pub auto_insert: bool,
    pub file_header: bool,
}

impl Default for LicenseConfig {
    fn default() -> Self {
        Self {
            kind: "MIT".to_string(),
            copyright_holder: None,
            year: None,
            auto_insert: true,
            file_header: true,
        }
    }
}
