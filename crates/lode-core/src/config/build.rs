use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildTarget {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildConfig {
    pub generate_makefile: bool,
    pub task_runner: String,
    pub targets: Vec<BuildTarget>,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            generate_makefile: true,
            task_runner: "just".to_string(),
            targets: Vec::new(),
        }
    }
}
