use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHooksConfig {
    pub pre_commit: bool,
    pub pre_push: bool,
    pub commit_msg: bool,
}

impl Default for GitHooksConfig {
    fn default() -> Self {
        Self {
            pre_commit: true,
            pre_push: true,
            commit_msg: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitConfig {
    pub auto_init: bool,
    pub initial_branch: String,
    pub initial_commit: bool,
    pub initial_commit_msg: String,
    pub branch_strategy: String,
    pub commit_convention: String,
    pub commit_signing: bool,
    pub sign_key: Option<String>,
    pub hooks: GitHooksConfig,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            auto_init: true,
            initial_branch: "main".to_string(),
            initial_commit: true,
            initial_commit_msg: "feat: initial commit".to_string(),
            branch_strategy: "trunk".to_string(),
            commit_convention: "conventional".to_string(),
            commit_signing: false,
            sign_key: None,
            hooks: GitHooksConfig::default(),
        }
    }
}
