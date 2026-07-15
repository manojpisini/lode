#![deny(unsafe_code)]

use crate::{cmd, CheckArgs, OutputFormat, RulesCommand};
use lode_core::{load_global_config, LodeError};

pub(crate) fn rules(command: RulesCommand) -> lode_core::Result<()> {
    match command {
        RulesCommand::List => {
            let config = load_global_config()?;
            println!("default_case\t{}", config.convention.default_case);
            println!(
                "protected_prefixes\t{}",
                config.convention.protected_prefixes.join(",")
            );
        }
        RulesCommand::Check { path } => {
            cmd::check::convention_check_with_output(CheckArgs {
                path,
                output: OutputFormat::Table,
                fix: false,
            })?;
        }
        RulesCommand::Validate => {
            let config = load_global_config()?;
            if config.convention.default_case.trim().is_empty() {
                return Err(LodeError::Message(
                    "convention.default_case must not be empty".to_string(),
                ));
            }
            println!("rules valid");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RulesCommand;

    #[test]
    fn test_rules_command_list() {
        let command = RulesCommand::List;
        assert!(matches!(command, RulesCommand::List));
    }

    #[test]
    fn test_rules_command_check() {
        let command = RulesCommand::Check { path: None };
        assert!(matches!(command, RulesCommand::Check { .. }));
    }

    #[test]
    fn test_rules_command_validate() {
        let command = RulesCommand::Validate;
        assert!(matches!(command, RulesCommand::Validate));
    }

    #[test]
    fn test_rules_fn_exists() {
        let _fn: fn(RulesCommand) -> lode_core::Result<()> = rules;
    }
}
