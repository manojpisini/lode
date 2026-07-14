#![deny(unsafe_code)]

use lode_core::{CommandReceipt, LodeError};

use crate::ReceiptCommand;
use crate::OutputFormat;

pub(crate) fn receipt_command(command: ReceiptCommand) -> lode_core::Result<()> {
    match command {
        ReceiptCommand::List { output } => receipt_list(output),
        ReceiptCommand::Show { receipt_id, output } => receipt_show(&receipt_id, output),
        ReceiptCommand::Resume { receipt_id } => receipt_resume(&receipt_id),
    }
}

fn receipts_dir() -> lode_core::Result<camino::Utf8PathBuf> {
    let cwd = std::env::current_dir().map_err(|e| {
        LodeError::Message(format!("cannot get current dir: {e}"))
    })?;
    let dir = camino::Utf8PathBuf::from_path_buf(cwd).map_err(|_| {
        LodeError::Message("non-UTF-8 path".to_string())
    })?;
    Ok(dir.join(".lode").join("state").join("receipts"))
}

fn receipt_list(output: OutputFormat) -> lode_core::Result<()> {
    let dir = receipts_dir()?;
    let receipts = CommandReceipt::list(&dir)?;

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&receipts)
                .map_err(|e| LodeError::Message(e.to_string()))?
        );
    } else {
        if receipts.is_empty() {
            println!("No receipts found");
            return Ok(());
        }
        println!("Receipts ({}):", receipts.len());
        for id in &receipts {
            let path = dir.join(format!("{id}.json"));
            if let Ok(receipt) = CommandReceipt::load(&path) {
                println!("  {}  {}  {}", id, receipt.command, receipt.started_at);
            } else {
                println!("  {id}");
            }
        }
    }
    Ok(())
}

fn receipt_show(receipt_id: &str, output: OutputFormat) -> lode_core::Result<()> {
    let dir = receipts_dir()?;
    let path = dir.join(format!("{receipt_id}.json"));
    let receipt = CommandReceipt::load(&path)?;

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&receipt)
                .map_err(|e| LodeError::Message(e.to_string()))?
        );
    } else {
        println!("Receipt: {}", receipt.receipt_id);
        println!("Command: {}", receipt.command);
        println!("Args: {}", receipt.args.join(" "));
        println!("Status: {:?}", receipt.status);
        println!("Started: {}", receipt.started_at);
        if !receipt.changed_files.is_empty() {
            println!("\nChanged files:");
            for f in &receipt.changed_files {
                println!("  - {f}");
            }
        }
        if !receipt.generated_assets.is_empty() {
            println!("\nGenerated assets:");
            for a in &receipt.generated_assets {
                println!("  - {a}");
            }
        }
        if !receipt.steps.is_empty() {
            println!("\nSteps:");
            for s in &receipt.steps {
                println!("  {}  {}  {}", s.id, s.description, s.status);
            }
        }
        if let Some(ref err) = receipt.error {
            println!("\nError: {err}");
        }
        if !receipt.result.next_actions.is_empty() {
            println!("\nNext actions:");
            for a in &receipt.result.next_actions {
                println!("  {}  {}  required={}", a.command, a.description, a.required);
            }
        }
    }
    Ok(())
}

fn receipt_resume(receipt_id: &str) -> lode_core::Result<()> {
    let dir = receipts_dir()?;
    let path = dir.join(format!("{receipt_id}.json"));
    let receipt = CommandReceipt::load(&path)?;

    if let Some(ref point) = receipt.resumption_point {
        println!("Resuming receipt {receipt_id} from: {point}");
    } else {
        println!("Receipt {receipt_id} has no resumption point");
    }
    if !receipt.result.next_actions.is_empty() {
        println!("\nSuggested next actions:");
        for a in &receipt.result.next_actions {
            println!("  {}  {}", a.command, a.description);
        }
    }
    Ok(())
}
