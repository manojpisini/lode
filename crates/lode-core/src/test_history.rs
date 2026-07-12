use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::{LodeError, Result, ValidatedRoot};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestHistory {
    pub runs: Vec<TestRun>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestRun {
    pub timestamp: String,
    pub passed: usize,
    pub failed: usize,
    pub duration_ms: u64,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestHistoryConfig {
    pub max_runs: usize,
}

impl Default for TestHistoryConfig {
    fn default() -> Self {
        Self { max_runs: 100 }
    }
}

fn history_path(project_dir: &std::path::Path) -> PathBuf {
    project_dir.join(".lode").join("test_history.toml")
}

pub fn load_test_history(project_dir: &std::path::Path) -> Result<TestHistory> {
    let path = history_path(project_dir);
    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            return Ok(TestHistory { runs: Vec::new() });
        }
        Err(source) => {
            return Err(LodeError::Io {
                path: path.clone(),
                source,
            });
        }
    };
    toml::from_str(&raw).map_err(|source| LodeError::TomlDeserialize {
        path,
        source: Box::new(source),
    })
}

pub fn save_test_history(project_dir: &std::path::Path, history: &TestHistory) -> Result<()> {
    let root = ValidatedRoot::new(project_dir)?;
    root.create_dir_all(".lode")?;
    let raw = toml::to_string_pretty(history)?;
    root.write_atomic(".lode/test_history.toml", raw)?;
    Ok(())
}

pub fn add_test_run(
    project_dir: &std::path::Path,
    run: TestRun,
    config: &TestHistoryConfig,
) -> Result<()> {
    let mut history = match load_test_history(project_dir) {
        Ok(h) => h,
        Err(_e) => {
            let path = history_path(project_dir);
            if path.exists() {
                let backup = path.with_extension("toml.bak");
                std::fs::rename(&path, &backup).ok();
                eprintln!(
                    "lode: warning: corrupted test history backed up to {:?}",
                    backup
                );
            }
            TestHistory { runs: Vec::new() }
        }
    };
    history.runs.push(run);
    if history.runs.len() > config.max_runs {
        let drain_count = history.runs.len() - config.max_runs;
        history.runs.drain(..drain_count);
    }
    save_test_history(project_dir, &history)
}

pub fn timestamp_now() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{seconds}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_project() -> (tempfile::TempDir, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().to_path_buf();
        (temp, dir)
    }

    #[test]
    fn load_returns_empty_history_when_file_missing() {
        let (_temp, dir) = temp_project();
        let history = load_test_history(&dir).unwrap();
        assert!(history.runs.is_empty());
    }

    #[test]
    fn save_and_load_round_trip() {
        let (_temp, dir) = temp_project();
        let history = TestHistory {
            runs: vec![TestRun {
                timestamp: "unix:1000".to_string(),
                passed: 10,
                failed: 0,
                duration_ms: 500,
                command: "cargo test".to_string(),
            }],
        };
        save_test_history(&dir, &history).unwrap();
        let loaded = load_test_history(&dir).unwrap();
        assert_eq!(loaded, history);
    }

    #[test]
    fn add_test_run_appends_and_trims() {
        let (_temp, dir) = temp_project();
        let config = TestHistoryConfig { max_runs: 2 };
        for i in 0..4 {
            let run = TestRun {
                timestamp: format!("unix:{i}"),
                passed: i,
                failed: 0,
                duration_ms: 100,
                command: "cargo test".to_string(),
            };
            add_test_run(&dir, run, &config).unwrap();
        }
        let history = load_test_history(&dir).unwrap();
        assert_eq!(history.runs.len(), 2);
        assert_eq!(history.runs[0].timestamp, "unix:2");
        assert_eq!(history.runs[1].timestamp, "unix:3");
    }

    #[test]
    fn default_config_has_max_100() {
        let config = TestHistoryConfig::default();
        assert_eq!(config.max_runs, 100);
    }
}
