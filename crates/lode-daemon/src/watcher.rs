use std::path::PathBuf;
use std::time::Duration;

use camino::Utf8PathBuf;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WatcherError {
    #[error("Failed to initialize watcher: {0}")]
    InitFailed(String),
    #[error("Watcher already running")]
    AlreadyRunning,
    #[error("Watcher not running")]
    NotRunning,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WatchEvent {
    Create(PathBuf),
    Modify(PathBuf),
    Rename { from: PathBuf, to: PathBuf },
    Delete(PathBuf),
}

impl WatchEvent {
    pub fn path(&self) -> &PathBuf {
        match self {
            WatchEvent::Create(p) | WatchEvent::Modify(p) | WatchEvent::Delete(p) => p,
            WatchEvent::Rename { from, .. } => from,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherConfig {
    pub debounce_ms: u64,
    pub no_rename: bool,
    pub no_sign: bool,
    pub no_stamp: bool,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            debounce_ms: 200,
            no_rename: false,
            no_sign: false,
            no_stamp: false,
        }
    }
}

pub struct DaemonWatcher {
    watcher: RecommendedWatcher,
    rx: std::sync::mpsc::Receiver<Result<Event, notify::Error>>,
    project_dir: Utf8PathBuf,
    config: WatcherConfig,
    running: bool,
}

impl DaemonWatcher {
    pub fn start(project_dir: Utf8PathBuf, config: WatcherConfig) -> Result<Self, WatcherError> {
        let (tx, rx) = std::sync::mpsc::channel();

        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                let _ = tx.send(res);
            },
            notify::Config::default().with_poll_interval(Duration::from_millis(config.debounce_ms)),
        )
        .map_err(|e| WatcherError::InitFailed(e.to_string()))?;

        Ok(Self {
            watcher,
            rx,
            project_dir,
            config,
            running: false,
        })
    }

    pub fn watch(&mut self) -> Result<(), WatcherError> {
        if self.running {
            return Err(WatcherError::AlreadyRunning);
        }

        self.watcher
            .watch(self.project_dir.as_std_path(), RecursiveMode::Recursive)
            .map_err(|e| WatcherError::InitFailed(e.to_string()))?;

        self.running = true;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), WatcherError> {
        if !self.running {
            return Err(WatcherError::NotRunning);
        }

        self.watcher
            .unwatch(self.project_dir.as_std_path())
            .map_err(|e| WatcherError::InitFailed(e.to_string()))?;

        self.running = false;
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn receive_event(&self) -> Option<WatchEvent> {
        match self.rx.try_recv() {
            Ok(Ok(event)) => self.map_event(event),
            Ok(Err(e)) => {
                eprintln!("Watcher error: {e}");
                None
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => None,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => None,
        }
    }

    fn map_event(&self, event: Event) -> Option<WatchEvent> {
        match event.kind {
            EventKind::Create(_) => event.paths.first().map(|p| WatchEvent::Create(p.clone())),
            EventKind::Modify(notify::event::ModifyKind::Name(notify::event::RenameMode::Both)) => {
                if self.config.no_rename || event.paths.len() < 2 {
                    None
                } else {
                    let from = event.paths[0].clone();
                    let to = event.paths[1].clone();
                    Some(WatchEvent::Rename { from, to })
                }
            }
            EventKind::Modify(_) => event.paths.first().map(|p| WatchEvent::Modify(p.clone())),
            EventKind::Remove(_) => event.paths.first().map(|p| WatchEvent::Delete(p.clone())),
            _ => None,
        }
    }
}
