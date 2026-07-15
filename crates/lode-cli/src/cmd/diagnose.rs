#![deny(unsafe_code)]

use std::fs;

use lode_core::diagnose;

use crate::DiagnoseCommand;
use crate::OutputFormat;

pub(crate) fn diagnose_command(command: DiagnoseCommand) -> lode_core::Result<()> {
    match command {
        DiagnoseCommand::Run { input, output } => diagnose_run(input, output),
        DiagnoseCommand::Patterns { output } => diagnose_patterns(output),
    }
}

fn diagnose_run(input: Option<String>, output: OutputFormat) -> lode_core::Result<()> {
    let content = match input.as_deref() {
        Some("-") | None => {
            let mut buf = String::new();
            std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)
                .map_err(|e| lode_core::LodeError::Message(e.to_string()))?;
            buf
        }
        Some(path) => {
            fs::read_to_string(path).map_err(|e| lode_core::LodeError::Message(e.to_string()))?
        }
    };

    let results = diagnose::diagnose_output(&content);
    let table = results
        .iter()
        .map(|d| format!("  [{:8}] {}: {}", d.severity, d.label, d.suggestion))
        .collect::<Vec<_>>()
        .join("\n");
    crate::print_output("diagnose", results, output, || table.clone());
    Ok(())
}

fn diagnose_patterns(output: OutputFormat) -> lode_core::Result<()> {
    let patterns = diagnose::list_diagnosis_patterns();
    let table = patterns
        .iter()
        .map(|p| format!("  {:28} {:8} {}", p.id, p.severity, p.label))
        .collect::<Vec<_>>()
        .join("\n");
    crate::print_output("diagnose patterns", patterns, output, || table.clone());
    Ok(())
}
