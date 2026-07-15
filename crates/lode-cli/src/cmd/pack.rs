#![deny(unsafe_code)]

use lode_core::organization;

use crate::OutputFormat;
use crate::PackCommand;

pub(crate) fn pack_command(command: PackCommand) -> lode_core::Result<()> {
    match command {
        PackCommand::List { output } => pack_list(output),
        PackCommand::Use { id, dry_run } => pack_use(&id, dry_run),
        PackCommand::Layer { id, dry_run } => pack_layer(&id, dry_run),
        PackCommand::Export { out } => pack_export(out),
    }
}

fn pack_list(output: OutputFormat) -> lode_core::Result<()> {
    let packs = organization::builtin_packs();
    let table = packs
        .iter()
        .map(|p| format!("  {:26} v{:8} {}", p.id, p.version, p.description))
        .collect::<Vec<_>>()
        .join("\n");
    crate::print_output("pack list", packs, output, || table.clone());
    Ok(())
}

fn pack_use(id: &str, _dry_run: bool) -> lode_core::Result<()> {
    let packs = organization::builtin_packs();
    let pack = packs
        .iter()
        .find(|p| p.id == id)
        .ok_or_else(|| lode_core::LodeError::Message(format!("pack not found: {id}")))?;
    println!("  activated pack: {} v{}", pack.name, pack.version);
    if !pack.policies.is_empty() {
        println!("  policies: {}", pack.policies.join(", "));
    }
    if !pack.recipes.is_empty() {
        println!("  recipes: {}", pack.recipes.join(", "));
    }
    if !pack.profiles.is_empty() {
        println!("  profiles: {}", pack.profiles.join(", "));
    }
    Ok(())
}

fn pack_layer(id: &str, _dry_run: bool) -> lode_core::Result<()> {
    let packs = organization::builtin_packs();
    let pack = packs
        .iter()
        .find(|p| p.id == id)
        .ok_or_else(|| lode_core::LodeError::Message(format!("pack not found: {id}")))?;
    println!("  layered pack: {} v{}", pack.name, pack.version);
    Ok(())
}

fn pack_export(_out: Option<camino::Utf8PathBuf>) -> lode_core::Result<()> {
    println!("  export not yet implemented");
    Ok(())
}
