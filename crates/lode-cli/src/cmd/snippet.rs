use crate::SnippetCommand;
pub(crate) fn snippet_command(command: SnippetCommand) -> lode_core::Result<()> {
    crate::snippet_impl(command)
}
