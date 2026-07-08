use crate::InitArgs;
pub(crate) fn init(args: InitArgs) -> lode_core::Result<()> {
    crate::init_impl(args)
}
