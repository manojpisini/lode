use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

use crate::{process::validate_program, LodeError, Result, ValidatedRoot};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub schema_version: u32,
    pub plan_id: String,
    pub created_at: String,
    pub intent: String,
    pub profile: Option<String>,
    pub operations: Vec<Operation>,
    pub rollback_ops: Vec<Operation>,
    pub verification: Vec<String>,
    pub metadata: PlanMetadata,
    pub status: PlanStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlanStatus {
    Pending,
    Validated,
    Applied,
    RolledBack,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanMetadata {
    pub estimated_files: usize,
    pub source_profile: Option<String>,
    pub source_recipes: Vec<String>,
    pub source_commands: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Operation {
    CreateFile {
        path: String,
        content: String,
        template: Option<String>,
    },
    ModifyFile {
        path: String,
        search: String,
        replace: String,
    },
    DeleteFile {
        path: String,
    },
    RunCommand {
        command: String,
        description: String,
    },
    ApplyRecipe {
        name: String,
    },
    SetConfig {
        key: String,
        value: String,
    },
    RunMacro {
        name: String,
        args: HashMap<String, String>,
    },
}

impl Operation {
    pub fn description(&self) -> String {
        match self {
            Operation::CreateFile { path, template, .. } => {
                if template.is_some() {
                    format!("create {path} from template")
                } else {
                    format!("create {path}")
                }
            }
            Operation::ModifyFile { path, .. } => format!("modify {path}"),
            Operation::DeleteFile { path } => format!("delete {path}"),
            Operation::RunCommand { description, .. } => description.clone(),
            Operation::ApplyRecipe { name } => format!("apply recipe {name}"),
            Operation::SetConfig { key, value } => format!("set config {key}={value}"),
            Operation::RunMacro { name, .. } => format!("run macro {name}"),
        }
    }

    pub fn inverse(&self) -> Option<Operation> {
        match self {
            Operation::CreateFile { path, .. } => {
                Some(Operation::DeleteFile { path: path.clone() })
            }
            Operation::ModifyFile {
                path,
                search,
                replace,
            } => Some(Operation::ModifyFile {
                path: path.clone(),
                search: replace.clone(),
                replace: search.clone(),
            }),
            Operation::DeleteFile { .. } => None,
            Operation::RunCommand { .. } => None,
            Operation::ApplyRecipe { .. } => None,
            Operation::SetConfig { key, value: _ } => Some(Operation::SetConfig {
                key: key.clone(),
                value: "".to_string(),
            }),
            Operation::RunMacro { .. } => None,
        }
    }
}

fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("T{}", d.as_secs())
}

fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("plan_{:016x}", nanos)
}

impl Plan {
    pub fn new(intent: &str) -> Self {
        Self {
            schema_version: 1,
            plan_id: generate_id(),
            created_at: now_iso(),
            intent: intent.to_string(),
            profile: None,
            operations: Vec::new(),
            rollback_ops: Vec::new(),
            verification: Vec::new(),
            metadata: PlanMetadata {
                estimated_files: 0,
                source_profile: None,
                source_recipes: Vec::new(),
                source_commands: Vec::new(),
                warnings: Vec::new(),
            },
            status: PlanStatus::Pending,
        }
    }

    pub fn add_operation(&mut self, op: Operation) {
        if let Some(inverse) = op.inverse() {
            self.rollback_ops.push(inverse);
        }
        self.operations.push(op);
    }

    pub fn validate(&self, project_dir: &Utf8Path) -> Result<PlanValidation> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        for op in &self.operations {
            match op {
                Operation::CreateFile { path, .. } => {
                    let full = project_dir.join(path);
                    if full.exists() {
                        warnings.push(format!("{} already exists", path));
                    }
                    if let Some(parent) = full.parent() {
                        if !parent.exists() {
                            errors.push(format!("parent directory does not exist for {path}"));
                        }
                    }
                }
                Operation::ModifyFile { path, .. } => {
                    let full = project_dir.join(path);
                    if !full.exists() {
                        errors.push(format!("{} does not exist", path));
                    }
                }
                Operation::DeleteFile { path } => {
                    let full = project_dir.join(path);
                    if !full.exists() {
                        warnings.push(format!("{} does not exist", path));
                    }
                    if full.is_dir() {
                        let is_empty = full
                            .read_dir()
                            .map(|mut d| d.next().is_none())
                            .unwrap_or(false);
                        if !is_empty {
                            errors.push(format!("{} is a non-empty directory", path));
                        }
                    }
                }
                Operation::RunCommand { command, .. } => {
                    if command.is_empty() {
                        errors.push("run command is empty".to_string());
                    }
                    if command.contains('\0') {
                        errors.push("run command contains null byte".to_string());
                    }
                    if let Some(prog) = command.split_whitespace().next() {
                        if let Err(e) = validate_program(prog) {
                            errors.push(format!("run command program: {e}"));
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(PlanValidation {
            valid: errors.is_empty(),
            errors,
            warnings,
        })
    }

    pub fn apply(&self, project_dir: &Utf8Path, dry_run: bool) -> Result<ApplyReport> {
        let root = ValidatedRoot::new(project_dir)?;
        let mut report = ApplyReport::new(self.plan_id.clone());
        report.total = self.operations.len();

        for op in &self.operations {
            let description = op.description();
            match op {
                Operation::CreateFile { path, content, .. } => {
                    if dry_run {
                        report.dry_run_ops.push(description);
                        continue;
                    }
                    let full = project_dir.join(path);
                    if let Some(parent) = full.parent() {
                        let _ = root.create_dir_all(parent.as_str());
                    }
                    root.write_atomic(path.as_str(), content)?;
                    report.created.push(path.clone());
                }
                Operation::ModifyFile {
                    path,
                    search,
                    replace,
                } => {
                    if dry_run {
                        report.dry_run_ops.push(description);
                        continue;
                    }
                    let full = project_dir.join(path);
                    let content = fs::read_to_string(&full).map_err(|e| LodeError::Io {
                        path: PathBuf::from(full.as_str()),
                        source: e,
                    })?;
                    if !content.contains(search.as_str()) {
                        report
                            .warnings
                            .push(format!("search string not found in {path}"));
                        continue;
                    }
                    let new_content = content.replace(search.as_str(), replace.as_str());
                    root.write_atomic(path.as_str(), new_content)?;
                    report.modified.push(path.clone());
                }
                Operation::DeleteFile { path } => {
                    if dry_run {
                        report.dry_run_ops.push(description);
                        continue;
                    }
                    let full = project_dir.join(path);
                    if full.exists() {
                        fs::remove_file(&full).map_err(|e| LodeError::Io {
                            path: PathBuf::from(full.as_str()),
                            source: e,
                        })?;
                        report.deleted.push(path.clone());
                    }
                }
                Operation::RunCommand { command, .. } => {
                    if dry_run {
                        report.dry_run_ops.push(description);
                        continue;
                    }
                    if command.is_empty() || command.contains('\0') {
                        report.errors.push(format!(
                            "{description}: invalid command (empty or null byte)"
                        ));
                        continue;
                    }
                    if let Some(prog) = command.split_whitespace().next() {
                        if let Err(e) = validate_program(prog) {
                            report.errors.push(format!("{description}: {e}"));
                            continue;
                        }
                    }
                    let output = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(command)
                        .current_dir(project_dir)
                        .output()
                        .map_err(|e| LodeError::Message(format!("command failed: {e}")))?;
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        report.errors.push(format!("{description}: {stderr}"));
                    }
                    report.commands_executed.push(description);
                }
                Operation::ApplyRecipe { name } => {
                    report
                        .warnings
                        .push(format!("recipe {name} would be applied"));
                }
                Operation::SetConfig { key, value } => {
                    if dry_run {
                        report.dry_run_ops.push(description);
                        continue;
                    }
                    report.config_changes.push((key.clone(), value.clone()));
                }
                Operation::RunMacro { name, .. } => {
                    report.warnings.push(format!("macro {name} would be run"));
                }
            }
            report.completed += 1;
        }

        report.status = if report.errors.is_empty() {
            "applied".to_string()
        } else {
            "partial".to_string()
        };
        Ok(report)
    }

    pub fn rollback(&self, project_dir: &Utf8Path, dry_run: bool) -> Result<ApplyReport> {
        let mut rollback_plan = Plan::new(&format!("rollback {}", self.plan_id));
        for op in self.rollback_ops.iter().rev() {
            rollback_plan.add_operation(op.clone());
        }
        rollback_plan.apply(project_dir, dry_run)
    }

    pub fn save(&self, project_dir: &Utf8Path) -> Result<Utf8PathBuf> {
        let dir = project_dir.join(".lode").join("plans");
        let root = ValidatedRoot::new(project_dir)?;
        root.create_dir_all(".lode/plans")?;
        let path = dir.join(format!("{}.json", self.plan_id));
        let json =
            serde_json::to_string_pretty(self).map_err(|e| LodeError::Message(e.to_string()))?;
        root.write_atomic(format!(".lode/plans/{}.json", self.plan_id), json)?;
        Ok(path)
    }

    pub fn load(project_dir: &Utf8Path, plan_id: &str) -> Result<Self> {
        let path = project_dir
            .join(".lode")
            .join("plans")
            .join(format!("{plan_id}.json"));
        let raw = fs::read_to_string(&path).map_err(|e| LodeError::Io {
            path: PathBuf::from(path.as_str()),
            source: e,
        })?;
        serde_json::from_str(&raw).map_err(|e| LodeError::Message(e.to_string()))
    }

    pub fn list(project_dir: &Utf8Path) -> Result<Vec<String>> {
        let dir = project_dir.join(".lode").join("plans");
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut plans = Vec::new();
        for entry in fs::read_dir(&dir).map_err(|e| LodeError::Io {
            path: PathBuf::from(dir.as_str()),
            source: e,
        })? {
            let entry = entry.map_err(|e| LodeError::Io {
                path: PathBuf::from(dir.as_str()),
                source: e,
            })?;
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".json") {
                    plans.push(name.trim_end_matches(".json").to_string());
                }
            }
        }
        plans.sort();
        Ok(plans)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanValidation {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyReport {
    pub plan_id: String,
    pub status: String,
    pub total: usize,
    pub completed: usize,
    pub created: Vec<String>,
    pub modified: Vec<String>,
    pub deleted: Vec<String>,
    pub commands_executed: Vec<String>,
    pub config_changes: Vec<(String, String)>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub dry_run_ops: Vec<String>,
}

impl ApplyReport {
    pub fn new(plan_id: String) -> Self {
        Self {
            plan_id,
            status: "pending".to_string(),
            total: 0,
            completed: 0,
            created: Vec::new(),
            modified: Vec::new(),
            deleted: Vec::new(),
            commands_executed: Vec::new(),
            config_changes: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            dry_run_ops: Vec::new(),
        }
    }
}
