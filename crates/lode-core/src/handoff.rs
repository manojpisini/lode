use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

use crate::{LodeError, Result, ValidatedRoot};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handoff {
    pub schema_version: u32,
    pub handoff_id: String,
    pub created_at: String,
    pub task: String,
    pub status: String,
    pub decisions: Vec<HandoffDecision>,
    pub changed_paths: Vec<String>,
    pub verification_performed: Vec<String>,
    pub remaining_risks: Vec<String>,
    pub next_action: String,
    pub context_ids: Vec<String>,
    pub plan_id: Option<String>,
    pub receipt_id: Option<String>,
    pub format: HandoffFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HandoffFormat {
    Pidgin,
    Markdown,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffDecision {
    pub id: String,
    pub description: String,
    pub rationale: String,
    pub alternatives: Vec<String>,
}

fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("T{}", d.as_secs())
}

fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("handoff_{:016x}", nanos)
}

impl Handoff {
    pub fn new(task: &str) -> Self {
        Self {
            schema_version: 1,
            handoff_id: generate_id(),
            created_at: now_iso(),
            task: task.to_string(),
            status: "in_progress".to_string(),
            decisions: Vec::new(),
            changed_paths: Vec::new(),
            verification_performed: Vec::new(),
            remaining_risks: Vec::new(),
            next_action: String::new(),
            context_ids: Vec::new(),
            plan_id: None,
            receipt_id: None,
            format: HandoffFormat::Pidgin,
        }
    }

    pub fn add_decision(&mut self, id: &str, desc: &str, rationale: &str, alternatives: &[String]) {
    let _ = alternatives;
        self.decisions.push(HandoffDecision {
            id: id.to_string(),
            description: desc.to_string(),
            rationale: rationale.to_string(),
            alternatives: alternatives.to_vec(),
        });
    }

    pub fn render_pidgin(&self) -> String {
        let mut p = String::new();

        p.push_str(&format!("TASK: {}\n", self.task));
        p.push_str(&format!("STATUS: {}\n", self.status));
        p.push('\n');

        if !self.decisions.is_empty() {
            p.push_str("DECISIONS:\n");
            for d in &self.decisions {
                p.push_str(&format!("  {}: {}  # {}\n", d.id, d.description, d.rationale));
            }
            p.push('\n');
        }

        if !self.changed_paths.is_empty() {
            p.push_str(&format!("CHANGED: {}\n\n", self.changed_paths.join(" ")));
        }

        if !self.verification_performed.is_empty() {
            p.push_str(&format!("VERIFIED: {}\n\n", self.verification_performed.join(" ")));
        }

        if !self.remaining_risks.is_empty() {
            p.push_str("RISKS:\n");
            for r in &self.remaining_risks {
                p.push_str(&format!("  - {}\n", r));
            }
            p.push('\n');
        }

        p.push_str(&format!("NEXT: {}\n", self.next_action));

        if let Some(ref pid) = self.plan_id {
            p.push_str(&format!("PLAN: {}\n", pid));
        }
        if let Some(ref rid) = self.receipt_id {
            p.push_str(&format!("RECEIPT: {}\n", rid));
        }

        p
    }

    pub fn render_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str(&format!("# Handoff: {}\n\n", self.task));
        md.push_str(&format!("**Status:** {}\n\n", self.status));

        if !self.decisions.is_empty() {
            md.push_str("## Decisions\n\n");
            for d in &self.decisions {
                md.push_str(&format!("### {}: {}\n\n", d.id, d.description));
                md.push_str(&format!("**Rationale:** {}\n\n", d.rationale));
                if !d.alternatives.is_empty() {
                    md.push_str("**Alternatives considered:**\n");
                    for a in &d.alternatives {
                        md.push_str(&format!("- {}\n", a));
                    }
                    md.push('\n');
                }
            }
        }

        if !self.changed_paths.is_empty() {
            md.push_str("## Changed Paths\n\n");
            for p in &self.changed_paths {
                md.push_str(&format!("- `{}`\n", p));
            }
            md.push('\n');
        }

        if !self.verification_performed.is_empty() {
            md.push_str("## Verification Performed\n\n");
            for v in &self.verification_performed {
                md.push_str(&format!("- `{}`\n", v));
            }
            md.push('\n');
        }

        if !self.remaining_risks.is_empty() {
            md.push_str("## Remaining Risks\n\n");
            for r in &self.remaining_risks {
                md.push_str(&format!("- {}\n", r));
            }
            md.push('\n');
        }

        md.push_str(&format!("## Next Action\n\n{}\n\n", self.next_action));

        if let Some(ref pid) = self.plan_id {
            md.push_str(&format!("**Plan:** `{}`\n\n", pid));
        }
        if let Some(ref rid) = self.receipt_id {
            md.push_str(&format!("**Receipt:** `{}`\n\n", rid));
        }
        if !self.context_ids.is_empty() {
            md.push_str("**Context IDs:**\n");
            for c in &self.context_ids {
                md.push_str(&format!("- `{}`\n", c));
            }
        }

        md
    }

    pub fn save(&self, project_dir: &Utf8Path) -> Result<Utf8PathBuf> {
        let root = ValidatedRoot::new(project_dir)?;
        root.create_dir_all(".lode/handoffs")?;
        let path = Utf8PathBuf::from(".lode/handoffs").join(format!("{}.json", self.handoff_id));

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| LodeError::Message(e.to_string()))?;
        root.write_atomic(&path, json)?;

        let md = self.render_markdown();
        let md_path = Utf8PathBuf::from("_ctx_").join(format!("HANDOFF_{}.md", &self.handoff_id[..8]));
        root.write_atomic(&md_path, md)?;

        Ok(project_dir.join(&path))
    }

    pub fn load(project_dir: &Utf8Path, handoff_id: &str) -> Result<Self> {
        let path = project_dir
            .join(".lode")
            .join("handoffs")
            .join(format!("{handoff_id}.json"));
        let raw = std::fs::read_to_string(&path).map_err(|e| LodeError::Io {
            path: std::path::PathBuf::from(path.as_str()),
            source: e,
        })?;
        serde_json::from_str(&raw).map_err(|e| LodeError::Message(e.to_string()))
    }

    pub fn list(project_dir: &Utf8Path) -> Result<Vec<String>> {
        let dir = project_dir.join(".lode").join("handoffs");
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut handoffs = Vec::new();
        for entry in std::fs::read_dir(&dir).map_err(|e| LodeError::Io {
            path: std::path::PathBuf::from(dir.as_str()),
            source: e,
        })? {
            let entry = entry.map_err(|e| LodeError::Io {
                path: std::path::PathBuf::from(dir.as_str()),
                source: e,
            })?;
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".json") {
                    handoffs.push(name.trim_end_matches(".json").to_string());
                }
            }
        }
        handoffs.sort();
        handoffs.reverse();
        Ok(handoffs)
    }
}
