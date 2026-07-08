use crate::RecipeCommand;
pub(crate) fn recipe_command(command: RecipeCommand) -> lode_core::Result<()> {
    crate::recipe_impl(command)
}
