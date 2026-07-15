#![deny(unsafe_code)]

use lode_core::archetype;

use crate::ArchetypeCommand;
use crate::OutputFormat;

pub(crate) fn archetype_command(command: ArchetypeCommand) -> lode_core::Result<()> {
    match command {
        ArchetypeCommand::List { output } => archetype_list(output),
        ArchetypeCommand::Show { id, output } => archetype_show(&id, output),
        ArchetypeCommand::Apply { id, dry_run } => archetype_apply(&id, dry_run),
    }
}

fn archetype_list(output: OutputFormat) -> lode_core::Result<()> {
    let list = archetype::list_archetypes();
    let table = list
        .iter()
        .map(|a| format!("  {:26} {}", a.id, a.summary))
        .collect::<Vec<_>>()
        .join("\n");
    crate::print_output("archetype list", list, output, || table.clone());
    Ok(())
}

fn archetype_show(id: &str, output: OutputFormat) -> lode_core::Result<()> {
    let a = archetype::resolve_archetype(id)?;
    let table = format!(
        "  id:        {}\n  summary:   {}\n  profile:   {}\n  depth:     {}\n  policies:  {}\n  recipes:   {}",
        a.id,
        a.summary,
        a.profile,
        a.depth,
        a.policies.join(", "),
        a.recipes.join(", "),
    );
    crate::print_output("archetype show", a, output, || table.clone());
    Ok(())
}

fn archetype_apply(id: &str, _dry_run: bool) -> lode_core::Result<()> {
    let a = archetype::resolve_archetype(id)?;
    println!("  archetype: {} ({})", a.id, a.summary);
    println!("  profile:   {}", a.profile);
    println!("  policies:  {}", a.policies.join(", "));
    println!("  recipes:   {}", a.recipes.join(", "));
    if !a.policies.is_empty() {
        println!("  Run `lode policy check` to enforce policies.");
    }
    if !a.recipes.is_empty() {
        println!("  Run `lode recipe apply <name>` to install recipes.");
    }
    Ok(())
}
