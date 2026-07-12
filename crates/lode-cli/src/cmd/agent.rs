#![deny(unsafe_code)]

use crate::AgentCommand;

pub(crate) fn agent_command(command: AgentCommand) -> lode_core::Result<()> {
    match command {
        AgentCommand::Sync => crate::agent_sync()?,
        AgentCommand::Status => crate::agent_status()?,
        AgentCommand::Export { out } => crate::agent_export(out)?,
        AgentCommand::Plan { command } => crate::agent_plan(command)?,
    }
    Ok(())
}
