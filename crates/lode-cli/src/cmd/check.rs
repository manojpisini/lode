use crate::CheckArgs;
pub(crate) fn convention_check(args: CheckArgs) -> lode_core::Result<()> {
    crate::convention_check_impl(args)
}
