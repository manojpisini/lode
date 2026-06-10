use serde::{Deserialize, Serialize};

use crate::{LodeError, Result};

pub const SCHEMA_VERSION: u32 = 3;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LodeConfig {
    pub schema_version: u32,
    pub identity: IdentityConfig,
    pub convention: ConventionConfig,
    pub scaffold: ScaffoldConfig,
    pub git: GitConfig,
    #[serde(default)]
    pub active_profile: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityConfig {
    pub author: String,
    pub email: String,
    pub org: String,
    pub license: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConventionConfig {
    pub default_case: String,
    pub protected_prefixes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScaffoldConfig {
    pub always_dirs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitConfig {
    pub auto_init: bool,
    pub initial_branch: String,
    pub initial_commit: bool,
    pub initial_commit_msg: String,
}

pub fn default_config() -> LodeConfig {
    LodeConfig {
        schema_version: SCHEMA_VERSION,
        identity: IdentityConfig {
            author: "Your Name".to_string(),
            email: "you@example.com".to_string(),
            org: "namespace".to_string(),
            license: "MIT OR Apache-2.0".to_string(),
        },
        convention: ConventionConfig {
            default_case: "snake_case".to_string(),
            protected_prefixes: vec![
                "_ref_".to_string(),
                "_ctx_".to_string(),
                ".lode".to_string(),
            ],
        },
        scaffold: ScaffoldConfig {
            always_dirs: vec![
                "src".to_string(),
                "tests".to_string(),
                "docs".to_string(),
                "scripts".to_string(),
                "assets".to_string(),
                "_ref_".to_string(),
                "_ctx_".to_string(),
                ".lode".to_string(),
            ],
        },
        git: GitConfig {
            auto_init: true,
            initial_branch: "main".to_string(),
            initial_commit: true,
            initial_commit_msg: "chore: scaffold [{org}/{project}]".to_string(),
        },
        active_profile: None,
    }
}

pub fn validate_schema(config: &LodeConfig) -> Result<()> {
    if config.schema_version == SCHEMA_VERSION {
        Ok(())
    } else {
        Err(LodeError::SchemaMismatch {
            expected: SCHEMA_VERSION,
            found: config.schema_version,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_round_trips_through_toml() {
        let config = default_config();
        let encoded = toml::to_string_pretty(&config).unwrap();
        let decoded: LodeConfig = toml::from_str(&encoded).unwrap();

        assert_eq!(decoded, config);
    }

    #[test]
    fn schema_mismatch_is_reported() {
        let mut config = default_config();
        config.schema_version = 2;

        let error = validate_schema(&config).unwrap_err();

        assert!(matches!(
            error,
            LodeError::SchemaMismatch {
                expected: SCHEMA_VERSION,
                found: 2
            }
        ));
    }
}
