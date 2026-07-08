use crate::PluginCommand;
pub(crate) fn plugin_command(command: PluginCommand) -> lode_core::Result<()> {
    crate::plugin_impl(command)
}
