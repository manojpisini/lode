use serde::{Deserialize, Serialize};

use crate::catalog::AssetCatalogEntry;
use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub intent: String,
    pub resolved_actions: Vec<SimulatedAction>,
    pub confidence: f64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatedAction {
    pub asset_id: String,
    pub action: String,
    pub probability: f64,
}

pub fn simulate_intent(intent: &str, catalog: &[AssetCatalogEntry]) -> Result<SimulationResult> {
    let start = std::time::Instant::now();

    let mut actions = Vec::new();
    let lower = intent.to_lowercase();

    for asset in catalog {
        let mut probability = 0.0f64;

        let summary = &asset.summary;
        if summary.to_lowercase().contains(&lower) || lower.contains(&summary.to_lowercase()) {
            probability += 0.4;
        }

        for intent_str in &asset.intents {
            if intent_str.to_lowercase().contains(&lower)
                || lower.contains(&intent_str.to_lowercase())
            {
                probability += 0.3;
            }
        }

        for tag in &asset.tags {
            if lower.contains(&tag.to_lowercase()) {
                probability += 0.2;
            }
        }

        if probability > 0.0 {
            actions.push(SimulatedAction {
                asset_id: asset.id.clone(),
                action: "suggest".to_string(),
                probability: probability.min(1.0),
            });
        }
    }

    actions.sort_by(|a, b| {
        b.probability
            .partial_cmp(&a.probability)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    actions.truncate(10);

    let confidence = actions.first().map(|a| a.probability).unwrap_or(0.0);
    let duration = start.elapsed();

    Ok(SimulationResult {
        intent: intent.to_string(),
        resolved_actions: actions,
        confidence,
        duration_ms: duration.as_millis() as u64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::AssetCatalogEntry;

    fn sample_catalog() -> Vec<AssetCatalogEntry> {
        fn entry(
            id: &str,
            summary: &str,
            intents: Vec<&str>,
            tags: Vec<&str>,
        ) -> AssetCatalogEntry {
            AssetCatalogEntry {
                id: id.into(),
                kind: "command".into(),
                summary: summary.into(),
                intents: intents.into_iter().map(|s| s.into()).collect(),
                tags: tags.into_iter().map(|s| s.into()).collect(),
                languages: vec![],
                project_types: vec![],
                maturity: "stable".into(),
                requires: vec![],
                recommends: vec![],
                conflicts: vec![],
                verification: vec![],
                status: "stable".into(),
                quality_score: None,
                last_verified: None,
                verification_tests: None,
                verification_fixtures: None,
                verification_last_result: None,
                deprecation_replacement: None,
                deprecation_remove_after: None,
                deprecation_migration: None,
            }
        }
        vec![
            entry(
                "cmd.build",
                "Build the project",
                vec!["compile", "build"],
                vec!["build"],
            ),
            entry(
                "cmd.test",
                "Run tests",
                vec!["test", "check"],
                vec!["testing"],
            ),
        ]
    }

    #[test]
    fn test_simulate_intent_build_match() {
        let result = simulate_intent("build", &sample_catalog()).unwrap();
        assert!(result.confidence > 0.0);
        assert_eq!(result.resolved_actions[0].asset_id, "cmd.build");
    }

    #[test]
    fn test_simulate_intent_no_match() {
        let result = simulate_intent("unknown", &sample_catalog()).unwrap();
        assert_eq!(result.confidence, 0.0);
        assert!(result.resolved_actions.is_empty());
    }

    #[test]
    fn test_simulate_intent_max_10_actions() {
        fn entry(id: &str) -> AssetCatalogEntry {
            AssetCatalogEntry {
                id: id.into(),
                kind: "command".into(),
                summary: "test summary".into(),
                intents: vec!["test".into()],
                tags: vec![],
                languages: vec![],
                project_types: vec![],
                maturity: "stable".into(),
                requires: vec![],
                recommends: vec![],
                conflicts: vec![],
                verification: vec![],
                status: "stable".into(),
                quality_score: None,
                last_verified: None,
                verification_tests: None,
                verification_fixtures: None,
                verification_last_result: None,
                deprecation_replacement: None,
                deprecation_remove_after: None,
                deprecation_migration: None,
            }
        }
        let many: Vec<AssetCatalogEntry> = (0..15).map(|i| entry(&format!("cmd.{i}"))).collect();
        let result = simulate_intent("test", &many).unwrap();
        assert!(result.resolved_actions.len() <= 10);
    }
}
