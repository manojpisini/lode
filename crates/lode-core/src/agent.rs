use std::{fs, path::PathBuf};

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

use crate::{LodeError, Result, ValidatedRoot};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub auto_sync: bool,
    pub generate_claude: bool,
    pub generate_agents: bool,
    pub generate_cursor: bool,
    pub generate_windsurf: bool,
    pub generate_mcp_json: bool,
    pub context_dir: Option<Utf8PathBuf>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPlan {
    pub next_id: u32,
    pub tasks: Vec<AgentTask>,
}

impl Default for AgentPlan {
    fn default() -> Self {
        Self {
            next_id: 1,
            tasks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: u32,
    pub task: String,
    pub branch: Option<String>,
    pub done: bool,
}

fn plan_path(project_dir: &Utf8Path) -> Utf8PathBuf {
    project_dir.join(".lode").join("agent_plan.json")
}

pub fn load_agent_plan(project_dir: &Utf8Path) -> Result<AgentPlan> {
    let path = plan_path(project_dir);
    if !path.exists() {
        return Ok(AgentPlan::default());
    }
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: PathBuf::from(path.as_str()),
        source,
    })?;
    serde_json::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))
}

pub fn save_agent_plan(project_dir: &Utf8Path, plan: &AgentPlan) -> Result<()> {
    let root = ValidatedRoot::new(project_dir)?;
    root.create_dir_all(".lode")?;
    let raw = serde_json::to_string_pretty(plan)
        .map_err(|error| LodeError::Message(error.to_string()))?;
    root.write_atomic(".lode/agent_plan.json", raw)?;
    Ok(())
}

pub fn agent_sync(project_dir: &Utf8Path, config: &AgentConfig) -> Result<SyncReport> {
    let plan = load_agent_plan(project_dir)?;
    let context_dir = config
        .context_dir
        .clone()
        .unwrap_or_else(|| project_dir.join(".lode").join("context"));
    create_dir_all_validated(&context_dir)?;
    let mut report = SyncReport::default();

    if config.generate_claude {
        let path = context_dir.join("CLAUDE.md");
        write_claude_context(project_dir, &path, &plan)?;
        report.files_written.push(path);
    }
    if config.generate_agents {
        let path = context_dir.join("AGENTS.md");
        write_agents_context(project_dir, &path, &plan)?;
        report.files_written.push(path);
    }
    if config.generate_cursor {
        let path = context_dir.join(".cursorrules");
        write_cursor_context(project_dir, &path, &plan)?;
        report.files_written.push(path);
    }
    if config.generate_windsurf {
        let path = context_dir.join(".windsurfrules");
        write_windsurf_context(project_dir, &path, &plan)?;
        report.files_written.push(path);
    }
    if config.generate_mcp_json {
        let path = context_dir.join("mcp.json");
        write_mcp_json(project_dir, &path)?;
        report.files_written.push(path);
    }

    report.plan = plan;
    Ok(report)
}

fn create_dir_all_validated(path: &Utf8Path) -> Result<()> {
    if path.exists() {
        ValidatedRoot::new(path)?;
        return Ok(());
    }
    let parent = path
        .parent()
        .ok_or_else(|| LodeError::Message(format!("path has no parent: {path}")))?;
    create_dir_all_validated(parent)?;
    let root = ValidatedRoot::new(parent)?;
    let name = path
        .file_name()
        .ok_or_else(|| LodeError::Message(format!("path has no file name: {path}")))?;
    root.create_dir_all(name)?;
    Ok(())
}

fn write_validated(path: &Utf8Path, content: impl AsRef<[u8]>) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| LodeError::Message(format!("path has no parent: {path}")))?;
    create_dir_all_validated(parent)?;
    let root = ValidatedRoot::new(parent)?;
    let name = path
        .file_name()
        .ok_or_else(|| LodeError::Message(format!("path has no file name: {path}")))?;
    root.write_atomic(name, content)?;
    Ok(())
}

#[derive(Debug, Default, Clone)]
pub struct SyncReport {
    pub files_written: Vec<Utf8PathBuf>,
    pub plan: AgentPlan,
}

fn write_claude_context(project_dir: &Utf8Path, path: &Utf8Path, plan: &AgentPlan) -> Result<()> {
    let mut content = format!(
        "# Project Context\n\nProject: {}\n\n",
        project_dir.file_name().unwrap_or("unknown")
    );
    if !plan.tasks.is_empty() {
        content.push_str("## Current Tasks\n\n");
        for task in &plan.tasks {
            let status = if task.done { "DONE" } else { "TODO" };
            content.push_str(&format!("- [{}] #{} {}\n", status, task.id, task.task));
        }
    }
    write_validated(path, content)?;
    Ok(())
}

fn write_agents_context(project_dir: &Utf8Path, path: &Utf8Path, plan: &AgentPlan) -> Result<()> {
    let mut content = format!(
        "# Agent Instructions\n\nProject: {}\n\n",
        project_dir.file_name().unwrap_or("unknown")
    );
    if !plan.tasks.is_empty() {
        content.push_str("## Pending Work\n\n");
        for task in plan.tasks.iter().filter(|t| !t.done) {
            content.push_str(&format!(
                "- #{} {} (branch: {})\n",
                task.id,
                task.task,
                task.branch.as_deref().unwrap_or("main")
            ));
        }
    }
    write_validated(path, content)?;
    Ok(())
}

fn write_cursor_context(project_dir: &Utf8Path, path: &Utf8Path, plan: &AgentPlan) -> Result<()> {
    let content = format!(
        "Project: {}\nActive tasks: {}\n",
        project_dir.file_name().unwrap_or("unknown"),
        plan.tasks.iter().filter(|t| !t.done).count()
    );
    write_validated(path, content)?;
    Ok(())
}

fn write_windsurf_context(project_dir: &Utf8Path, path: &Utf8Path, plan: &AgentPlan) -> Result<()> {
    let content = format!(
        "Project: {}\nActive tasks: {}\n",
        project_dir.file_name().unwrap_or("unknown"),
        plan.tasks.iter().filter(|t| !t.done).count()
    );
    write_validated(path, content)?;
    Ok(())
}

fn write_mcp_json(project_dir: &Utf8Path, path: &Utf8Path) -> Result<()> {
    let content = serde_json::json!({
        "mcpServers": {},
        "project": project_dir.file_name().unwrap_or("unknown")
    });
    write_validated(path, content.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_plan_returns_empty_when_missing() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let plan = load_agent_plan(&root).unwrap();
        assert!(plan.tasks.is_empty());
        assert_eq!(plan.next_id, 1);
    }

    #[test]
    fn save_load_round_trip() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let plan = AgentPlan {
            next_id: 3,
            tasks: vec![
                AgentTask {
                    id: 1,
                    task: "fix bug".to_string(),
                    branch: Some("fix/bug".to_string()),
                    done: true,
                },
                AgentTask {
                    id: 2,
                    task: "add tests".to_string(),
                    branch: None,
                    done: false,
                },
            ],
        };
        save_agent_plan(&root, &plan).unwrap();
        let loaded = load_agent_plan(&root).unwrap();
        assert_eq!(loaded.next_id, 3);
        assert_eq!(loaded.tasks.len(), 2);
        assert!(loaded.tasks[0].done);
        assert!(!loaded.tasks[1].done);
    }

    #[test]
    fn agent_sync_generates_claude_and_agents() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let config = AgentConfig {
            auto_sync: true,
            generate_claude: true,
            generate_agents: true,
            generate_cursor: false,
            generate_windsurf: false,
            generate_mcp_json: false,
            context_dir: None,
        };
        let report = agent_sync(&root, &config).unwrap();
        assert_eq!(report.files_written.len(), 2);
        assert!(report
            .files_written
            .iter()
            .any(|p| p.file_name() == Some("CLAUDE.md")));
        assert!(report
            .files_written
            .iter()
            .any(|p| p.file_name() == Some("AGENTS.md")));
    }

    #[test]
    fn agent_sync_respects_config_flags() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let config = AgentConfig {
            auto_sync: false,
            generate_claude: false,
            generate_agents: false,
            generate_cursor: false,
            generate_windsurf: false,
            generate_mcp_json: false,
            context_dir: None,
        };
        let report = agent_sync(&root, &config).unwrap();
        assert!(report.files_written.is_empty());
    }
}
