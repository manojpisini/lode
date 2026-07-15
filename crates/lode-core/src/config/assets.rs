use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionExpr {
    Simple(Condition),
    AllOf { all_of: Vec<ConditionExpr> },
    AnyOf { any_of: Vec<ConditionExpr> },
    Not { not: Box<ConditionExpr> },
}

impl ConditionExpr {
    pub fn evaluate(&self, ctx: &EvaluateContext) -> bool {
        match self {
            ConditionExpr::Simple(c) => c.evaluate(ctx),
            ConditionExpr::AllOf { all_of } => all_of.iter().all(|e| e.evaluate(ctx)),
            ConditionExpr::AnyOf { any_of } => any_of.iter().any(|e| e.evaluate(ctx)),
            ConditionExpr::Not { not } => !not.evaluate(ctx),
        }
    }

    pub fn always_true() -> Self {
        ConditionExpr::Simple(Condition {
            r#type: "always".to_string(),
            value: String::new(),
            not: false,
            case_sensitive: false,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Condition {
    pub r#type: String,
    pub value: String,
    #[serde(default)]
    pub not: bool,
    #[serde(default)]
    pub case_sensitive: bool,
}

impl Condition {
    pub fn evaluate(&self, ctx: &EvaluateContext) -> bool {
        let result = match self.r#type.as_str() {
            "always" => true,
            "never" => false,
            "platform" => {
                let target = if self.case_sensitive {
                    ctx.os_platform.to_string()
                } else {
                    ctx.os_platform.to_lowercase()
                };
                let val = if self.case_sensitive {
                    self.value.clone()
                } else {
                    self.value.to_lowercase()
                };
                target == val
            }
            "has_feature" => {
                let val = if self.case_sensitive {
                    self.value.clone()
                } else {
                    self.value.to_lowercase()
                };
                ctx.features.iter().any(|f| {
                    if self.case_sensitive {
                        f == &val
                    } else {
                        f.eq_ignore_ascii_case(&val)
                    }
                })
            }
            "profile" => {
                if let Some(ref p) = ctx.profile {
                    let target = if self.case_sensitive {
                        p.clone()
                    } else {
                        p.to_lowercase()
                    };
                    let val = if self.case_sensitive {
                        self.value.clone()
                    } else {
                        self.value.to_lowercase()
                    };
                    target == val
                } else {
                    false
                }
            }
            "profile_contains" => {
                if let Some(ref p) = ctx.profile {
                    let target = if self.case_sensitive {
                        p.clone()
                    } else {
                        p.to_lowercase()
                    };
                    let val = if self.case_sensitive {
                        self.value.clone()
                    } else {
                        self.value.to_lowercase()
                    };
                    target.contains(&val)
                } else {
                    false
                }
            }
            "lang" => {
                if let Some(ref l) = ctx.lang {
                    let target = if self.case_sensitive {
                        l.clone()
                    } else {
                        l.to_lowercase()
                    };
                    let val = if self.case_sensitive {
                        self.value.clone()
                    } else {
                        self.value.to_lowercase()
                    };
                    target == val
                } else {
                    false
                }
            }
            "env_var" => {
                let val = if self.case_sensitive {
                    self.value.clone()
                } else {
                    self.value.to_uppercase()
                };
                ctx.env_vars.get(&val).is_some_and(|v| !v.is_empty())
            }
            "env_var_eq" => {
                let parts: Vec<&str> = self.value.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let key = if self.case_sensitive {
                        parts[0].to_string()
                    } else {
                        parts[0].to_uppercase()
                    };
                    let expected = parts[1].to_string();
                    ctx.env_vars.get(&key).is_some_and(|v| *v == expected)
                } else {
                    false
                }
            }
            "exists" => {
                let val = if self.case_sensitive {
                    self.value.clone()
                } else {
                    self.value.to_lowercase()
                };
                std::path::Path::new(&val).exists()
            }
            _ => false,
        };
        if self.not {
            !result
        } else {
            result
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetsConfig {
    pub auto_register: bool,
    pub templates: HashMap<String, AssetEntry>,
    pub profiles: HashMap<String, AssetEntry>,
    pub snippets: HashMap<String, AssetEntry>,
    pub commands: HashMap<String, AssetEntry>,
    pub recipes: HashMap<String, AssetEntry>,
    pub licenses: HashMap<String, AssetEntry>,
}

impl Default for AssetsConfig {
    fn default() -> Self {
        Self {
            auto_register: true,
            templates: HashMap::new(),
            profiles: HashMap::new(),
            snippets: HashMap::new(),
            commands: HashMap::new(),
            recipes: HashMap::new(),
            licenses: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetEntry {
    pub enabled: bool,
    #[serde(default)]
    pub condition: Option<ConditionExpr>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub priority: Option<i32>,
}

impl AssetEntry {
    pub fn is_active(&self, ctx: &EvaluateContext) -> bool {
        if !self.enabled {
            return false;
        }
        if let Some(ref expr) = self.condition {
            expr.evaluate(ctx)
        } else {
            true
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EvaluateContext {
    pub os_platform: String,
    pub profile: Option<String>,
    pub features: Vec<String>,
    pub lang: Option<String>,
    pub env_vars: HashMap<String, String>,
}

impl EvaluateContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_env() -> Self {
        let os_platform = if cfg!(target_os = "windows") {
            "windows".to_string()
        } else if cfg!(target_os = "linux") {
            "linux".to_string()
        } else if cfg!(target_os = "macos") {
            "macos".to_string()
        } else {
            "unknown".to_string()
        };
        let env_vars = std::env::vars().collect();
        Self {
            os_platform,
            profile: None,
            features: Vec::new(),
            lang: None,
            env_vars,
        }
    }

    pub fn with_profile(mut self, profile: &str) -> Self {
        self.profile = Some(profile.to_string());
        self
    }

    pub fn with_feature(mut self, feature: &str) -> Self {
        self.features.push(feature.to_string());
        self
    }

    pub fn with_lang(mut self, lang: &str) -> Self {
        self.lang = Some(lang.to_string());
        self
    }

    pub fn features_from_profile(&mut self, profile: &str) {
        let parts: Vec<&str> = profile.split('/').collect();
        if parts.len() >= 2 {
            self.features.push(parts[0].to_string());
            self.features.push(parts[1].to_string());
        }
        for segment in &parts {
            self.features.push(segment.to_string());
        }
        match parts.first().copied() {
            Some("systems") | Some("backend") | Some("frontend") | Some("python")
            | Some("java") | Some("desktop") | Some("games") | Some("challenge") => {
                if parts.len() >= 2 {
                    let lang_guess = parts[1].split('-').next().unwrap_or(parts[1]);
                    if self.lang.is_none() {
                        self.lang = Some(lang_guess.to_string());
                    }
                }
            }
            Some("core") => {
                let lang_map: HashMap<&str, &str> =
                    [("app", "generic"), ("cli", "generic"), ("lib", "generic")]
                        .iter()
                        .copied()
                        .collect();
                if parts.len() >= 2 {
                    if let Some(l) = lang_map.get(parts[1]) {
                        if self.lang.is_none() {
                            self.lang = Some(l.to_string());
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn default_assets_config() -> AssetsConfig {
    AssetsConfig::default()
}

pub fn merge_assets(base: &AssetsConfig, o: &AssetsConfig) -> AssetsConfig {
    let merge_map = |base_map: &HashMap<String, AssetEntry>,
                     o_map: &HashMap<String, AssetEntry>|
     -> HashMap<String, AssetEntry> {
        let mut merged = base_map.clone();
        for (k, v) in o_map {
            merged.insert(k.clone(), v.clone());
        }
        merged
    };
    AssetsConfig {
        auto_register: o.auto_register,
        templates: merge_map(&base.templates, &o.templates),
        profiles: merge_map(&base.profiles, &o.profiles),
        snippets: merge_map(&base.snippets, &o.snippets),
        commands: merge_map(&base.commands, &o.commands),
        recipes: merge_map(&base.recipes, &o.recipes),
        licenses: merge_map(&base.licenses, &o.licenses),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn condition_platform_matches() {
        let current = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else {
            "macos"
        };
        let c = Condition {
            r#type: "platform".to_string(),
            value: current.to_string(),
            not: false,
            case_sensitive: false,
        };
        let ctx = EvaluateContext::from_env();
        assert!(c.evaluate(&ctx));
    }

    #[test]
    fn condition_platform_negation() {
        let c = Condition {
            r#type: "platform".to_string(),
            value: "nonexistent_os".to_string(),
            not: true,
            case_sensitive: false,
        };
        let ctx = EvaluateContext::from_env();
        assert!(c.evaluate(&ctx));
    }

    #[test]
    fn condition_has_feature() {
        let c = Condition {
            r#type: "has_feature".to_string(),
            value: "rust".to_string(),
            not: false,
            case_sensitive: false,
        };
        let ctx = EvaluateContext::from_env().with_feature("rust");
        assert!(c.evaluate(&ctx));
    }

    #[test]
    fn condition_has_feature_missing() {
        let c = Condition {
            r#type: "has_feature".to_string(),
            value: "python".to_string(),
            not: false,
            case_sensitive: false,
        };
        let ctx = EvaluateContext::from_env().with_feature("rust");
        assert!(!c.evaluate(&ctx));
    }

    #[test]
    fn condition_profile_exact() {
        let c = Condition {
            r#type: "profile".to_string(),
            value: "systems/rust-bin".to_string(),
            not: false,
            case_sensitive: false,
        };
        let ctx = EvaluateContext::from_env().with_profile("systems/rust-bin");
        assert!(c.evaluate(&ctx));
    }

    #[test]
    fn condition_profile_contains() {
        let c = Condition {
            r#type: "profile_contains".to_string(),
            value: "rust".to_string(),
            not: false,
            case_sensitive: false,
        };
        let ctx = EvaluateContext::from_env().with_profile("systems/rust-bin");
        assert!(c.evaluate(&ctx));
    }

    #[test]
    fn condition_env_var() {
        std::env::set_var("LODE_TEST_ASSET_VAR", "present");
        let c = Condition {
            r#type: "env_var".to_string(),
            value: "LODE_TEST_ASSET_VAR".to_string(),
            not: false,
            case_sensitive: false,
        };
        let ctx = EvaluateContext::from_env();
        assert!(c.evaluate(&ctx));
        std::env::remove_var("LODE_TEST_ASSET_VAR");
    }

    #[test]
    fn condition_env_var_eq() {
        let c = Condition {
            r#type: "env_var_eq".to_string(),
            value: "LODE_TEST=active".to_string(),
            not: false,
            case_sensitive: false,
        };
        let ctx = EvaluateContext {
            env_vars: [("LODE_TEST".to_string(), "active".to_string())]
                .iter()
                .cloned()
                .collect(),
            ..EvaluateContext::from_env()
        };
        assert!(c.evaluate(&ctx));
    }

    #[test]
    fn condition_lang_match() {
        let c = Condition {
            r#type: "lang".to_string(),
            value: "rust".to_string(),
            not: false,
            case_sensitive: false,
        };
        let ctx = EvaluateContext::from_env().with_lang("rust");
        assert!(c.evaluate(&ctx));
    }

    #[test]
    fn condition_expr_all_of() {
        let expr = ConditionExpr::AllOf {
            all_of: vec![
                ConditionExpr::Simple(Condition {
                    r#type: "has_feature".to_string(),
                    value: "rust".to_string(),
                    not: false,
                    case_sensitive: false,
                }),
                ConditionExpr::Simple(Condition {
                    r#type: "lang".to_string(),
                    value: "rust".to_string(),
                    not: false,
                    case_sensitive: false,
                }),
            ],
        };
        let ctx = EvaluateContext::from_env()
            .with_feature("rust")
            .with_lang("rust");
        assert!(expr.evaluate(&ctx));
    }

    #[test]
    fn condition_expr_any_of() {
        let expr = ConditionExpr::AnyOf {
            any_of: vec![
                ConditionExpr::Simple(Condition {
                    r#type: "has_feature".to_string(),
                    value: "python".to_string(),
                    not: false,
                    case_sensitive: false,
                }),
                ConditionExpr::Simple(Condition {
                    r#type: "has_feature".to_string(),
                    value: "rust".to_string(),
                    not: false,
                    case_sensitive: false,
                }),
            ],
        };
        let ctx = EvaluateContext::from_env().with_feature("rust");
        assert!(expr.evaluate(&ctx));
    }

    #[test]
    fn condition_expr_not() {
        let expr = ConditionExpr::Not {
            not: Box::new(ConditionExpr::Simple(Condition {
                r#type: "has_feature".to_string(),
                value: "python".to_string(),
                not: false,
                case_sensitive: false,
            })),
        };
        let ctx = EvaluateContext::from_env().with_feature("rust");
        assert!(expr.evaluate(&ctx));
    }

    #[test]
    fn asset_entry_disabled_by_default() {
        let entry = AssetEntry::default();
        let ctx = EvaluateContext::from_env();
        assert!(!entry.is_active(&ctx));
    }

    #[test]
    fn asset_entry_enabled_passes() {
        let entry = AssetEntry {
            enabled: true,
            condition: None,
            tags: Vec::new(),
            priority: None,
        };
        let ctx = EvaluateContext::from_env();
        assert!(entry.is_active(&ctx));
    }

    #[test]
    fn asset_entry_enabled_with_failing_condition() {
        let entry = AssetEntry {
            enabled: true,
            condition: Some(ConditionExpr::Simple(Condition {
                r#type: "has_feature".to_string(),
                value: "python".to_string(),
                not: false,
                case_sensitive: false,
            })),
            tags: Vec::new(),
            priority: None,
        };
        let ctx = EvaluateContext::from_env().with_feature("rust");
        assert!(!entry.is_active(&ctx));
    }

    #[test]
    fn features_from_profile_rust() {
        let mut ctx = EvaluateContext::from_env();
        ctx.features_from_profile("systems/rust-bin");
        assert!(ctx.features.contains(&"systems".to_string()));
        assert!(ctx.features.contains(&"rust-bin".to_string()));
        assert_eq!(ctx.lang.as_deref(), Some("rust"));
    }

    #[test]
    fn features_from_profile_core() {
        let mut ctx = EvaluateContext::from_env();
        ctx.features_from_profile("core/bare");
        assert!(ctx.features.contains(&"core".to_string()));
        assert!(ctx.features.contains(&"bare".to_string()));
    }

    #[test]
    fn merge_assets_preserves_auto_register() {
        let base = AssetsConfig::default();
        let mut overrides = AssetsConfig::default();
        overrides.auto_register = false;
        let merged = merge_assets(&base, &overrides);
        assert!(!merged.auto_register);
    }

    #[test]
    fn merge_assets_combines_maps() {
        let mut base = AssetsConfig::default();
        base.templates.insert(
            "root/README.md".to_string(),
            AssetEntry {
                enabled: true,
                ..Default::default()
            },
        );
        let mut overrides = AssetsConfig::default();
        overrides.templates.insert(
            "rust/Cargo.toml".to_string(),
            AssetEntry {
                enabled: true,
                ..Default::default()
            },
        );
        let merged = merge_assets(&base, &overrides);
        assert_eq!(merged.templates.len(), 2);
    }

    #[test]
    fn condition_always_true() {
        let expr = ConditionExpr::always_true();
        let ctx = EvaluateContext::from_env();
        assert!(expr.evaluate(&ctx));
    }

    #[test]
    fn condition_never() {
        let c = Condition {
            r#type: "never".to_string(),
            value: String::new(),
            not: false,
            case_sensitive: false,
        };
        let ctx = EvaluateContext::from_env();
        assert!(!c.evaluate(&ctx));
    }
}
