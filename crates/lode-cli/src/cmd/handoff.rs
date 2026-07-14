#![deny(unsafe_code)]

use lode_core::{Handoff, HandoffFormat, LodeError};

use crate::HandoffCommand;
use crate::OutputFormat;

pub(crate) fn handoff_command(command: HandoffCommand) -> lode_core::Result<()> {
    match command {
        HandoffCommand::Create { task, format, next, plan_id } => {
            handoff_create(&task, &format, &next, plan_id.as_deref())
        }
        HandoffCommand::Show { handoff_id, output } => handoff_show(&handoff_id, output),
        HandoffCommand::Verify { handoff_id } => handoff_verify(&handoff_id),
        HandoffCommand::Resume { handoff_id } => handoff_resume(&handoff_id),
        HandoffCommand::List { output } => handoff_list(output),
    }
}

fn project_dir() -> lode_core::Result<camino::Utf8PathBuf> {
    let cwd = std::env::current_dir().map_err(|e| {
        LodeError::Message(format!("cannot get current dir: {e}"))
    })?;
    camino::Utf8PathBuf::from_path_buf(cwd).map_err(|_| {
        LodeError::Message("non-UTF-8 path".to_string())
    })
}

fn handoff_create(task: &str, format: &str, next: &str, plan_id: Option<&str>) -> lode_core::Result<()> {
    let dir = project_dir()?;
    let mut handoff = Handoff::new(task);
    handoff.next_action = next.to_string();
    handoff.plan_id = plan_id.map(|s| s.to_string());

    handoff.format = match format {
        "markdown" => HandoffFormat::Markdown,
        "json" => HandoffFormat::Json,
        _ => HandoffFormat::Pidgin,
    };

    let path = handoff.save(&dir)?;
    println!("Handoff created: {}", handoff.handoff_id);
    println!("Path: {path}");

    match &handoff.format {
        HandoffFormat::Pidgin => {
            println!("\n---\n{}", handoff.render_pidgin());
        }
        HandoffFormat::Markdown => {
            println!("\n---\n{}", handoff.render_markdown());
        }
        HandoffFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&handoff)
                    .map_err(|e| LodeError::Message(e.to_string()))?
            );
        }
    }
    Ok(())
}

fn handoff_show(handoff_id: &str, output: OutputFormat) -> lode_core::Result<()> {
    let dir = project_dir()?;
    let handoff = Handoff::load(&dir, handoff_id)?;

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&handoff)
                .map_err(|e| LodeError::Message(e.to_string()))?
        );
    } else {
        println!("Handoff: {}", handoff.handoff_id);
        println!("Task: {}", handoff.task);
        println!("Status: {}", handoff.status);
        println!("Created: {}", handoff.created_at);
        if !handoff.decisions.is_empty() {
            println!("\nDecisions ({})", handoff.decisions.len());
            for d in &handoff.decisions {
                println!("  {}: {}", d.id, d.description);
            }
        }
        if !handoff.changed_paths.is_empty() {
            println!("\nChanged paths:");
            for p in &handoff.changed_paths {
                println!("  - {p}");
            }
        }
        if !handoff.verification_performed.is_empty() {
            println!("\nVerification performed:");
            for v in &handoff.verification_performed {
                println!("  - {v}");
            }
        }
        if !handoff.remaining_risks.is_empty() {
            println!("\nRemaining risks:");
            for r in &handoff.remaining_risks {
                println!("  - {r}");
            }
        }
        println!("\nNext action: {}", handoff.next_action);
        println!("\n--- Pidgin ---\n{}", handoff.render_pidgin());
    }
    Ok(())
}

fn handoff_verify(handoff_id: &str) -> lode_core::Result<()> {
    let dir = project_dir()?;
    let handoff = Handoff::load(&dir, handoff_id)?;

    if handoff.decisions.is_empty() {
        println!("Warning: no decisions recorded in handoff");
    }
    if handoff.changed_paths.is_empty() {
        println!("Warning: no changed paths recorded");
    }
    if handoff.verification_performed.is_empty() {
        println!("Warning: no verification performed");
    }
    if handoff.next_action.is_empty() {
        println!("Warning: no next action specified");
    }
    if handoff.status != "completed" {
        println!("Status: {} (not completed)", handoff.status);
    }

    if handoff.decisions.is_empty() || handoff.next_action.is_empty() {
        println!("\nHandoff needs attention before handover");
    } else {
        println!("Handoff {} verified OK", handoff_id);
    }
    Ok(())
}

fn handoff_resume(handoff_id: &str) -> lode_core::Result<()> {
    let dir = project_dir()?;
    let handoff = Handoff::load(&dir, handoff_id)?;

    println!("Resuming handoff: {}", handoff.handoff_id);
    println!("Task: {}", handoff.task);
    println!("Status: {}", handoff.status);
    println!("\nContext:");
    for c in &handoff.context_ids {
        println!("  - {c}");
    }
    if let Some(ref pid) = handoff.plan_id {
        println!("Plan: {pid}");
    }
    println!("\nNext action: {}", handoff.next_action);
    Ok(())
}

fn handoff_list(output: OutputFormat) -> lode_core::Result<()> {
    let dir = project_dir()?;
    let handoffs = Handoff::list(&dir)?;

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&handoffs)
                .map_err(|e| LodeError::Message(e.to_string()))?
        );
    } else {
        if handoffs.is_empty() {
            println!("No handoffs found");
            return Ok(());
        }
        println!("Handoffs ({}):", handoffs.len());
        for id in &handoffs {
            if let Ok(h) = Handoff::load(&dir, id) {
                println!("  {}  {}  {}", id, h.status, h.task);
            } else {
                println!("  {id}");
            }
        }
    }
    Ok(())
}
