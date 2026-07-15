use std::fs;

use serde::{Deserialize, Serialize};

use crate::{LodeError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackManifest {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub profiles: Vec<String>,
    #[serde(default)]
    pub policies: Vec<String>,
    #[serde(default)]
    pub preferences: Vec<String>,
    #[serde(default)]
    pub recipes: Vec<String>,
    #[serde(default)]
    pub commands: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

pub fn list_packs() -> Result<Vec<PackManifest>> {
    let dir = crate::install::global_asset_dir("packs")?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut packs = Vec::new();
    for entry in fs::read_dir(dir.as_std_path()).map_err(|source| LodeError::Io {
        path: dir.as_str().into(),
        source,
    })? {
        let entry = entry.map_err(|source| LodeError::Io {
            path: dir.as_str().into(),
            source,
        })?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let content = fs::read_to_string(&path).map_err(|source| LodeError::Io {
            path: path.clone(),
            source,
        })?;
        match toml::from_str::<PackManifest>(&content) {
            Ok(p) => packs.push(p),
            Err(e) => {
                eprintln!(
                    "lode: warning: pack file {} parse error: {}",
                    path.display(),
                    e
                );
            }
        }
    }
    Ok(packs)
}

pub fn builtin_packs() -> Vec<PackManifest> {
    vec![
        PackManifest {
            id: "personal".to_string(),
            name: "Personal".to_string(),
            description: "Default personal preferences".to_string(),
            version: "1.0.0".to_string(),
            profiles: vec![],
            policies: vec![],
            preferences: vec![],
            recipes: vec![],
            commands: vec![],
            tags: vec!["default".to_string()],
        },
        PackManifest {
            id: "open-source".to_string(),
            name: "Open Source".to_string(),
            description: "Preferences for open source development".to_string(),
            version: "1.0.0".to_string(),
            profiles: vec![],
            policies: vec![
                "documentation.readme".to_string(),
                "quality.format".to_string(),
            ],
            preferences: vec![],
            recipes: vec![],
            commands: vec![],
            tags: vec!["open-source".to_string()],
        },
        PackManifest {
            id: "enterprise".to_string(),
            name: "Enterprise".to_string(),
            description: "Enterprise-grade preferences with full governance".to_string(),
            version: "1.0.0".to_string(),
            profiles: vec![],
            policies: vec![
                "security.no-plaintext-secrets".to_string(),
                "testing.must-have-tests".to_string(),
                "operations.healthcheck".to_string(),
            ],
            preferences: vec![],
            recipes: vec![],
            commands: vec![],
            tags: vec!["enterprise".to_string(), "governance".to_string()],
        },
        PackManifest {
            id: "experimental".to_string(),
            name: "Experimental".to_string(),
            description: "Bleeding-edge experimental preferences".to_string(),
            version: "1.0.0".to_string(),
            profiles: vec![],
            policies: vec![],
            preferences: vec![],
            recipes: vec![],
            commands: vec![],
            tags: vec!["experimental".to_string()],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_packs_returns_4() {
        assert_eq!(builtin_packs().len(), 4);
    }

    #[test]
    fn test_personal_pack_is_first() {
        let packs = builtin_packs();
        assert_eq!(packs[0].id, "personal");
    }

    #[test]
    fn test_enterprise_pack_has_policies() {
        let packs = builtin_packs();
        let ep = packs.iter().find(|p| p.id == "enterprise").unwrap();
        assert!(ep.policies.contains(&"testing.must-have-tests".to_string()));
    }
}
