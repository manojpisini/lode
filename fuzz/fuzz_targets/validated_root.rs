#![no_main]

use libfuzzer_sys::fuzz_target;
use std::path::Path;

fuzz_target!(|data: &[u8]| {
    let tmp = match tempfile::tempdir() {
        Ok(d) => d,
        Err(_) => return,
    };
    let root = match lode_core::ValidatedRoot::new(tmp.path()) {
        Ok(r) => r,
        Err(_) => return,
    };
    let input = String::from_utf8_lossy(data);
    let _ = root.resolve(&*input);
    let _ = root.resolve(Path::new(&*input));
});
