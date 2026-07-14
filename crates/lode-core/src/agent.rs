use std::{fs, path::PathBuf};

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

use crate::{add_managed_file, load_project_config, LodeError, ManagedBy, Result, ValidatedRoot};

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

#[derive(Debug, Clone, Serialize)]
pub struct PolicyReport {
    pub files_written: Vec<Utf8PathBuf>,
    pub project_name: String,
    pub profile: String,
    pub language: String,
}

pub fn generate_agent_policy(project_dir: &Utf8Path) -> Result<PolicyReport> {
    let proj_path = project_dir.to_path_buf();
    let config = load_project_config(&proj_path).ok();
    let project_name = config
        .as_ref()
        .map(|c| c.project.name.clone())
        .or_else(|| project_dir.file_name().map(|s| s.to_string()))
        .unwrap_or_else(|| "project".to_string());
    let profile = config
        .as_ref()
        .map(|c| c.project.profile.clone())
        .unwrap_or_else(|| "unknown".to_string());
    let language = config
        .as_ref()
        .and_then(|c| c.project.language.clone())
        .unwrap_or_else(|| "unknown".to_string());
    let components = config
        .as_ref()
        .map(|c| c.project.components.clone())
        .unwrap_or_default();

    let plan = load_agent_plan(project_dir).unwrap_or_default();
    let root = ValidatedRoot::new(project_dir)?;

    let mut files_written = Vec::new();
    let policy_dir = Utf8Path::new(".lode");

    let agents_content = generate_agents_md(&project_name, &profile, &language, &components, &plan);
    let agents_path = Utf8PathBuf::from("AGENTS.md");
    root.write_atomic(&agents_path, &agents_content)?;
    files_written.push(agents_path.clone());
    let _ = add_managed_file(project_dir, &agents_path, ManagedBy::Agent, "Canonical agent bootstrap contract");

    let claude_content = generate_claude_md(&project_name, &profile, &language, &plan);
    let claude_path = Utf8PathBuf::from("CLAUDE.md");
    root.write_atomic(&claude_path, &claude_content)?;
    files_written.push(claude_path.clone());
    let _ = add_managed_file(project_dir, &claude_path, ManagedBy::Agent, "Claude-specific project context");

    let codex_content = generate_codex_md(&project_name, &language, &components);
    let codex_path = Utf8PathBuf::from("CODEX.md");
    root.write_atomic(&codex_path, &codex_content)?;
    files_written.push(codex_path.clone());
    let _ = add_managed_file(project_dir, &codex_path, ManagedBy::Agent, "Codex-specific code generation patterns");

    let cursor_rules = generate_cursor_rules(&language, &profile);
    let cursor_path = Utf8PathBuf::from(".cursorrules");
    root.write_atomic(&cursor_path, &cursor_rules)?;
    files_written.push(cursor_path.clone());
    let _ = add_managed_file(project_dir, &cursor_path, ManagedBy::Agent, "Cursor editor rules");

    let windsurf_rules = generate_windsurf_rules(&language, &profile);
    let windsurf_path = Utf8PathBuf::from(".windsurfrules");
    root.write_atomic(&windsurf_path, &windsurf_rules)?;
    files_written.push(windsurf_path.clone());
    let _ = add_managed_file(project_dir, &windsurf_path, ManagedBy::Agent, "Windsurf editor rules");

    let mcp_json = generate_mcp_json(&project_name);
    let mcp_path = Utf8PathBuf::from(".mcp.json");
    root.write_atomic(&mcp_path, &mcp_json)?;
    files_written.push(mcp_path.clone());
    let _ = add_managed_file(project_dir, &mcp_path, ManagedBy::Agent, "MCP server configuration");

    let plan_content = generate_plan_md(&project_name, &plan);
    let plan_path = policy_dir.join("context").join("PLAN.md");
    root.create_dir_all(Utf8Path::new(".lode/context"))?;
    root.write_atomic(&plan_path, &plan_content)?;
    files_written.push(plan_path.clone());
    let _ = add_managed_file(project_dir, &plan_path, ManagedBy::Agent, "Project plan");

    let constraints_content = generate_constraints_md(&profile, &language, &components);
    let constraints_path = policy_dir.join("context").join("CONSTRAINTS.md");
    root.write_atomic(&constraints_path, &constraints_content)?;
    files_written.push(constraints_path.clone());
    let _ = add_managed_file(project_dir, &constraints_path, ManagedBy::Agent, "Project constraints");

    let tasks_content = generate_tasks_md(&plan);
    let tasks_path = policy_dir.join("context").join("TASKS.md");
    root.write_atomic(&tasks_path, &tasks_content)?;
    files_written.push(tasks_path.clone());
    let _ = add_managed_file(project_dir, &tasks_path, ManagedBy::Agent, "Project tasks");

    Ok(PolicyReport {
        files_written,
        project_name,
        profile,
        language,
    })
}

fn generate_agents_md(
    name: &str,
    profile: &str,
    language: &str,
    components: &[String],
    plan: &AgentPlan,
) -> String {
    let pending = plan.tasks.iter().filter(|t| !t.done).count();
    let mut md = format!(
        "# LODE Agent Bootstrap Contract\n\n\
        ## Project\n\n\
        - **Name:** {name}\n\
        - **Profile:** {profile}\n\
        - **Language:** {language}\n",
    );
    if !components.is_empty() {
        md.push_str("- **Components:** ");
        md.push_str(&components.join(", "));
        md.push('\n');
    }
    md.push_str("\n## Core Principles\n\n\
        1. **Discover before create** — Always ask LODE first\n\
        2. **Reuse before compose** — Prefer composing existing recipes\n\
        3. **Compose before customize** — Use recipe composition\n\
        4. **Create only when necessary** — Promote back to LODE\n\n");
    md.push_str("## Contract\n\n\
        - LODE owns: scaffolding, templates, profiles, recipes, conventions\n\
        - Agent owns: implementation logic, architecture decisions\n\
        - Both own: agent context files, build config, CI\n\n");
    md.push_str("## Quick Start\n\n");
    md.push_str("```\n");
    md.push_str("lode agent bootstrap --json\n");
    md.push_str("lode assets search \"database\"\n");
    md.push_str("lode agent resolve --intent \"...\" --json\n");
    md.push_str("lode plan create --intent \"...\"\n");
    md.push_str("```\n");
    if pending > 0 {
        md.push_str(&format!("\n## Pending Tasks ({pending})\n\n"));
        for task in plan.tasks.iter().filter(|t| !t.done) {
            md.push_str(&format!(
                "- #{} {} (branch: {})\n",
                task.id,
                task.task,
                task.branch.as_deref().unwrap_or("main")
            ));
        }
    }
    md
}

fn generate_claude_md(name: &str, profile: &str, language: &str, plan: &AgentPlan) -> String {
    let pending: Vec<&AgentTask> = plan.tasks.iter().filter(|t| !t.done).collect();
    let mut md = format!(
        "# Project Context\n\n\
        ## Identity\n\n\
        - **Project:** {name}\n\
        - **Profile:** {profile}\n\
        - **Language:** {language}\n\n\
        ## Instructions\n\n\
        1. Run `lode agent bootstrap --json` to discover LODE capabilities\n\
        2. Use `lode assets search <query>` before writing custom code\n\
        3. Keep AGENTS.md and CLAUDE.md in sync with project changes\n\n",
    );
    if !pending.is_empty() {
        md.push_str("## Current Tasks\n\n");
        for task in &pending {
            md.push_str(&format!(
                "- [TODO] #{} {} (branch: {})\n",
                task.id,
                task.task,
                task.branch.as_deref().unwrap_or("main")
            ));
        }
        md.push('\n');
    }
    md
}

fn generate_codex_md(name: &str, language: &str, components: &[String]) -> String {
    let mut md = format!(
        "# Code Generation Patterns\n\n\
        ## Project\n\n\
        - **Name:** {name}\n\
        - **Language:** {language}\n\n\
        ## Style Preferences\n\n\
        1. Follow the existing code style in the repository\n\
        2. Use the same patterns as neighboring files\n\
        3. Prefer the project's established framework choices\n\n",
    );
    if !components.is_empty() {
        md.push_str("## Active Components\n\n");
        for c in components {
            md.push_str(&format!("- {c}\n"));
        }
        md.push('\n');
    }
    md.push_str("## Conventions\n\n");
    md.push_str("- Run `lode check` before committing\n");
    md.push_str("- Run `lode scan secrets` to prevent secret leakage\n");
    md.push_str("- Follow the project's naming conventions\n");
    md
}

fn generate_cursor_rules(language: &str, profile: &str) -> String {
    format!(
        "# Cursor Rules\n\n\
        ## Project\n\
        Language: {language}\n\
        Profile: {profile}\n\n\
        ## Editing\n\
        - Prefer reading files before editing\n\
        - Use lode commands for project operations\n\
        - Run `lode check` after changes\n"
    )
}

fn generate_windsurf_rules(language: &str, profile: &str) -> String {
    format!(
        "# Windsurf Rules\n\n\
        ## Project\n\
        Language: {language}\n\
        Profile: {profile}\n\n\
        ## Editing\n\
        - Read files before editing\n\
        - Use lode for scaffolding and conventions\n\
        - Verify with `lode check` after changes\n"
    )
}

fn generate_mcp_json(name: &str) -> String {
    serde_json::json!({
        "mcpServers": {},
        "project": name
    })
    .to_string()
}

fn generate_plan_md(name: &str, plan: &AgentPlan) -> String {
    let mut md = format!("# Plan: {name}\n\n");
    if plan.tasks.is_empty() {
        md.push_str("_No tasks yet. Use `lode agent plan add <task>` to start._\n");
    } else {
        for task in &plan.tasks {
            let status = if task.done { "DONE" } else { "PENDING" };
            md.push_str(&format!(
                "- [{}] #{} {}\n",
                status, task.id, task.task
            ));
        }
    }
    md
}

fn generate_constraints_md(profile: &str, language: &str, components: &[String]) -> String {
    let mut md = format!(
        "# Constraints\n\n\
        ## Project\n\
        - **Profile:** {profile}\n\
        - **Language:** {language}\n\n\
        ## Hard Rules\n\n\
        1. All write paths must pass centralized path validation\n\
        2. All child processes must go through the approved process runner\n\
        3. All secrets must be redacted before logs, metrics, or errors\n\
        4. Add tests for every new security-sensitive behavior\n\
        5. Prefer safe defaults over convenience\n\n",
    );
    if !components.is_empty() {
        md.push_str("## Component Constraints\n\n");
        for c in components {
            md.push_str(&format!("- `{c}`: follow standard conventions for this component\n"));
        }
    }
    md
}

fn generate_tasks_md(plan: &AgentPlan) -> String {
    let mut md = String::from("# Tasks\n\n");
    if plan.tasks.is_empty() {
        md.push_str("_No tasks defined._\n");
    } else {
        md.push_str("| # | Status | Task | Branch |\n");
        md.push_str("|---|--------|------|--------|\n");
        for task in &plan.tasks {
            let status = if task.done { "✅" } else { "⬜" };
            let branch = task.branch.as_deref().unwrap_or("-");
            md.push_str(&format!("| {} | {} | {} | {} |\n", task.id, status, task.task, branch));
        }
    }
    md
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
