use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceMember {
    pub path: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub members: Vec<WorkspaceMember>,
    pub shared_deps: bool,
    pub shared_toolchain: bool,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            members: Vec::new(),
            shared_deps: true,
            shared_toolchain: true,
        }
    }
}
