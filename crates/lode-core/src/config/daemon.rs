use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub enabled: bool,
    pub idle_timeout_s: u64,
    pub debounce_ms: u64,
    pub watch_rename: bool,
    pub watch_headers: bool,
    pub watch_path_sync: bool,
    pub watch_env_drift: bool,
    pub watch_license: bool,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            idle_timeout_s: 300,
            debounce_ms: 150,
            watch_rename: true,
            watch_headers: true,
            watch_path_sync: true,
            watch_env_drift: true,
            watch_license: true,
        }
    }
}
