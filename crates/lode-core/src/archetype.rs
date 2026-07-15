use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Archetype {
    pub id: String,
    pub summary: String,
    pub profile: String,
    pub depth: String,
    #[serde(default)]
    pub policies: Vec<String>,
    #[serde(default)]
    pub recipes: Vec<String>,
    #[serde(default)]
    pub commands: Vec<String>,
    #[serde(default)]
    pub languages: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

pub fn builtin_archetypes() -> Vec<Archetype> {
    vec![
        Archetype {
            id: "startup-api".to_string(),
            summary: "Quickly build and iterate on an API service".to_string(),
            profile: "core/bare".to_string(),
            depth: "development".to_string(),
            policies: vec![
                "security.no-plaintext-secrets".to_string(),
                "quality.format".to_string(),
            ],
            recipes: vec!["api/rest".to_string()],
            commands: vec![],
            languages: vec![],
            tags: vec![
                "rapid".to_string(),
                "api".to_string(),
                "minimal".to_string(),
            ],
        },
        Archetype {
            id: "production-api".to_string(),
            summary: "Production-grade API service with observability and security".to_string(),
            profile: "systems/rust-service".to_string(),
            depth: "production".to_string(),
            policies: vec![
                "security.no-plaintext-secrets".to_string(),
                "quality.format".to_string(),
                "testing.must-have-tests".to_string(),
                "dependencies.vulnerability-scan".to_string(),
            ],
            recipes: vec![
                "database/postgres".to_string(),
                "observability/opentelemetry".to_string(),
                "security/hardened".to_string(),
                "ci/github-actions".to_string(),
            ],
            commands: vec![],
            languages: vec!["rust".to_string(), "python".to_string()],
            tags: vec![
                "production".to_string(),
                "api".to_string(),
                "service".to_string(),
            ],
        },
        Archetype {
            id: "open-source-cli".to_string(),
            summary: "Open source CLI tool with CI and release automation".to_string(),
            profile: "systems/rust-cli".to_string(),
            depth: "production".to_string(),
            policies: vec![
                "quality.format".to_string(),
                "testing.must-have-tests".to_string(),
                "documentation.readme".to_string(),
            ],
            recipes: vec!["ci/github-actions".to_string(), "git/release".to_string()],
            commands: vec![],
            languages: vec!["rust".to_string()],
            tags: vec!["open-source".to_string(), "cli".to_string()],
        },
        Archetype {
            id: "internal-tool".to_string(),
            summary: "Internal tool or microservice".to_string(),
            profile: "core/bare".to_string(),
            depth: "development".to_string(),
            policies: vec!["security.no-plaintext-secrets".to_string()],
            recipes: vec![],
            commands: vec![],
            languages: vec![],
            tags: vec!["internal".to_string(), "tool".to_string()],
        },
        Archetype {
            id: "enterprise-service".to_string(),
            summary: "Enterprise service with full governance".to_string(),
            profile: "systems/rust-service".to_string(),
            depth: "production".to_string(),
            policies: vec![
                "security.no-plaintext-secrets".to_string(),
                "testing.must-have-tests".to_string(),
                "dependencies.vulnerability-scan".to_string(),
                "documentation.readme".to_string(),
                "operations.healthcheck".to_string(),
            ],
            recipes: vec![
                "database/postgres".to_string(),
                "database/redis".to_string(),
                "observability/opentelemetry".to_string(),
                "security/hardened".to_string(),
                "ci/github-actions".to_string(),
                "operations/backup".to_string(),
                "api/rest".to_string(),
            ],
            commands: vec![],
            languages: vec![
                "rust".to_string(),
                "python".to_string(),
                "typescript".to_string(),
            ],
            tags: vec![
                "enterprise".to_string(),
                "governance".to_string(),
                "service".to_string(),
            ],
        },
        Archetype {
            id: "agentic-research-system".to_string(),
            summary: "Research system with agent orchestration".to_string(),
            profile: "core/bare".to_string(),
            depth: "production".to_string(),
            policies: vec![],
            recipes: vec![
                "agent/native".to_string(),
                "observability/opentelemetry".to_string(),
            ],
            commands: vec![],
            languages: vec!["python".to_string(), "rust".to_string()],
            tags: vec!["agent".to_string(), "research".to_string()],
        },
        Archetype {
            id: "developer-platform".to_string(),
            summary: "Developer platform or tool with extensibility".to_string(),
            profile: "systems/rust-cli".to_string(),
            depth: "production".to_string(),
            policies: vec![
                "quality.format".to_string(),
                "testing.must-have-tests".to_string(),
            ],
            recipes: vec![
                "ci/github-actions".to_string(),
                "git/release".to_string(),
                "plugin/system".to_string(),
            ],
            commands: vec![],
            languages: vec!["rust".to_string(), "typescript".to_string()],
            tags: vec!["platform".to_string(), "developer-tools".to_string()],
        },
    ]
}

pub fn resolve_archetype(id: &str) -> Result<Archetype> {
    let archetypes = builtin_archetypes();
    archetypes
        .into_iter()
        .find(|a| a.id == id)
        .ok_or_else(|| crate::LodeError::Message(format!("archetype not found: {id}")))
}

pub fn list_archetypes() -> Vec<Archetype> {
    builtin_archetypes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_archetypes_returns_7() {
        assert_eq!(list_archetypes().len(), 7);
    }

    #[test]
    fn test_resolve_archetype_finds_production_api() {
        let a = resolve_archetype("production-api").unwrap();
        assert_eq!(a.profile, "systems/rust-service");
        assert!(a.policies.contains(&"testing.must-have-tests".to_string()));
    }

    #[test]
    fn test_resolve_archetype_unknown_returns_error() {
        assert!(resolve_archetype("nonexistent").is_err());
    }

    #[test]
    fn test_startup_api_has_minimal_policies() {
        let a = resolve_archetype("startup-api").unwrap();
        assert_eq!(a.depth, "development");
    }
}
