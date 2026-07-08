use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeConfig {
    pub timezone: Option<String>,
    pub date_format: String,
    pub time_format: String,
    pub timestamp_format: String,
}

impl Default for TimeConfig {
    fn default() -> Self {
        Self {
            timezone: None,
            date_format: "%Y-%m-%d".to_string(),
            time_format: "%H:%M:%S".to_string(),
            timestamp_format: "%Y-%m-%dT%H:%M:%S%.3fZ".to_string(),
        }
    }
}
