use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use thiserror::Error;
use tokio::sync::mpsc;
use tokio::time::sleep;

#[derive(Error, Debug)]
pub enum IdleError {
    #[error("Watchdog error: {0}")]
    WatchdogFailed(String),
}

pub struct IdleWatchdog {
    timeout: Duration,
    last_event: Arc<tokio::sync::RwLock<Instant>>,
    shutdown_sender: mpsc::Sender<()>,
    running: Arc<AtomicBool>,
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl IdleWatchdog {
    pub fn new(timeout_secs: u64, shutdown_sender: mpsc::Sender<()>) -> Self {
        Self {
            timeout: Duration::from_secs(timeout_secs),
            last_event: Arc::new(tokio::sync::RwLock::new(Instant::now())),
            shutdown_sender,
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    pub async fn start(&mut self) -> Result<(), IdleError> {
        if self.running.load(Ordering::SeqCst) {
            return Err(IdleError::WatchdogFailed("Already running".to_string()));
        }

        self.running.store(true, Ordering::SeqCst);

        let timeout = self.timeout;
        let last_event = Arc::clone(&self.last_event);
        let running = Arc::clone(&self.running);
        let shutdown_sender = self.shutdown_sender.clone();

        let handle = tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(1)).await;

                if !running.load(Ordering::SeqCst) {
                    break;
                }

                let elapsed = {
                    let last = last_event.read().await;
                    last.elapsed()
                };

                if elapsed >= timeout {
                    eprintln!("Idle timeout reached, shutting down");
                    let _ = shutdown_sender.send(()).await;
                    break;
                }
            }
        });

        self.handle = Some(handle);
        Ok(())
    }

    pub async fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);

        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }

    pub async fn reset(&self) {
        let mut last = self.last_event.write().await;
        *last = Instant::now();
    }
}
