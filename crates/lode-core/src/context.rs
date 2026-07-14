use std::collections::HashMap;
use std::fs;

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

use crate::{add_managed_file, LodeError, ManagedBy, Result, ValidatedRoot};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPack {
    pub schema_version: u32,
    pub project_name: String,
    pub files: Vec<ContextFile>,
    pub decisions: Vec<Decision>,
    pub quality_gates: Vec<QualityGate>,
    pub commands: Vec<ContextCommand>,
    pub risks: Vec<Risk>,
    pub recent_changes: Vec<Change>,
    pub dependencies: Vec<Dependency>,
    pub hashes: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextFile {
    pub path: String,
    pub summary: String,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: String,
    pub title: String,
    pub description: String,
    pub date: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGate {
    pub name: String,
    pub command: String,
    pub required: bool,
    pub last_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextCommand {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub id: String,
    pub description: String,
    pub severity: String,
    pub mitigated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub path: String,
    pub summary: String,
    pub timestamp: String,
    pub author: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub kind: String,
}

fn hash_content(content: &str) -> String {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

impl ContextPack {
    pub fn new(project_name: &str) -> Self {
        Self {
            schema_version: 1,
            project_name: project_name.to_string(),
            files: Vec::new(),
            decisions: Vec::new(),
            quality_gates: Vec::new(),
            commands: Vec::new(),
            risks: Vec::new(),
            recent_changes: Vec::new(),
            dependencies: Vec::new(),
            hashes: HashMap::new(),
        }
    }

    pub fn from_project(project_dir: &Utf8Path) -> Result<Self> {
        let name = project_dir
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "project".to_string());
        let mut pack = Self::new(&name);

        pack.index_files(project_dir)?;
        pack.detect_git_state(project_dir);
        pack.detect_quality_gates(project_dir);
        pack.load_decisions(project_dir);
        pack.detect_dependencies(project_dir);

        Ok(pack)
    }

    fn index_files(&mut self, project_dir: &Utf8Path) -> Result<()> {
        let context_dir = project_dir.join("_ctx_");
        if !context_dir.exists() {
            return Ok(());
        }
        for entry in fs::read_dir(&context_dir).map_err(|e| LodeError::Io {
            path: std::path::PathBuf::from(context_dir.as_str()),
            source: e,
        })? {
            let entry = entry.map_err(|e| LodeError::Io {
                path: std::path::PathBuf::from(context_dir.as_str()),
                source: e,
            })?;
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if let Ok(content) = fs::read_to_string(&path) {
                        let h = hash_content(&content);
                        let first_line = content.lines().next().unwrap_or("").to_string();
                        let summary = first_line.trim_start_matches("# ").to_string();
                        self.files.push(ContextFile {
                            path: format!("_ctx_/{name}"),
                            summary,
                            hash: h.clone(),
                        });
                        self.hashes.insert(format!("_ctx_/{name}"), h);
                    }
                }
            }
        }
        Ok(())
    }

    fn detect_git_state(&mut self, project_dir: &Utf8Path) {
        let git_dir = project_dir.join(".git");
        if !git_dir.exists() {
            return;
        }
        if let Ok(output) = std::process::Command::new("git")
            .args(["log", "--oneline", "-5"])
            .current_dir(project_dir)
            .output()
        {
            if output.status.success() {
                let log = String::from_utf8_lossy(&output.stdout);
                for line in log.lines() {
                    let parts: Vec<&str> = line.splitn(2, ' ').collect();
                    if parts.len() == 2 {
                        self.recent_changes.push(Change {
                            path: String::new(),
                            summary: parts[1].to_string(),
                            timestamp: String::new(),
                            author: parts[0].to_string(),
                        });
                    }
                }
            }
        }
    }

    fn detect_quality_gates(&mut self, project_dir: &Utf8Path) {
        let config_dir = project_dir.join(".lode");
        if !config_dir.exists() {
            return;
        }
        self.quality_gates.push(QualityGate {
            name: "conventions".to_string(),
            command: "lode check".to_string(),
            required: true,
            last_status: None,
        });
        self.quality_gates.push(QualityGate {
            name: "secrets".to_string(),
            command: "lode scan secrets".to_string(),
            required: true,
            last_status: None,
        });
        self.quality_gates.push(QualityGate {
            name: "build".to_string(),
            command: "lode build".to_string(),
            required: false,
            last_status: None,
        });
        self.quality_gates.push(QualityGate {
            name: "tests".to_string(),
            command: "lode test".to_string(),
            required: false,
            last_status: None,
        });
    }

    fn load_decisions(&mut self, project_dir: &Utf8Path) {
        let decisions_path = project_dir.join("_ctx_").join("ACTIVE_DECISIONS.md");
        if decisions_path.exists() {
            if let Ok(content) = fs::read_to_string(&decisions_path) {
                for line in content.lines() {
                    if line.starts_with("- [") || line.starts_with("* [") {
                        let clean = line.trim_start_matches("- [").trim_start_matches("* [");
                        let parts: Vec<&str> = clean.splitn(3, ']').collect();
                        if parts.len() >= 2 {
                            let status = parts[0].trim();
                            let title = parts[1].trim().trim_start_matches(" ");
                            self.decisions.push(Decision {
                                id: format!("D-{}", self.decisions.len() + 1),
                                title: title.to_string(),
                                description: String::new(),
                                date: String::new(),
                                status: if status == "x" { "accepted".to_string() } else { "open".to_string() },
                            });
                        }
                    }
                }
            }
        }
    }

    fn detect_dependencies(&mut self, project_dir: &Utf8Path) {
        let cargo = project_dir.join("Cargo.toml");
        if cargo.exists() {
            if let Ok(content) = fs::read_to_string(&cargo) {
                if let Ok(value) = content.parse::<toml::Value>() {
                    if let Some(deps) = value.get("dependencies").and_then(|d| d.as_table()) {
                        for (name, detail) in deps {
                            let version = detail
                                .get("version")
                                .and_then(|v| v.as_str())
                                .unwrap_or("*")
                                .to_string();
                            self.dependencies.push(Dependency {
                                name: name.clone(),
                                version,
                                kind: "runtime".to_string(),
                            });
                        }
                    }
                    if let Some(deps) = value.get("dev-dependencies").and_then(|d| d.as_table()) {
                        for (name, detail) in deps {
                            let version = detail
                                .get("version")
                                .and_then(|v| v.as_str())
                                .unwrap_or("*")
                                .to_string();
                            self.dependencies.push(Dependency {
                                name: name.clone(),
                                version,
                                kind: "dev".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    pub fn render_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str(&format!("# Context: {}\n\n", self.project_name));

        md.push_str("## Files\n\n");
        for file in &self.files {
            md.push_str(&format!("- `{}` — {}\n", file.path, file.summary));
        }
        md.push('\n');

        if !self.decisions.is_empty() {
            md.push_str("## Active Decisions\n\n");
            for d in &self.decisions {
                md.push_str(&format!("- [{}] {} ({})\n", if d.status == "accepted" { "x" } else { " " }, d.title, d.status));
            }
            md.push('\n');
        }

        if !self.quality_gates.is_empty() {
            md.push_str("## Quality Gates\n\n");
            md.push_str("| Gate | Command | Required |\n");
            md.push_str("|------|---------|----------|\n");
            for g in &self.quality_gates {
                md.push_str(&format!("| {} | `{}` | {} |\n", g.name, g.command, g.required));
            }
            md.push('\n');
        }

        if !self.commands.is_empty() {
            md.push_str("## Commands\n\n");
            for c in &self.commands {
                md.push_str(&format!("- `lode {}` — {}\n", c.name, c.description));
            }
            md.push('\n');
        }

        if !self.recent_changes.is_empty() {
            md.push_str("## Recent Changes\n\n");
            for c in &self.recent_changes {
                if c.path.is_empty() {
                    md.push_str(&format!("- {} ({})\n", c.summary, c.author));
                } else {
                    md.push_str(&format!("- `{}` — {} ({})\n", c.path, c.summary, c.author));
                }
            }
            md.push('\n');
        }

        md
    }

    pub fn generate(project_dir: &Utf8Path) -> Result<ContextPack> {
        let pack = Self::from_project(project_dir)?;
        let root = ValidatedRoot::new(project_dir)?;
        root.create_dir_all("_ctx_")?;

        let files: Vec<(&str, &str)> = vec![
            ("_ctx_/CONTEXT_INDEX.md", "Context Index"),
            ("_ctx_/PROJECT_SUMMARY.md", "Project Summary"),
            ("_ctx_/CURRENT_STATE.md", "Current State"),
            ("_ctx_/ARCHITECTURE_MAP.md", "Architecture Map"),
            ("_ctx_/QUALITY_GATES.md", "Quality Gates"),
            ("_ctx_/ACTIVE_DECISIONS.md", "Active Decisions"),
            ("_ctx_/OPEN_RISKS.md", "Open Risks"),
            ("_ctx_/RECENT_CHANGES.md", "Recent Changes"),
        ];

        for (rel_path, title) in &files {
            let full = project_dir.join(rel_path);
            if !full.exists() {
                let content = format!("# {title}\n\n_Automatically generated by `lode context build`._\n\n");
                root.write_atomic(rel_path.trim_start_matches("/"), &content)?;
            }
        }

        let summary_md = format!(
            "# Project Summary\n\n**Name:** {}\n\n**Last built:** {}\n\n**Files tracked:** {}\n\n**Decisions:** {}\n**Quality gates:** {}\n**Dependencies:** {}\n",
            pack.project_name,
            chrono_now(),
            pack.files.len(),
            pack.decisions.len(),
            pack.quality_gates.len(),
            pack.dependencies.len(),
        );
        root.write_atomic("_ctx_/PROJECT_SUMMARY.md", &summary_md)?;

        let gates_md = pack.render_markdown();
        root.write_atomic("_ctx_/QUALITY_GATES.md", &gates_md)?;

        Ok(pack)
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| LodeError::Message(e.to_string()))
    }

    pub fn to_compact_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| LodeError::Message(e.to_string()))
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("T{}", d.as_secs())
}

#[derive(Debug, Clone, Serialize)]
pub struct CompileEntry {
    pub path: String,
    pub priority: u32,
    pub estimated_tokens: usize,
    pub included: bool,
    pub char_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompileReport {
    pub budget_tokens: usize,
    pub total_estimated_tokens: usize,
    pub total_files: usize,
    pub included_files: usize,
    pub trimmed_files: usize,
    pub skipped_files: usize,
    pub entries: Vec<CompileEntry>,
    pub output_path: String,
}

pub fn estimate_token_count(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    let char_estimate = text.len() / 4;
    let word_count = text.split_whitespace().count();
    let word_estimate = (word_count as f64 * 1.33) as usize;
    std::cmp::max(char_estimate, word_estimate).max(1)
}

fn file_priority(name: &str) -> u32 {
    let lower = name.to_lowercase();
    if lower.contains("project_summary") || lower.contains("constraints") || lower.contains("project") {
        1
    } else if lower.contains("current_state") || lower.contains("plan") {
        2
    } else if lower.contains("architecture") || lower.contains("tasks") || lower.contains("index") {
        3
    } else if lower.contains("decisions") || lower.contains("memory") {
        4
    } else if lower.contains("quality") || lower.contains("gates") || lower.contains("review") {
        5
    } else if lower.contains("risks") || lower.contains("changes") || lower.contains("changelog") {
        6
    } else if lower.contains("claude") || lower.contains("agents") || lower.contains("codex") {
        7
    } else {
        8
    }
}

pub fn compile_context(
    project_dir: &Utf8Path,
    budget_tokens: Option<usize>,
) -> Result<CompileReport> {
    let budget = budget_tokens.unwrap_or(6000);
    let mut entries: Vec<CompileEntry> = Vec::new();

    let scan_dirs = [
        project_dir.join("_ctx_"),
        project_dir.join(".lode").join("context"),
    ];

    for scan_dir in &scan_dirs {
        if !scan_dir.exists() {
            continue;
        }
        if let Ok(dir_entries) = fs::read_dir(scan_dir.as_std_path()) {
            for entry in dir_entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if ext != "md" {
                            continue;
                        }
                    }
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        let rel = format!(
                            "{}/{}",
                            scan_dir
                                .file_name()
                                .map(|s| s.to_string())
                                .unwrap_or_default(),
                            name
                        );
                        if let Ok(content) = fs::read_to_string(&path) {
                            let estimated = estimate_token_count(&content);
                            let priority = file_priority(name);
                            entries.push(CompileEntry {
                                path: rel,
                                priority,
                                estimated_tokens: estimated,
                                included: false,
                                char_count: content.len(),
                            });
                        }
                    }
                }
            }
        }
    }

    let total_estimated: usize = entries.iter().map(|e| e.estimated_tokens).sum();

    entries.sort_by(|a, b| a.priority.cmp(&b.priority));

    let mut budget_remaining = budget;
    let mut included_count = 0;
    for entry in entries.iter_mut() {
        if entry.estimated_tokens <= budget_remaining {
            entry.included = true;
            budget_remaining = budget_remaining.saturating_sub(entry.estimated_tokens);
            included_count += 1;
        } else if entry.estimated_tokens > 0 && budget_remaining > 0 {
            entry.included = true;
            budget_remaining = 0;
            included_count += 1;
        }
    }

    let output_rel = Utf8PathBuf::from(".lode/context/COMPILED.md");
    let mut compiled = String::new();

    compiled.push_str("# Compiled Context\n\n");
    compiled.push_str(&format!(
        "> Token budget: **{}** tokens — Used: **{}** — Files: **{}**\n\n",
        budget,
        total_estimated.min(budget),
        included_count,
    ));

    let included_paths: Vec<&str> = entries
        .iter()
        .filter(|e| e.included)
        .map(|e| e.path.as_str())
        .collect();

    compiled.push_str("## Included Files\n\n");
    for path in &included_paths {
        compiled.push_str(&format!("- `{}`\n", path));
    }
    compiled.push('\n');

    let skipped: Vec<&CompileEntry> = entries.iter().filter(|e| !e.included).collect();
    if !skipped.is_empty() {
        compiled.push_str("## Skipped Files (budget exceeded)\n\n");
        for entry in &skipped {
            compiled.push_str(&format!(
                "- `{}` (~{} tokens, priority {})\n",
                entry.path, entry.estimated_tokens, entry.priority
            ));
        }
        compiled.push('\n');
    }

    compiled.push_str("---\n\n## Full Content\n\n");
    for entry in entries.iter().filter(|e| e.included) {
        let scan_dir = if entry.path.starts_with("_ctx_") {
            project_dir.join("_ctx_")
        } else {
            project_dir.join(".lode").join("context")
        };
        let name = entry.path.split('/').last().unwrap_or(&entry.path);
        let file_path = scan_dir.join(name);
        if let Ok(content) = fs::read_to_string(&file_path) {
            compiled.push_str(&format!("### {}\n\n", entry.path));
            compiled.push_str(&content);
            compiled.push_str("\n\n---\n\n");
        }
    }

    let root = ValidatedRoot::new(project_dir)?;
    root.create_dir_all(Utf8Path::new(".lode/context"))?;
    root.write_atomic(&output_rel, &compiled)?;

    entries.sort_by(|a, b| a.priority.cmp(&b.priority));

    let _ = add_managed_file(
        project_dir,
        &output_rel,
        ManagedBy::Context,
        "Compiled context pack with token budget",
    );

    Ok(CompileReport {
        budget_tokens: budget,
        total_estimated_tokens: total_estimated,
        total_files: entries.len(),
        included_files: entries.iter().filter(|e| e.included).count(),
        trimmed_files: 0,
        skipped_files: entries.iter().filter(|e| !e.included).count(),
        entries,
        output_path: output_rel.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn estimate_token_count_empty() {
        assert_eq!(estimate_token_count(""), 0);
    }

    #[test]
    fn estimate_token_count_basic() {
        let count = estimate_token_count("hello world");
        assert!(count >= 1);
        assert!(count <= 10);
    }

    #[test]
    fn estimate_token_count_long_text() {
        let text = "token ".repeat(100);
        let count = estimate_token_count(&text);
        // 600 chars / 4 = 150, 100 words * 1.33 = 133, max = 150
        assert!(count > 50);
    }

    #[test]
    fn file_priority_ordering() {
        let high = file_priority("PROJECT_SUMMARY.md");
        let mid = file_priority("TASKS.md");
        let low = file_priority("RECENT_CHANGES.md");
        assert!(high < mid);
        assert!(mid < low);
    }

    #[test]
    fn compile_context_empty_directory() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();

        let report = compile_context(&root, Some(1000)).unwrap();
        assert_eq!(report.total_files, 0);
        assert_eq!(report.included_files, 0);
        assert!(!report.output_path.is_empty());
    }

    #[test]
    fn compile_context_with_files() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();

        let ctx_dir = root.join("_ctx_");
        fs::create_dir_all(&ctx_dir).unwrap();
        fs::write(ctx_dir.join("PROJECT_SUMMARY.md"), "# My Project\n\nThis is a test project.\n").unwrap();
        fs::write(ctx_dir.join("TASKS.md"), "# Tasks\n\n- [ ] Task one\n- [ ] Task two\n").unwrap();
        fs::write(ctx_dir.join("RECENT_CHANGES.md"), "# Changes\n\n- Fixed bug\n").unwrap();

        let report = compile_context(&root, Some(5000)).unwrap();
        assert_eq!(report.total_files, 3);
        assert_eq!(report.included_files, 3);
        assert_eq!(report.skipped_files, 0);
    }

    #[test]
    fn compile_context_respects_budget() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();

        let ctx_dir = root.join("_ctx_");
        fs::create_dir_all(&ctx_dir).unwrap();
        let big_content = "Big content\n".repeat(1000);
        fs::write(ctx_dir.join("PROJECT_SUMMARY.md"), &big_content).unwrap();
        fs::write(ctx_dir.join("TASKS.md"), &big_content).unwrap();
        fs::write(ctx_dir.join("RECENT_CHANGES.md"), &big_content).unwrap();

        let report = compile_context(&root, Some(100)).unwrap();
        assert_eq!(report.total_files, 3);
        assert!(report.included_files < 3);
        assert!(report.skipped_files > 0);
    }

    #[test]
    fn compile_scans_lode_context_too() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();

        let lode_ctx = root.join(".lode").join("context");
        fs::create_dir_all(&lode_ctx).unwrap();
        fs::write(lode_ctx.join("PLAN.md"), "# Plan\n\n- Step 1\n").unwrap();
        fs::write(lode_ctx.join("CONSTRAINTS.md"), "# Constraints\n\nMust be fast.\n").unwrap();

        let report = compile_context(&root, Some(5000)).unwrap();
        assert_eq!(report.total_files, 2);
        assert_eq!(report.included_files, 2);
    }

    #[test]
    fn compile_report_has_correct_schema() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let ctx_dir = root.join("_ctx_");
        fs::create_dir_all(&ctx_dir).unwrap();
        fs::write(ctx_dir.join("PROJECT_SUMMARY.md"), "# Test\n").unwrap();

        let report = compile_context(&root, Some(1000)).unwrap();
        assert_eq!(report.budget_tokens, 1000);
        assert!(report.total_estimated_tokens > 0);
        assert!(report.output_path.contains("COMPILED.md"));
    }

    #[test]
    fn estimate_token_count_is_reasonable() {
        let text = "The quick brown fox jumps over the lazy dog. ";
        let count = estimate_token_count(text);
        // ~44 chars / 4 = 11 tokens, 10 words * 1.33 = 13 tokens
        assert!(count >= 5 && count <= 20, "count={count} for text={text:?}");
    }
}
