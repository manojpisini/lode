#![deny(unsafe_code)]

use lode_core::agent_sim;
use lode_core::build_catalog;

use crate::AgentSimCommand;
use crate::OutputFormat;

pub(crate) fn agent_sim_command(command: AgentSimCommand) -> lode_core::Result<()> {
    match command {
        AgentSimCommand::Simulate { intent, output } => agent_simulate(&intent, output),
    }
}

fn agent_simulate(intent: &str, output: OutputFormat) -> lode_core::Result<()> {
    let config = lode_core::load_global_config()?;
    let catalog = build_catalog(&config);
    let result = agent_sim::simulate_intent(intent, &catalog.entries)?;
    let table = result
        .resolved_actions
        .iter()
        .map(|a| format!("  {:5.0}%  {}", a.probability * 100.0, a.asset_id))
        .collect::<Vec<_>>()
        .join("\n");
    crate::print_output("agent-sim simulate", result, output, || table.clone());
    Ok(())
}
