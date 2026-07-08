#![allow(dead_code)]

pub(crate) fn success(msg: &str) {
    println!("{msg}");
}

pub(crate) fn error(msg: &str) {
    eprintln!("error: {msg}");
}

pub(crate) fn warn(msg: &str) {
    eprintln!("warning: {msg}");
}

pub(crate) fn info(msg: &str) {
    println!("{msg}");
}

pub(crate) fn tree_line(name: &str, value: &str) {
    println!("{name}\t{value}");
}

pub(crate) fn spinner(_msg: &str) {}

pub(crate) fn status_bool(b: bool) -> &'static str {
    if b {
        "yes"
    } else {
        "no"
    }
}

pub(crate) fn format_seconds(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}
