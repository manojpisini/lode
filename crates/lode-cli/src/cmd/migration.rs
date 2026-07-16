#![deny(unsafe_code)]

use lode_core::migration;

use crate::MigrationCommand;

pub(crate) fn migration_command(command: MigrationCommand) -> lode_core::Result<()> {
    match command {
        MigrationCommand::Plan { description, kind } => {
            let id = migration::plan_migration(&description, &kind)?;
            println!("  migration planned: {id}");
            Ok(())
        }
        MigrationCommand::Apply { id } => {
            if migration::apply_migration(&id)? {
                println!("  migration {id} applied");
            } else {
                println!("  migration {id} already applied");
            }
            Ok(())
        }
        MigrationCommand::Rollback { id } => {
            if migration::rollback_migration(&id)? {
                println!("  migration {id} rolled back");
            } else {
                println!("  migration {id} not applied");
            }
            Ok(())
        }
        MigrationCommand::List { output } => {
            let plan = migration::list_migrations()?;
            let table = plan
                .migrations
                .iter()
                .map(|m| {
                    let status = if m.applied { "APPLIED" } else { "PENDING" };
                    format!("  {status:8} {}  {}", m.id, m.description)
                })
                .collect::<Vec<_>>()
                .join("\n");
            crate::print_output("migration list", plan, output, || table.clone());
            Ok(())
        }
    }
}
