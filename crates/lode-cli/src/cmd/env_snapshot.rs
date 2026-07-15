#![deny(unsafe_code)]

use lode_core::env_snapshot;

use crate::EnvSnapshotCommand;
use crate::OutputFormat;

pub(crate) fn env_snapshot_command(command: EnvSnapshotCommand) -> lode_core::Result<()> {
    match command {
        EnvSnapshotCommand::Create { label } => {
            let snap = env_snapshot::create_snapshot(&label)?;
            println!("  snapshot created: {} ({})", snap.id, snap.label);
            Ok(())
        }
        EnvSnapshotCommand::List { output } => {
            let list = env_snapshot::list_snapshots()?;
            let table = list
                .iter()
                .map(|s| format!("  {}  {}", s.id, s.label))
                .collect::<Vec<_>>()
                .join("\n");
            crate::print_output("env-snapshot list", list, output, || table.clone());
            Ok(())
        }
        EnvSnapshotCommand::Compare { id1, id2 } => {
            let diff = env_snapshot::compare_snapshots(&id1, &id2)?;
            println!("  added:   {}", diff.added.len());
            println!("  removed: {}", diff.removed.len());
            println!("  changed: {}", diff.changed.len());
            println!("  same:    {}", diff.same);
            Ok(())
        }
        EnvSnapshotCommand::Restore { id } => {
            env_snapshot::restore_snapshot(&id)?;
            println!("  snapshot {id} restored");
            Ok(())
        }
    }
}
