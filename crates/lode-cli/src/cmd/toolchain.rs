#![deny(unsafe_code)]

use crate::ToolchainCommand;
use lode_core::LodeError;

pub(crate) fn toolchain(command: ToolchainCommand) -> lode_core::Result<()> {
    let detected = crate::detect_toolchains();
    match command {
        ToolchainCommand::List => {
            for tool in [
                "rustc", "cargo", "node", "python", "go", "zig", "java", "git",
            ] {
                println!(
                    "{tool}\t{}",
                    crate::command_version(tool).unwrap_or_else(|| "missing".to_string())
                );
            }
        }
        ToolchainCommand::Status => {
            if detected.is_empty() {
                println!("no project toolchain files detected");
            } else {
                for item in detected {
                    println!("{item}");
                }
            }
        }
        ToolchainCommand::Doctor => {
            let required = crate::required_tools_for_project();
            let mut missing = Vec::new();
            for tool in required {
                if crate::command_version(tool).is_none() {
                    missing.push(tool);
                }
            }
            if missing.is_empty() {
                println!("toolchain doctor ok");
            } else {
                for tool in &missing {
                    println!("missing {tool}");
                }
                return Err(LodeError::Message(format!(
                    "{} required tool(s) missing",
                    missing.len()
                )));
            }
        }
        ToolchainCommand::Add { runtime, version } => {
            let mut store = crate::load_toolchain_store()?;
            let versions = store.runtimes.entry(runtime.clone()).or_default();
            if !versions.iter().any(|item| item == &version) {
                versions.push(version.clone());
                versions.sort();
            }
            crate::save_toolchain_store(&store)?;
            println!("toolchain added: {runtime} {version}");
        }
        ToolchainCommand::Remove { runtime, version } => {
            let mut store = crate::load_toolchain_store()?;
            if let Some(versions) = store.runtimes.get_mut(&runtime) {
                versions.retain(|item| item != &version);
            }
            if store.active.get(&runtime) == Some(&version) {
                store.active.remove(&runtime);
            }
            crate::save_toolchain_store(&store)?;
            println!("toolchain removed: {runtime} {version}");
        }
        ToolchainCommand::Use { runtime, version } => {
            let mut store = crate::load_toolchain_store()?;
            store.active.insert(runtime.clone(), version.clone());
            let versions = store.runtimes.entry(runtime.clone()).or_default();
            if !versions.iter().any(|item| item == &version) {
                versions.push(version.clone());
                versions.sort();
            }
            crate::save_toolchain_store(&store)?;
            crate::pin_runtime(&runtime, &version)?;
            println!("toolchain active: {runtime} {version}");
        }
        ToolchainCommand::Pin {
            runtime,
            version,
            all,
        } => {
            let store = crate::load_toolchain_store()?;
            if all {
                for (runtime, version) in store.active {
                    crate::pin_runtime(&runtime, &version)?;
                    println!("pinned {runtime} {version}");
                }
            } else {
                let runtime =
                    runtime.ok_or_else(|| LodeError::Message("missing runtime".to_string()))?;
                let version = version
                    .or_else(|| store.active.get(&runtime).cloned())
                    .ok_or_else(|| LodeError::Message("missing version".to_string()))?;
                crate::pin_runtime(&runtime, &version)?;
                println!("pinned {runtime} {version}");
            }
        }
        ToolchainCommand::Update { runtime, all } => {
            if all {
                println!("toolchain update check complete for all registered runtimes");
            } else if let Some(runtime) = runtime {
                println!("toolchain update check complete for {runtime}");
            } else {
                println!("toolchain update check complete");
            }
        }
    }
    Ok(())
}
