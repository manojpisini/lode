use crossterm::event::{KeyCode, KeyEvent};

use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Overview,
    Metrics,
    Time,
    Activity,
    Deps,
    Files,
    Registry,
}

impl Pane {
    pub fn all() -> &'static [Pane] {
        &[
            Pane::Overview,
            Pane::Metrics,
            Pane::Time,
            Pane::Activity,
            Pane::Deps,
            Pane::Files,
            Pane::Registry,
        ]
    }

    pub fn label(&self) -> &str {
        match self {
            Pane::Overview => "Overview",
            Pane::Metrics => "Metrics",
            Pane::Time => "Time",
            Pane::Activity => "Activity",
            Pane::Deps => "Deps",
            Pane::Files => "Files",
            Pane::Registry => "Registry",
        }
    }

    pub fn key_hint(&self) -> &str {
        match self {
            Pane::Overview => "1",
            Pane::Metrics => "2",
            Pane::Time => "3",
            Pane::Activity => "4",
            Pane::Deps => "5",
            Pane::Files => "6",
            Pane::Registry => "7",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectData {
    pub name: String,
    pub path: String,
    pub profile: String,
    pub languages: Vec<String>,
    pub health_score: u8,
    pub checks: Checks,
    pub metrics: Metrics,
    pub session_today: Vec<SessionEntry>,
    pub dir_bar: Vec<(String, u64)>,
    pub heatmap: [[u8; 7]; 4],
    pub deps_vulns: usize,
    pub deps_outdated: Vec<String>,
    pub audit_ok: bool,
    pub file_tree: Vec<FileNode>,
    pub violations: Vec<String>,
    pub registry: Vec<RegistryEntry>,
}

#[derive(Debug, Clone, Default)]
pub struct Checks {
    pub convention: bool,
    pub secrets: bool,
    pub license: bool,
    pub env: bool,
    pub readme: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Metrics {
    pub sparkline: Vec<u64>,
    pub coverage: f64,
    pub issues: u32,
    pub warnings: u32,
}

#[derive(Debug, Clone)]
pub struct SessionEntry {
    pub time: String,
    pub duration: String,
    pub files_changed: u32,
}

#[derive(Debug, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub violation: bool,
    pub signed: bool,
}

#[derive(Debug, Clone)]
pub struct RegistryEntry {
    pub name: String,
    pub score: u8,
    pub status: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppResult {
    Continue,
    Quit,
}

pub struct App {
    pub active_pane: Pane,
    pub project_data: ProjectData,
    pub theme: Theme,
}

impl App {
    pub fn new(project_name: &str) -> Self {
        let data = Self::load_project_data(project_name);

        Self {
            active_pane: Pane::Overview,
            project_data: data,
            theme: crate::theme::dark_theme(),
        }
    }

    pub fn tick(&mut self) {}

    pub fn handle_key(&mut self, key: KeyEvent) -> AppResult {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => AppResult::Quit,
            KeyCode::Char('1') => {
                self.active_pane = Pane::Overview;
                AppResult::Continue
            }
            KeyCode::Char('2') => {
                self.active_pane = Pane::Metrics;
                AppResult::Continue
            }
            KeyCode::Char('3') => {
                self.active_pane = Pane::Time;
                AppResult::Continue
            }
            KeyCode::Char('4') => {
                self.active_pane = Pane::Activity;
                AppResult::Continue
            }
            KeyCode::Char('5') => {
                self.active_pane = Pane::Deps;
                AppResult::Continue
            }
            KeyCode::Char('6') => {
                self.active_pane = Pane::Files;
                AppResult::Continue
            }
            KeyCode::Char('7') => {
                self.active_pane = Pane::Registry;
                AppResult::Continue
            }
            KeyCode::Tab => {
                let panes = Pane::all();
                let idx = panes
                    .iter()
                    .position(|p| *p == self.active_pane)
                    .unwrap_or(0);
                self.active_pane = panes[(idx + 1) % panes.len()];
                AppResult::Continue
            }
            KeyCode::BackTab => {
                let panes = Pane::all();
                let idx = panes
                    .iter()
                    .position(|p| *p == self.active_pane)
                    .unwrap_or(0);
                self.active_pane = panes[(idx + panes.len() - 1) % panes.len()];
                AppResult::Continue
            }
            _ => AppResult::Continue,
        }
    }

    fn load_project_data(name: &str) -> ProjectData {
        let cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let utf8_path = camino::Utf8Path::new(&cwd);
        let path = std::path::Path::new(&cwd);
        let config = lode_core::default_config();

        let profile = lode_core::load_registry()
            .ok()
            .and_then(|reg| reg.projects.into_iter().find(|p| p.name == name))
            .map(|p| p.profile.clone())
            .unwrap_or_else(|| "default".to_string());

        let mut languages = Vec::new();
        if path.join("Cargo.toml").exists() {
            languages.push("Rust".to_string());
        }
        if path.join("package.json").exists() {
            languages.push("JavaScript".to_string());
        }
        if path.join("pyproject.toml").exists() || path.join("requirements.txt").exists() {
            languages.push("Python".to_string());
        }
        if path.join("go.mod").exists() {
            languages.push("Go".to_string());
        }
        if languages.is_empty() {
            if let Ok(entries) = std::fs::read_dir(path.join("src")) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    let ext = entry_path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");
                    let lang = match ext {
                        "rs" => "Rust",
                        "ts" | "tsx" => "TypeScript",
                        "js" | "jsx" => "JavaScript",
                        "py" => "Python",
                        "go" => "Go",
                        "java" => "Java",
                        "rb" => "Ruby",
                        "kt" => "Kotlin",
                        "c" | "h" => "C",
                        "cpp" | "hpp" => "C++",
                        "swift" => "Swift",
                        _ => continue,
                    };
                    languages.push(lang.to_string());
                    break;
                }
            }
        }
        if languages.is_empty() {
            languages.push("Unknown".to_string());
        }

        let convention_report = lode_core::check_path(utf8_path, &config).ok();
        let convention_ok = convention_report
            .as_ref()
            .map(|r| r.is_clean())
            .unwrap_or(true);
        let convention_violations = convention_report.map(|r| r.violations).unwrap_or_default();

        let secret_report = lode_core::scan_secrets(utf8_path).ok();
        let secrets_ok = secret_report
            .as_ref()
            .map(|r| r.findings.is_empty())
            .unwrap_or(true);
        let secret_findings = secret_report.map(|r| r.findings).unwrap_or_default();

        let license_ok = path.join("LICENSE").exists();
        let readme_ok = path.join("README.md").exists();
        let env_ok = path.join(".env").exists() || path.join(".env.example").exists();

        let audit = lode_core::audit_project(utf8_path, &config).ok();
        let health_score = audit.as_ref().map(|r| r.score).unwrap_or(85);

        let issues_count = convention_violations.len() as u32 + secret_findings.len() as u32;

        let mut dir_bar = Vec::new();
        for dir_name in &["src", "tests", "docs", "examples", "assets"] {
            let dir_path = path.join(dir_name);
            if dir_path.is_dir() {
                let size = dir_size(&dir_path).unwrap_or(0);
                dir_bar.push((dir_name.to_string(), (size / 1024).max(1)));
            }
        }

        let mut file_tree = Vec::new();
        if let Ok(entries) = std::fs::read_dir(path) {
            let mut dirs: Vec<_> = entries.flatten().collect();
            dirs.sort_by_key(|e| e.file_name());
            for entry in dirs {
                let entry_name = entry.file_name().to_string_lossy().to_string();
                if entry_name.starts_with('.')
                    || entry_name == "target"
                    || entry_name == "node_modules"
                {
                    continue;
                }
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                let viol = convention_violations
                    .iter()
                    .any(|v| v.path.as_str().ends_with(&entry_name));
                file_tree.push(FileNode {
                    name: entry_name.clone(),
                    path: entry.path().display().to_string(),
                    is_dir,
                    violation: viol,
                    signed: false,
                });
            }
        }

        let mut violations: Vec<String> = Vec::new();
        for v in &convention_violations {
            violations.push(format!("convention: {}", v.path));
        }
        for s in &secret_findings {
            violations.push(format!("secret: {}:{}", s.path, s.line));
        }
        if !license_ok {
            violations.push("missing LICENSE file".to_string());
        }
        if !readme_ok {
            violations.push("missing README.md file".to_string());
        }

        let registry = lode_core::load_registry()
            .ok()
            .map(|reg| {
                reg.projects
                    .into_iter()
                    .map(|p| RegistryEntry {
                        name: p.name,
                        score: health_score,
                        status: if health_score >= 70 {
                            "healthy".into()
                        } else {
                            "degraded".into()
                        },
                    })
                    .collect()
            })
            .unwrap_or_default();

        ProjectData {
            name: name.to_string(),
            path: cwd,
            profile,
            languages,
            health_score,
            checks: Checks {
                convention: convention_ok,
                secrets: secrets_ok,
                license: license_ok,
                env: env_ok,
                readme: readme_ok,
            },
            metrics: Metrics {
                sparkline: vec![12, 15, 8, 20, 18, 25, 22, 30, 28, 35],
                coverage: 78.5,
                issues: issues_count,
                warnings: 0,
            },
            session_today: vec![],
            dir_bar,
            heatmap: [
                [2, 3, 1, 4, 2, 0, 1],
                [3, 2, 4, 1, 3, 2, 0],
                [1, 0, 2, 3, 4, 1, 2],
                [4, 2, 1, 0, 3, 2, 1],
            ],
            deps_vulns: 0,
            deps_outdated: vec![],
            audit_ok: true,
            file_tree,
            violations,
            registry,
        }
    }
}

fn dir_size(dir: &std::path::Path) -> std::io::Result<u64> {
    let mut total = 0u64;
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if matches!(name, ".git" | "target" | "node_modules" | ".venv") {
                    continue;
                }
                total += dir_size(&path)?;
            } else {
                total += entry.metadata()?.len();
            }
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    #[test]
    fn pane_all_contains_all_panes() {
        let panes = Pane::all();
        assert_eq!(panes.len(), 7);
        assert!(panes.contains(&Pane::Overview));
        assert!(panes.contains(&Pane::Metrics));
        assert!(panes.contains(&Pane::Time));
        assert!(panes.contains(&Pane::Activity));
        assert!(panes.contains(&Pane::Deps));
        assert!(panes.contains(&Pane::Files));
        assert!(panes.contains(&Pane::Registry));
    }

    #[test]
    fn pane_labels_match() {
        assert_eq!(Pane::Overview.label(), "Overview");
        assert_eq!(Pane::Metrics.label(), "Metrics");
        assert_eq!(Pane::Registry.label(), "Registry");
    }

    #[test]
    fn pane_key_hints_match() {
        assert_eq!(Pane::Overview.key_hint(), "1");
        assert_eq!(Pane::Registry.key_hint(), "7");
    }

    #[test]
    fn app_new_sets_default_pane() {
        let app = App::new("test-project");
        assert_eq!(app.active_pane, Pane::Overview);
        assert_eq!(app.project_data.name, "test-project");
    }

    #[test]
    fn handle_key_q_quits() {
        let mut app = App::new("test");
        let result = app.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
        assert_eq!(result, AppResult::Quit);
    }

    #[test]
    fn handle_key_esc_quits() {
        let mut app = App::new("test");
        let result = app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(result, AppResult::Quit);
    }

    #[test]
    fn handle_key_digit_switches_pane() {
        let mut app = App::new("test");
        let result = app.handle_key(KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE));
        assert_eq!(result, AppResult::Continue);
        assert_eq!(app.active_pane, Pane::Time);
    }

    #[test]
    fn handle_key_tab_cycles_forward() {
        let mut app = App::new("test");
        app.active_pane = Pane::Metrics;
        let _ = app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(app.active_pane, Pane::Time);
    }

    #[test]
    fn handle_key_backtab_cycles_backward() {
        let mut app = App::new("test");
        app.active_pane = Pane::Files;
        let _ = app.handle_key(KeyEvent::new(KeyCode::BackTab, KeyModifiers::NONE));
        assert_eq!(app.active_pane, Pane::Deps);
    }

    #[test]
    fn handle_key_unhandled_continues() {
        let mut app = App::new("test");
        let result = app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(result, AppResult::Continue);
    }

    #[test]
    fn tab_wraps_from_last_to_first() {
        let mut app = App::new("test");
        app.active_pane = Pane::Registry;
        let _ = app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(app.active_pane, Pane::Overview);
    }

    #[test]
    fn backtab_wraps_from_first_to_last() {
        let mut app = App::new("test");
        app.active_pane = Pane::Overview;
        let _ = app.handle_key(KeyEvent::new(KeyCode::BackTab, KeyModifiers::NONE));
        assert_eq!(app.active_pane, Pane::Registry);
    }
}
