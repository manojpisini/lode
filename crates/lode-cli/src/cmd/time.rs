use crate::TimeCommand;
pub(crate) fn time_command(command: TimeCommand) -> lode_core::Result<()> {
    crate::time_impl(command)
}
