use crate::ProjectsCommand;
pub(crate) fn projects(command: ProjectsCommand) -> lode_core::Result<()> {
    crate::projects_impl(command)
}
