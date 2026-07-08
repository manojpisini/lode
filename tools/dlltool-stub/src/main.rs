/// A minimal dlltool replacement that:
/// 1. Copies existing import libraries from the rust toolchain when possible
/// 2. Creates minimal valid import libraries otherwise
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut out_file = String::new();
    let mut dll_name = String::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-l" => { i += 1; out_file = args.get(i).cloned().unwrap_or_default(); }
            "-D" => { i += 1; dll_name = args.get(i).cloned().unwrap_or_default(); }
            _ => {}
        }
        i += 1;
    }

    if out_file.is_empty() {
        eprintln!("stub-dlltool: no output file specified");
        exit(1);
    }

    let parent = Path::new(&out_file).parent().unwrap_or(Path::new("."));
    fs::create_dir_all(parent).ok();

    // Try to find existing import library in rust toolchain
    let rustup_home = env::var("RUSTUP_HOME")
        .or_else(|_| {
            let home = env::var("USERPROFILE")
                .or_else(|_| env::var("HOME"))
                .unwrap_or_default();
            Ok::<String, _>(format!("{home}\\.rustup"))
        })
        .unwrap_or_default();

    let lib_name = dll_name.trim_end_matches(".dll").to_lowercase();
    let toolchain_lib = Path::new(&rustup_home)
        .join("toolchains")
        .join("stable-x86_64-pc-windows-gnu")
        .join("lib")
        .join("rustlib")
        .join("x86_64-pc-windows-gnu")
        .join("lib")
        .join(format!("lib{lib_name}.a"));

    if toolchain_lib.exists() {
        fs::copy(&toolchain_lib, &out_file).ok();
        eprintln!("stub-dlltool: copied {out_file} from rust toolchain");
    } else {
        // Create minimal valid ar archive
        create_empty_archive(&out_file).ok();
        eprintln!("stub-dlltool: created empty archive {out_file} (no toolchain lib for {lib_name})");
    }

    // Verify the file was created
    if !Path::new(&out_file).exists() {
        eprintln!("stub-dlltool: FATAL: could not create {out_file}");
        exit(1);
    }
}

fn create_empty_archive(path: &str) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(b"!<arch>\n")?;
    Ok(())
}
