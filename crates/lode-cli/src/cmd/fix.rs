#![deny(unsafe_code)]

use crate::CheckArgs;
use crate::OutputFormat;
use camino::Utf8PathBuf;

pub fn convention_fix(path: Option<Utf8PathBuf>) -> lode_core::Result<()> {
    crate::cmd::check::convention_check_with_output(CheckArgs {
        path,
        output: OutputFormat::Table,
        fix: true,
    })
}
