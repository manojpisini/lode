use std::{fs, path::PathBuf};

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

use crate::{LodeError, Result, ValidatedRoot};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConfig {
    pub enabled: bool,
    pub idle_threshold_s: u64,
    pub min_session_s: u64,
    pub track_by_dir: bool,
    pub history_days: u32,
}

impl Default for TimeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            idle_threshold_s: 300,
            min_session_s: 60,
            track_by_dir: true,
            history_days: 90,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSession {
    pub started_at: String,
    pub ended_at: String,
    pub seconds: u64,
    pub project: String,
    pub file: Option<String>,
    pub task: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeLog {
    pub sessions: Vec<TimeSession>,
}

impl Default for TimeLog {
    fn default() -> Self {
        Self {
            sessions: Vec::new(),
        }
    }
}

fn log_path(project_dir: &Utf8Path) -> Utf8PathBuf {
    project_dir.join(".lode").join("time_log.json")
}

pub fn load_time_log(project_dir: &Utf8Path) -> Result<TimeLog> {
    let path = log_path(project_dir);
    if !path.exists() {
        return Ok(TimeLog::default());
    }
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: PathBuf::from(path.as_str()),
        source,
    })?;
    serde_json::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))
}

pub fn save_time_log(project_dir: &Utf8Path, log: &TimeLog) -> Result<()> {
    let root = ValidatedRoot::new(project_dir)?;
    root.create_dir_all(".lode")?;
    let raw =
        serde_json::to_string_pretty(log).map_err(|error| LodeError::Message(error.to_string()))?;
    root.write_atomic(".lode/time_log.json", raw)?;
    Ok(())
}

pub fn add_session(project_dir: &Utf8Path, session: TimeSession) -> Result<()> {
    let mut log = load_time_log(project_dir)?;
    log.sessions.push(session);
    save_time_log(project_dir, &log)
}

pub fn time_today(project_dir: &Utf8Path) -> Result<u64> {
    let log = load_time_log(project_dir)?;
    let today = chrono_date_string();
    let total: u64 = log
        .sessions
        .iter()
        .filter(|s| s.ended_at.starts_with(&today))
        .map(|s| s.seconds)
        .sum();
    Ok(total)
}

pub fn time_report(project_dir: &Utf8Path, since: &str) -> Result<TimeLog> {
    let log = load_time_log(project_dir)?;
    let filtered: Vec<TimeSession> = log
        .sessions
        .into_iter()
        .filter(|s| s.ended_at.as_str() >= since)
        .collect();
    Ok(TimeLog { sessions: filtered })
}

fn chrono_date_string() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let days = now.as_secs() / 86400;
    let (y, m, d) = days_to_ymd(days as i64 + 719468);
    format!("{y:04}-{m:02}-{d:02}")
}

fn days_to_ymd(g: i64) -> (i64, u32, u32) {
    let y = (10000 * g + 14780) / 3652425;
    let mut doy = g - (365 * y + y / 4 - y / 100 + y / 400);
    if doy < 0 {
        let ly = y - 1;
        doy = g - (365 * ly + ly / 4 - ly / 100 + ly / 400);
    }
    let mi = (100 * doy + 52) / 3060;
    let month = (mi + 2) % 12 + 1;
    let year = y + (mi + 2) / 12;
    let day = doy - (mi * 306 + 5) / 10 + 1;
    (year, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_session(project: &str, seconds: u64, ended: &str) -> TimeSession {
        TimeSession {
            started_at: String::new(),
            ended_at: ended.to_string(),
            seconds,
            project: project.to_string(),
            file: None,
            task: None,
        }
    }

    #[test]
    fn load_save_round_trip() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let log = TimeLog {
            sessions: vec![make_session("proj", 120, "2026-01-15")],
        };
        save_time_log(&root, &log).unwrap();
        let loaded = load_time_log(&root).unwrap();
        assert_eq!(loaded.sessions.len(), 1);
        assert_eq!(loaded.sessions[0].project, "proj");
    }

    #[test]
    fn add_session_appends() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        add_session(&root, make_session("a", 10, "2026-01-01")).unwrap();
        add_session(&root, make_session("b", 20, "2026-01-02")).unwrap();
        let log = load_time_log(&root).unwrap();
        assert_eq!(log.sessions.len(), 2);
    }

    #[test]
    fn time_report_filters_correctly() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        add_session(&root, make_session("a", 10, "2026-01-01")).unwrap();
        add_session(&root, make_session("b", 20, "2026-06-15")).unwrap();
        let report = time_report(&root, "2026-03-01").unwrap();
        assert_eq!(report.sessions.len(), 1);
        assert_eq!(report.sessions[0].project, "b");
    }

    #[test]
    fn load_returns_empty_log_when_missing() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let log = load_time_log(&root).unwrap();
        assert!(log.sessions.is_empty());
    }
}
