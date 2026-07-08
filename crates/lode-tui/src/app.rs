use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::sync::mpsc;
use std::time::Duration;

use crate::ipc::{DaemonEvent, DaemonIpc};
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
    pub events: Vec<DaemonEvent>,
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

pub enum AppResult {
    Continue,
    Quit,
}

pub struct App {
    pub active_pane: Pane,
    pub project_data: ProjectData,
    pub theme: Theme,
    #[allow(dead_code)]
    event_rx: mpsc::Receiver<KeyEvent>,
}

impl App {
    pub fn new(project_name: &str) -> Self {
        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || loop {
            if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                if let Ok(Event::Key(key)) = event::read() {
                    let _ = tx.send(key);
                }
            }
        });

        let data = Self::load_project_data(project_name);

        Self {
            active_pane: Pane::Overview,
            project_data: data,
            theme: crate::theme::dark_theme(),
            event_rx: rx,
        }
    }

    pub fn tick(&mut self) {
        if let Ok(mut ipc) = DaemonIpc::connect(&self.project_data.name) {
            while let Some(ev) = ipc.read_event() {
                self.project_data.events.push(ev);
            }
        }
    }

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
        ProjectData {
            name: name.to_string(),
            path: std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            profile: "default".to_string(),
            languages: vec!["Rust".to_string()],
            health_score: 85,
            checks: Checks {
                convention: true,
                secrets: false,
                license: true,
                env: true,
                readme: false,
            },
            metrics: Metrics {
                sparkline: vec![12, 15, 8, 20, 18, 25, 22, 30, 28, 35],
                coverage: 78.5,
                issues: 3,
                warnings: 7,
            },
            session_today: vec![
                SessionEntry {
                    time: "09:15".into(),
                    duration: "45m".into(),
                    files_changed: 5,
                },
                SessionEntry {
                    time: "14:30".into(),
                    duration: "1h 20m".into(),
                    files_changed: 12,
                },
            ],
            dir_bar: vec![
                ("src".into(), 850),
                ("tests".into(), 230),
                ("docs".into(), 120),
            ],
            heatmap: [
                [2, 3, 1, 4, 2, 0, 1],
                [3, 2, 4, 1, 3, 2, 0],
                [1, 0, 2, 3, 4, 1, 2],
                [4, 2, 1, 0, 3, 2, 1],
            ],
            events: vec![],
            deps_vulns: 0,
            deps_outdated: vec!["serde_json 1.0".into(), "crossterm 0.27".into()],
            audit_ok: true,
            file_tree: vec![
                FileNode {
                    name: "src".into(),
                    path: "src".into(),
                    is_dir: true,
                    violation: false,
                    signed: false,
                },
                FileNode {
                    name: "Cargo.toml".into(),
                    path: "Cargo.toml".into(),
                    is_dir: false,
                    violation: false,
                    signed: true,
                },
            ],
            violations: vec!["missing LICENSE file".into()],
            registry: vec![
                RegistryEntry {
                    name: "lode-core".into(),
                    score: 92,
                    status: "healthy".into(),
                },
                RegistryEntry {
                    name: "lode-tui".into(),
                    score: 85,
                    status: "healthy".into(),
                },
            ],
        }
    }
}
