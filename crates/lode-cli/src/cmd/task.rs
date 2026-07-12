#![deny(unsafe_code)]

use crate::{list_make_targets, run_make};

pub fn task_command(target: Option<String>, no_store: bool) -> lode_core::Result<()> {
    match target.as_deref() {
        None | Some("list") => list_make_targets(),
        Some("test") => {
            if no_store {
                println!("task test running without storing history");
            }
            run_make("test")
        }
        Some(target) => run_make(target),
    }
}
