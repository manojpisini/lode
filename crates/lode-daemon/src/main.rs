#![deny(unsafe_code)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use camino::Utf8PathBuf;
use clap::Parser;
use tokio::sync::mpsc;

use lode_core::auto_register_global_assets;
use lode_daemon::{
    handle_create, handle_delete, handle_modify, handle_rename, load_state, run_ipc_listener,
    save_state, DaemonControl, DaemonState, DaemonWatcher, IdleWatchdog, IpcServer, WatcherConfig,
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

    #[arg(long)]
    no_auto_register: bool,

    #[arg(long, default_value = ".lode/daemon")]
    state_dir: PathBuf,

    #[arg(long, default_value = "300")]
    idle_timeout: u64,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let project_dir = std::env::current_dir()
        .ok()
        .and_then(|p| Utf8PathBuf::try_from(p).ok())
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "non-UTF-8 project path")
        })?;

    let config = WatcherConfig {
        debounce_ms: 200,
        no_rename: args.no_rename,
        no_sign: args.no_sign,
        no_stamp: args.no_stamp,
    };

    let state_path = args.state_dir.join("state.json");
    let mut state = load_state(&state_path).unwrap_or_else(|e| {
        eprintln!(
            "lode-daemon: warning: failed to load state from {}: {}. Using fresh state.",
            state_path.display(),
            e
        );
        DaemonState::new()
    });

    let mut watcher = DaemonWatcher::start(project_dir.clone(), config.clone())?;
    watcher.watch()?;
    watcher.watch_global();

    let ipc_socket = args.state_dir.join("daemon.sock");
    let mut ipc_server = IpcServer::new(ipc_socket.clone());
    ipc_server.start().await?;
    let auth_token = ipc_server.auth_token().to_string();

    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
    let control = DaemonControl::new(shutdown_tx.clone());

    let ctrl = DaemonControl {
        shutdown_tx: control.shutdown_tx.clone(),
        paused: std::sync::Arc::clone(&control.paused),
    };
    let ipc_task = tokio::spawn(async move {
        if let Err(error) = run_ipc_listener(ipc_socket, auth_token, ctrl).await {
            eprintln!("IPC listener stopped: {error}");
        }
    });
    let mut idle_watchdog = IdleWatchdog::new(args.idle_timeout, shutdown_tx.clone());
    idle_watchdog.start().await?;

    state.active = true;
    state.add_watcher(project_dir.to_string());
    save_state(&state_path, &state)?;

    eprintln!("Lode daemon started for {project_dir}");
    eprintln!("State saved to {}", state_path.display());

    let running = Arc::new(AtomicBool::new(true));

    {
        let r = running.clone();
        let shutdown_tx = shutdown_tx.clone();
        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            eprintln!("Received SIGINT, shutting down...");
            r.store(false, Ordering::SeqCst);
            let _ = shutdown_tx.send(()).await;
        });
    }

    #[cfg(unix)]
    {
        let r = running.clone();
        let shutdown_tx = shutdown_tx.clone();
        tokio::spawn(async move {
            let mut sigterm =
                match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Failed to register SIGTERM handler: {e}");
                        return;
                    }
                };
            sigterm.recv().await;
            eprintln!("Received SIGTERM, shutting down...");
            r.store(false, Ordering::SeqCst);
            let _ = shutdown_tx.send(()).await;
        });
    }

    #[cfg(windows)]
    {
        let r = running.clone();
        let shutdown_tx = shutdown_tx.clone();
        tokio::spawn(async move {
            let _ = tokio::task::spawn_blocking(|| {
                use std::io::Read;
                let mut buf = [0u8; 1];
                loop {
                    match std::io::stdin().read(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {}
                    }
                }
            })
            .await;
            eprintln!("Received shutdown signal, shutting down...");
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
                if control.paused.load(std::sync::atomic::Ordering::SeqCst) {
                    continue;
                }
                if let Some(event) = watcher.receive_event() {
                    idle_watchdog.reset().await;
                    state.increment_events();

                    if !args.no_auto_register && watcher.is_global_event(&event) {
                        if matches!(event, lode_daemon::WatchEvent::Create(_) | lode_daemon::WatchEvent::Modify(_)) {
                            let _ = auto_register_global_assets();
                        }
                    }

                    let result = match &event {
                        lode_daemon::WatchEvent::Create(p) => handle_create(p, &config),
                        lode_daemon::WatchEvent::Modify(p) => handle_modify(p, &config),
                        lode_daemon::WatchEvent::Rename { from, to } => handle_rename(from, to),
                        lode_daemon::WatchEvent::Delete(p) => handle_delete(p),
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
    let _ = ipc_server.stop().await;
    ipc_task.abort();
    idle_watchdog.stop().await;

    state.stop();
    save_state(&state_path, &state)?;

    // Clean up IPC socket, port, lock, and token files
    let sock = args.state_dir.join("daemon.sock");
    let port_file = sock.with_extension("port");
    let lock_file = sock.with_extension("lock");
    let token_file = sock.with_extension("token");
    let _ = std::fs::remove_file(&sock);
    let _ = std::fs::remove_file(&port_file);
    let _ = std::fs::remove_file(&lock_file);
    let _ = std::fs::remove_file(&token_file);

    eprintln!("Daemon stopped.");
    Ok(())
}
