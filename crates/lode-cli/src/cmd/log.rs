#![deny(unsafe_code)]

use crate::LogCommand;
use std::fs;

pub(crate) fn log_command(command: LogCommand) -> lode_core::Result<()> {
    match command {
        LogCommand::Init => {
            let root = crate::daemon_global_root()?;
            let path = root.resolve("logs/daemon.log")?;
            if !path.exists() {
                root.write_atomic("logs/daemon.log", "")?;
            }
            println!("log initialised at {}", path.display());
        }
        LogCommand::Daemon { tail } => {
            let log = fs::read_to_string(crate::daemon_log_path()?).unwrap_or_default();
            let mut lines = log.lines().collect::<Vec<_>>();
            if let Some(tail) = tail {
                let start = lines.len().saturating_sub(tail);
                lines = lines[start..].to_vec();
            }
            for line in lines {
                println!("{line}");
            }
        }
        LogCommand::Clear => {
            let root = crate::daemon_global_root()?;
            let path = root.resolve("logs/daemon.log")?;
            if path.exists() {
                root.write_atomic("logs/daemon.log", "")?;
            }
            println!("logs cleared");
        }
    }
    Ok(())
}
