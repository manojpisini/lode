#![deny(unsafe_code)]

use camino::Utf8PathBuf;
use crate::CheckArgs;

pub fn convention_fix(path: Option<Utf8PathBuf>) -> lode_core::Result<()> {
    crate::cmd::check::convention_check(CheckArgs {
        path,
        json: false,
        fix: true,
    })
}
