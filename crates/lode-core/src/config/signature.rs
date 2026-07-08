use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionMarkers {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignatureConfig {
    pub enabled: bool,
    pub auto_insert: bool,
    pub auto_update_date: bool,
    pub include_path: bool,
    pub include_hash: bool,
    pub include_license: bool,
    pub separator_char: char,
    pub section_markers: SectionMarkers,
    pub comment_styles: HashMap<String, String>,
}

impl Default for SignatureConfig {
    fn default() -> Self {
        let mut comment_styles = HashMap::new();
        comment_styles.insert("rust".to_string(), "//".to_string());
        comment_styles.insert("python".to_string(), "#".to_string());
        comment_styles.insert("javascript".to_string(), "//".to_string());
        comment_styles.insert("typescript".to_string(), "//".to_string());

        Self {
            enabled: true,
            auto_insert: true,
            auto_update_date: true,
            include_path: true,
            include_hash: false,
            include_license: true,
            separator_char: '=',
            section_markers: SectionMarkers {
                start: " --- ".to_string(),
                end: " --- ".to_string(),
            },
            comment_styles,
        }
    }
}
