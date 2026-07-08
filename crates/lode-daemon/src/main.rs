use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use camino::Utf8PathBuf;
use clap::Parser;
use tokio::sync::mpsc;

use lode_daemon::{
    handle_create, handle_delete, handle_modify, handle_rename, load_state, save_state,
    DaemonState, DaemonWatcher, IdleWatchdog, IpcServer, WatcherConfig,
};

#[derive(Parser)]
#[command(name = "lode-daemon")]
#[command(about = "LODE daemon - watches project files and responds to events")]
struct Args {
    #[arg(long)]
    foreground: bool,

    #[arg(long)]
    no_rename: bool,

    #[arg(long)]
    no_sign: bool,

    #[arg(long)]
    no_stamp: bool,

    #[arg(long, default_value = ".lode/daemon")]
    state_dir: PathBuf,

    #[arg(long, default_value = "300")]
    idle_timeout: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let project_dir = std::env::current_dir()
        .map(|p| Utf8PathBuf::try_from(p).expect("Invalid project directory"))?;

    let config = WatcherConfig {
        debounce_ms: 200,
        no_rename: args.no_rename,
        no_sign: args.no_sign,
        no_stamp: args.no_stamp,
    };

    let state_path = args.state_dir.join("state.json");
    let mut state = load_state(&state_path).unwrap_or_else(|_| DaemonState::new());

    let mut watcher = DaemonWatcher::start(project_dir.clone(), config.clone())?;
    watcher.watch()?;

    let ipc_socket = args.state_dir.join("daemon.sock");
    let mut ipc_server = IpcServer::new(ipc_socket);
    ipc_server.start().await?;

    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
    let mut idle_watchdog = IdleWatchdog::new(args.idle_timeout, shutdown_tx.clone());
    idle_watchdog.start().await?;

    state.active = true;
    state.add_watcher(project_dir.to_string());
    save_state(&state_path, &state)?;

    eprintln!("Lode daemon started for {project_dir}");
    eprintln!("State saved to {}", state_path.display());

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        eprintln!("Received SIGINT, shutting down...");
        r.store(false, Ordering::SeqCst);
        let _ = shutdown_tx.send(()).await;
    });

    #[cfg(unix)]
    {
        let r = running.clone();
        tokio::spawn(async move {
            let mut sigterm =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                    .expect("Failed to register SIGTERM handler");
            sigterm.recv().await;
            eprintln!("Received SIGTERM, shutting down...");
            r.store(false, Ordering::SeqCst);
            let _ = shutdown_tx.send(()).await;
        });
    }

    while running.load(Ordering::SeqCst) {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                break;
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                if let Some(event) = watcher.receive_event() {
                    idle_watchdog.reset().await;
                    state.increment_events();

                    let result = match &event {
                        lode_daemon::WatchEvent::Create(p) => handle_create(p, &config),
                        lode_daemon::WatchEvent::Modify(p) => handle_modify(p, &config),
                        lode_daemon::WatchEvent::Rename { from, to } => handle_rename(from, to, &config),
                        lode_daemon::WatchEvent::Delete(p) => handle_delete(p, &config),
                    };

                    match result {
                        Ok(actions) => {
                            for action in actions {
                                eprintln!("  {action}");
                            }
                        }
                        Err(e) => {
                            eprintln!("Handler error: {e}");
                        }
                    }
                }
            }
        }
    }

    eprintln!("Shutting down daemon...");

    let _ = watcher.stop();
    let _ = ipc_server.stop();
    idle_watchdog.stop().await;

    state.stop();
    save_state(&state_path, &state)?;

    eprintln!("Daemon stopped.");
    Ok(())
}
