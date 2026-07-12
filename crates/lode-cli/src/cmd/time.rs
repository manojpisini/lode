#![deny(unsafe_code)]

use crate::TimeCommand;
use lode_core::{ValidatedRoot, LodeError};

pub(crate) fn time_command(command: TimeCommand) -> lode_core::Result<()> {
    match command {
        TimeCommand::Today { format } => {
            let log = crate::load_time_log()?;
            let today = crate::today_utc();
            let sessions = log
                .sessions
                .into_iter()
                .filter(|session| session.started_at.starts_with(&today))
                .collect::<Vec<_>>();
            crate::print_time_sessions("today", &sessions, &format)?;
        }
        TimeCommand::Show { since, by, format } => {
            let log = crate::load_time_log()?;
            let sessions = crate::filter_time_sessions(log.sessions, since.as_deref());
            crate::print_time_summary(&sessions, &by, &format)?;
        }
        TimeCommand::Report { since, format, out } => {
            let log = crate::load_time_log()?;
            let sessions = crate::filter_time_sessions(log.sessions, since.as_deref());
            let report = crate::render_time_report(&sessions, &format)?;
            if let Some(path) = out {
                crate::write_validated_output(&path, report)?;
                println!("wrote time report to {path}");
            } else {
                print!("{report}");
            }
        }
        TimeCommand::Clear { before, confirm } => {
            if !confirm {
                return Err(LodeError::Message(
                    "refusing to clear time log without --confirm".to_string(),
                ));
            }
            let path = crate::time_log_path()?;
            if let Some(before) = before {
                let mut log = crate::load_time_log()?;
                let before_key = crate::resolve_since_key(&before).unwrap_or(before);
                let before_key = before_key.as_str();
                let before_count = log.sessions.len();
                log.sessions
                    .retain(|session| session.started_at.as_str() >= before_key);
                crate::save_time_log(&log)?;
                println!(
                    "time log cleared: removed {} session(s)",
                    before_count - log.sessions.len()
                );
            } else if path.exists() {
                ValidatedRoot::new(crate::current_dir()?)?.remove_file(".lode/time-log.json")?;
                println!("time log cleared");
            } else {
                println!("time log cleared");
            }
        }
    }
    Ok(())
}
