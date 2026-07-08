use crate::ConfigCommand;
pub(crate) fn config_command(command: ConfigCommand) -> lode_core::Result<()> {
    crate::config_impl(command)
}
