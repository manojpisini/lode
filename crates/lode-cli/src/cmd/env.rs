use crate::EnvCommand;
pub(crate) fn env_command(command: EnvCommand) -> lode_core::Result<()> {
    crate::env_impl(command)
}
