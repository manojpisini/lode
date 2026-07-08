use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentConfig {
    pub auto_sync: bool,
    pub generate_claude: bool,
    pub generate_agents: bool,
    pub generate_cursor: bool,
    pub generate_windsurf: bool,
    pub generate_mcp_json: bool,
    pub context_dir: Option<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            auto_sync: true,
            generate_claude: true,
            generate_agents: true,
            generate_cursor: false,
            generate_windsurf: false,
            generate_mcp_json: false,
            context_dir: None,
        }
    }
}
