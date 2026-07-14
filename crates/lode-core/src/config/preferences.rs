use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreferencesConfig {
    pub architecture: ArchitecturePrefs,
    pub testing: TestingPrefs,
    pub agents: AgentPrefs,
    pub git: GitPrefs,
}

impl Default for PreferencesConfig {
    fn default() -> Self {
        Self {
            architecture: ArchitecturePrefs::default(),
            testing: TestingPrefs::default(),
            agents: AgentPrefs::default(),
            git: GitPrefs::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchitecturePrefs {
    pub default_style: String,
    pub service_style: String,
    pub prefer_explicit_boundaries: bool,
    pub avoid_premature_microservices: bool,
}

impl Default for ArchitecturePrefs {
    fn default() -> Self {
        Self {
            default_style: "modular".to_string(),
            service_style: "hexagonal".to_string(),
            prefer_explicit_boundaries: true,
            avoid_premature_microservices: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestingPrefs {
    pub require_unit_tests: bool,
    pub require_integration_tests_for_io: bool,
    pub minimum_coverage: u8,
    pub prefer_property_tests: bool,
    pub framework: String,
}

impl Default for TestingPrefs {
    fn default() -> Self {
        Self {
            require_unit_tests: true,
            require_integration_tests_for_io: true,
            minimum_coverage: 80,
            prefer_property_tests: false,
            framework: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentPrefs {
    pub reuse_lode_assets_first: bool,
    pub require_plan_before_write: bool,
    pub require_verification_before_completion: bool,
    pub handoff_format: String,
    pub context_budget_tokens: u32,
}

impl Default for AgentPrefs {
    fn default() -> Self {
        Self {
            reuse_lode_assets_first: true,
            require_plan_before_write: false,
            require_verification_before_completion: true,
            handoff_format: "pidgin".to_string(),
            context_budget_tokens: 6000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitPrefs {
    pub commit_style: String,
    pub prefer_atomic_commits: bool,
    pub require_clean_verification: bool,
}

impl Default for GitPrefs {
    fn default() -> Self {
        Self {
            commit_style: "conventional".to_string(),
            prefer_atomic_commits: true,
            require_clean_verification: true,
        }
    }
}
