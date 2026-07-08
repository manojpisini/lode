use crate::WorkspaceCommand;
pub(crate) fn workspace(command: WorkspaceCommand) -> lode_core::Result<()> {
    crate::workspace_impl(command)
}
