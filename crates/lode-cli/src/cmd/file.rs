#![deny(unsafe_code)]

use camino::Utf8PathBuf;
use lode_core::{
    add_managed_file, check_file_integrity, file_manifest_path, list_managed_files,
    remove_managed_file, format_file_manifest_table, LodeError, ManagedBy,
};

use crate::FileCommand;
use crate::OutputFormat;
use crate::output;

pub(crate) fn file_command(command: FileCommand) -> lode_core::Result<()> {
    match command {
        FileCommand::List { output } => file_list(output),
        FileCommand::Check { output } => file_check(output),
        FileCommand::Add {
            path,
            managed_by,
            desc,
        } => file_add(path, managed_by, desc),
        FileCommand::Remove { path } => file_remove(path),
    }
}

fn project_dir() -> lode_core::Result<Utf8PathBuf> {
    let cwd = std::env::current_dir()
        .map_err(|e| LodeError::Message(format!("cannot get current dir: {e}")))?;
    Utf8PathBuf::from_path_buf(cwd)
        .map_err(|_| LodeError::Message("non-UTF-8 path".to_string()))
}

fn file_list(output: OutputFormat) -> lode_core::Result<()> {
    let root = project_dir()?;
    let entries = list_managed_files(&root)?;

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&entries)
                .map_err(|e| LodeError::Message(e.to_string()))?
        );
    } else {
        if entries.is_empty() {
            println!("{}", output::dim("No managed files."));
            return Ok(());
        }
        println!("{}", format_file_manifest_table(&entries));
        println!(
            "\n{} {} {}",
            output::dim("Total:"),
            output::bold(&entries.len().to_string()),
            output::dim("files")
        );
    }
    Ok(())
}

fn file_check(output: OutputFormat) -> lode_core::Result<()> {
    let root = project_dir()?;
    let manifest_path = file_manifest_path(&root);

    if !manifest_path.exists() {
        println!("{}", output::dim("No file manifest found. Run `lode file add` first."));
        return Ok(());
    }

    let results = check_file_integrity(&root)?;

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&results)
                .map_err(|e| LodeError::Message(e.to_string()))?
        );
    } else {
        if results.is_empty() {
            println!("{}", output::dim("No files in manifest."));
            return Ok(());
        }

        let mut ok_count = 0;
        let mut modified_count = 0;
        let mut missing_count = 0;
        let mut untracked_count = 0;

        for result in &results {
            match result.status.as_str() {
                "ok" => {
                    ok_count += 1;
                    println!("  {}  {}", output::green("✔"), result.path);
                }
                "modified" => {
                    modified_count += 1;
                    println!("  {}  {}  {} changed", output::yellow("⚠"), result.path, output::red("HASH"));
                }
                "missing" => {
                    missing_count += 1;
                    println!("  {}  {}  {}", output::red("✘"), result.path, output::red("MISSING"));
                }
                "not_tracked" => {
                    untracked_count += 1;
                    println!("  {}  {}  {}", output::cyan("ℹ"), result.path, output::dim("(hash not tracked)"));
                }
                _ => {
                    println!("  {}  {}  {}", output::red("?"), result.path, result.status);
                }
            }
        }

        println!();
        if ok_count > 0 {
            println!("  {} {}", output::green("✔"), format!("{} ok", ok_count));
        }
        if modified_count > 0 {
            println!("  {} {}", output::yellow("⚠"), format!("{} modified", modified_count));
        }
        if missing_count > 0 {
            println!("  {} {}", output::red("✘"), format!("{} missing", missing_count));
        }
        if untracked_count > 0 {
            println!("  {} {}", output::cyan("ℹ"), format!("{} not tracked", untracked_count));
        }
    }
    Ok(())
}

fn file_add(
    path: Utf8PathBuf,
    managed_by: Option<String>,
    desc: Option<String>,
) -> lode_core::Result<()> {
    let root = project_dir()?;
    let full_path = if path.is_absolute() {
        path.clone()
    } else {
        root.join(&path)
    };

    if !full_path.exists() {
        return Err(LodeError::Message(format!(
            "file not found: {}",
            full_path
        )));
    }

    let subsystem = match managed_by.as_deref().unwrap_or("cli") {
        "scaffold" => ManagedBy::Scaffold,
        "adopt" => ManagedBy::Adopt,
        "sync" => ManagedBy::Sync,
        "agent" => ManagedBy::Agent,
        "init" => ManagedBy::Init,
        "context" => ManagedBy::Context,
        "handoff" => ManagedBy::Handoff,
        "verify" => ManagedBy::Verify,
        "depgraph" => ManagedBy::DepGraph,
        _ => {
            return Err(LodeError::Message(format!(
                "unknown subsystem: {}. Valid: scaffold, adopt, sync, agent, init, context, handoff, verify, depgraph",
                managed_by.as_deref().unwrap_or("")
            )));
        }
    };

    let description = desc.unwrap_or_default();
    let entry = add_managed_file(&root, &full_path, subsystem, &description)?;

    println!(
        "{} {} {} {}",
        output::green("✔"),
        output::bold("added"),
        entry.path,
        output::dim(&format!("({})", entry.managed_by.iter().map(|m| m.to_string()).collect::<Vec<_>>().join(", ")))
    );
    Ok(())
}

fn file_remove(path: Utf8PathBuf) -> lode_core::Result<()> {
    let root = project_dir()?;
    let full_path = if path.is_absolute() {
        path.clone()
    } else {
        root.join(&path)
    };

    let removed = remove_managed_file(&root, &full_path)?;
    if removed {
        let display = if full_path.starts_with(&root) {
            full_path
                .strip_prefix(&root)
                .unwrap_or(&full_path)
                .to_path_buf()
        } else {
            full_path.clone()
        };
        println!("{} {} {}", output::green("✔"), output::bold("removed"), display);
    } else {
        println!("{} {} not found in manifest", output::yellow("⚠"), path);
    }
    Ok(())
}
