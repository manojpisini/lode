/// A minimal dlltool replacement that copies existing Rust GNU import libraries.
/// It fails loudly when no real import library exists; creating an empty archive
/// hides the root cause and fails later at link time.
use std::env;
use std::fs;
use std::path::Path;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut out_file = String::new();
    let mut dll_name = String::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-l" => {
                i += 1;
                out_file = args.get(i).cloned().unwrap_or_default();
            }
            "-D" => {
                i += 1;
                dll_name = args.get(i).cloned().unwrap_or_default();
            }
            _ => {}
        }
        i += 1;
    }

    if out_file.is_empty() {
        eprintln!("stub-dlltool: no output file specified");
        exit(1);
    }

    let parent = Path::new(&out_file).parent().unwrap_or(Path::new("."));
    if let Err(error) = fs::create_dir_all(parent) {
        eprintln!("stub-dlltool: FATAL: could not create {}: {error}", parent.display());
        exit(1);
    }

    let rustup_home = env::var("RUSTUP_HOME")
        .or_else(|_| {
            let home = env::var("USERPROFILE")
                .or_else(|_| env::var("HOME"))
                .unwrap_or_default();
            Ok::<String, env::VarError>(format!("{home}\\.rustup"))
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

    if let Err(error) = fs::copy(&toolchain_lib, &out_file) {
        eprintln!(
            "stub-dlltool: FATAL: no usable import library for {lib_name}; expected {} ({error}). Install MSYS2/LLVM dlltool or add the real import library.",
            toolchain_lib.display()
        );
        exit(1);
    }

    eprintln!("stub-dlltool: copied {out_file} from rust toolchain");
}