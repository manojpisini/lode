use crate::RulesCommand;
pub(crate) fn rules(command: RulesCommand) -> lode_core::Result<()> {
    crate::rules_impl(command)
}
