use crate::LogCommand;
pub(crate) fn log_command(command: LogCommand) -> lode_core::Result<()> {
    crate::log_impl(command)
}
