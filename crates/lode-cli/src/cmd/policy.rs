#![deny(unsafe_code)]

use camino::Utf8PathBuf;
use lode_core::policies;

use crate::OutputFormat;
use crate::PolicyCommand;

pub(crate) fn policy_command(command: PolicyCommand) -> lode_core::Result<()> {
    match command {
        PolicyCommand::Check { output } => policy_check(output),
        PolicyCommand::List { output } => policy_list(output),
        PolicyCommand::Explain { id, output } => policy_explain(&id, output),
        PolicyCommand::Waive {
            policy_id,
            reason,
            expires,
            owner,
        } => policy_waive(&policy_id, &reason, expires, owner),
    }
}

fn lode_dir() -> Utf8PathBuf {
    Utf8PathBuf::from(".lode")
}

fn policy_check(output: OutputFormat) -> lode_core::Result<()> {
    let project_dir = lode_dir();
    let policies_list = policies::load_policies()?;
    let waivers = policies::load_waivers(&project_dir)?;
    let report = policies::check_policies(&policies_list, &waivers);

    let table = report
        .results
        .iter()
        .map(|r| {
            let status = if r.passed {
                "PASS"
            } else if r.waived {
                "WAIVED"
            } else {
                "FAIL"
            };
            format!("  {status:7} {}: {}", r.policy_id, r.message)
        })
        .collect::<Vec<_>>()
        .join("\n");

    crate::print_output("policy check", report, output, || table.clone());
    Ok(())
}

fn policy_list(output: OutputFormat) -> lode_core::Result<()> {
    let list = policies::load_policies()?;
    let table = list
        .iter()
        .map(|p| format!("  {:26} [{:8}] {}", p.id, p.severity, p.check.kind))
        .collect::<Vec<_>>()
        .join("\n");
    crate::print_output("policy list", list, output, || table.clone());
    Ok(())
}

fn policy_explain(id: &str, output: OutputFormat) -> lode_core::Result<()> {
    let list = policies::load_policies()?;
    let policy = list
        .iter()
        .find(|p| p.id == id)
        .ok_or_else(|| lode_core::LodeError::Message(format!("policy not found: {id}")))?;
    let table = format!(
        "  id:          {}\n  severity:    {}\n  scope:       {}\n  check kind:  {}\n  remediation: {}",
        policy.id,
        policy.severity,
        policy.scope.join(", "),
        policy.check.kind,
        policy.remediation.as_ref().map(|r| r.recipe.as_str()).unwrap_or("none"),
    );
    crate::print_output("policy explain", policy, output, || table.clone());
    Ok(())
}

fn policy_waive(
    policy_id: &str,
    reason: &str,
    expires: Option<String>,
    owner: Option<String>,
) -> lode_core::Result<()> {
    let project_dir = lode_dir();
    let mut waivers = policies::load_waivers(&project_dir)?;
    waivers.push(policies::PolicyWaiver {
        policy_id: policy_id.to_string(),
        reason: reason.to_string(),
        expires,
        owner,
    });
    policies::save_waivers(&project_dir, &waivers)?;
    println!("  waived policy {policy_id}");
    Ok(())
}
