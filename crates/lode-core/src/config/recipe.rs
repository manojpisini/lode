use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipeStep {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recipe {
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<RecipeStep>,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipeConfig {
    pub recipes: Vec<Recipe>,
}

impl Default for RecipeConfig {
    fn default() -> Self {
        Self {
            recipes: Vec::new(),
        }
    }
}
