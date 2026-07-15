use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::error::{LodeError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    pub id: String,
    pub description: String,
    pub kind: String,
    pub created_at: u64,
    pub applied: bool,
    pub rollback_steps: Vec<MigrationStep>,
    pub apply_steps: Vec<MigrationStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStep {
    pub action: String,
    pub target: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    pub migrations: Vec<Migration>,
    pub state: HashMap<String, bool>,
}

fn migrations_path() -> Result<std::path::PathBuf> {
    let dir = crate::install::global_asset_dir("state")?;
    Ok(dir.join("migrations.json").into())
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::from_secs(0))
        .as_secs()
}

pub fn list_migrations() -> Result<MigrationPlan> {
    let path = migrations_path()?;
    if !path.exists() {
        return Ok(MigrationPlan {
            migrations: Vec::new(),
            state: HashMap::new(),
        });
    }
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.clone(),
        source,
    })?;
    serde_json::from_str(&raw).map_err(|e| LodeError::Message(e.to_string()))
}

pub fn save_plan(plan: &MigrationPlan) -> Result<()> {
    let path = migrations_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LodeError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let raw = serde_json::to_string_pretty(plan).map_err(|e| LodeError::Message(e.to_string()))?;
    fs::write(&path, &raw).map_err(|source| LodeError::Io {
        path: path.clone(),
        source,
    })?;
    Ok(())
}

pub fn plan_migration(description: &str, kind: &str) -> Result<String> {
    let mut plan = list_migrations()?;
    let id = format!("mig-{:x}", now_secs());
    plan.migrations.push(Migration {
        id: id.clone(),
        description: description.to_string(),
        kind: kind.to_string(),
        created_at: now_secs(),
        applied: false,
        rollback_steps: Vec::new(),
        apply_steps: Vec::new(),
    });
    plan.state.insert(id.clone(), false);
    save_plan(&plan)?;
    Ok(id)
}

pub fn apply_migration(id: &str) -> Result<bool> {
    let mut plan = list_migrations()?;
    let migration = plan
        .migrations
        .iter_mut()
        .find(|m| m.id == id)
        .ok_or_else(|| LodeError::Message(format!("migration not found: {id}")))?;
    if migration.applied {
        return Ok(false);
    }
    migration.applied = true;
    plan.state.insert(id.to_string(), true);
    save_plan(&plan)?;
    Ok(true)
}

pub fn rollback_migration(id: &str) -> Result<bool> {
    let mut plan = list_migrations()?;
    let migration = plan
        .migrations
        .iter_mut()
        .find(|m| m.id == id)
        .ok_or_else(|| LodeError::Message(format!("migration not found: {id}")))?;
    if !migration.applied {
        return Ok(false);
    }
    migration.applied = false;
    plan.state.insert(id.to_string(), false);
    save_plan(&plan)?;
    Ok(true)
}
