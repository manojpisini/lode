#![deny(unsafe_code)]

use camino::Utf8PathBuf;
use lode_core::LodeError;

pub fn completions(
    shell: &str,
    install: bool,
    dry_run: bool,
    out: Option<Utf8PathBuf>,
) -> lode_core::Result<()> {
    let script = crate::completion_script(shell)?;
    let output_path = if install {
        Some(out.unwrap_or(crate::default_completion_path(shell)?))
    } else {
        out
    };
    if let Some(path) = output_path {
        let hint = crate::completion_install_hint(shell, &path)?;
        let source = crate::completion_source_line(shell, &path)?;
        if dry_run {
            println!("would write {shell} completions to {path}");
            println!("would record completion install receipt");
            println!("{hint}");
            println!("{source}");
            return Ok(());
        }
        crate::write_validated_output(&path, script)?;
        if install {
            crate::write_completion_install_receipt(shell, &path, &source, &hint)?;
        }
        println!("wrote {shell} completions to {path}");
        if install {
            println!("{hint}");
            println!("{source}");
        }
    } else {
        if dry_run {
            return Err(LodeError::Message(
                "--dry-run is only meaningful with --install or --out".to_string(),
            ));
        }
        print!("{script}");
    }
    Ok(())
}
