use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub auto_snapshot: bool,
    pub snapshot_history: usize,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_snapshot: true,
            snapshot_history: 10,
        }
    }
}
