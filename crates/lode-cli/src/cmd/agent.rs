use crate::AgentCommand;
pub(crate) fn agent_command(command: AgentCommand) -> lode_core::Result<()> {
    crate::agent_impl(command)
}
