use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
    pub secret: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvValidation {
    pub required: Vec<String>,
    pub warn_missing: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvConfig {
    pub auto_create: bool,
    pub runtime_lock: bool,
    pub vars: Vec<EnvVar>,
    pub validation: EnvValidation,
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            auto_create: true,
            runtime_lock: true,
            vars: Vec::new(),
            validation: EnvValidation::default(),
        }
    }
}
