#![deny(unsafe_code)]

use lode_core::sandbox::{run_in_sandbox, SandboxConfig};

use crate::OutputFormat;
use crate::SandboxCommand;

pub(crate) fn sandbox_command(command: SandboxCommand) -> lode_core::Result<()> {
    match command {
        SandboxCommand::Run {
            command,
            args,
            timeout,
            inherit_env,
            output,
        } => sandbox_run(&command, &args, timeout, inherit_env, output),
    }
}

fn sandbox_run(
    command: &str,
    args: &[String],
    timeout: u64,
    inherit_env: bool,
    output: OutputFormat,
) -> lode_core::Result<()> {
    let config = SandboxConfig {
        timeout_secs: timeout,
        inherit_env,
        ..SandboxConfig::default()
    };
    let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let result = run_in_sandbox(&config, command, &str_args, &[])?;
    let table = format!(
        "  exit code: {}\n  stdout:    {} bytes\n  stderr:    {} bytes\n  duration:  {}ms\n  files:     {}",
        result.exit_code,
        result.stdout.len(),
        result.stderr.len(),
        result.duration_ms,
        result.files_written.len(),
    );
    crate::print_output("sandbox run", result, output, || table.clone());
    Ok(())
}
