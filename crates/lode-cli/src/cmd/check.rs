#![deny(unsafe_code)]

use crate::cmd::output;
use crate::CheckArgs;
use lode_core::{check_path, fix_path, load_global_config, LodeError};

pub(crate) fn convention_check_with_output(args: CheckArgs) -> lode_core::Result<()> {
    let config = load_global_config()?;
    let path = args.path.unwrap_or(crate::current_dir()?);
    let report = if args.fix {
        fix_path(&path, &config)?
    } else {
        check_path(&path, &config)?
    };

    if args.output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|error| LodeError::Message(error.to_string()))?
        );
    } else if report.violations.is_empty() {
        println!("{}", output::section("Conventions Check"));
        println!(
            "  {} {} checked: {}",
            output::ok("ok"),
            output::dim("scanned"),
            output::cyan(&report.checked.to_string())
        );
        for (from, to) in &report.renamed {
            println!(
                "  {} {} -> {}",
                output::dim("renamed"),
                output::dim(from.as_str()),
                output::green(to.as_str())
            );
        }
    } else {
        println!("{}", output::section("Convention Violations"));
        for violation in &report.violations {
            println!(
                "  {} {} -> {}",
                output::fail(""),
                violation.path,
                output::cyan(&violation.expected_name)
            );
        }
    }

    if !report.violations.is_empty() {
        return Err(LodeError::Violations {
            count: report.violations.len(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CheckArgs, OutputFormat};

    #[test]
    fn test_check_args_defaults() {
        let args = CheckArgs {
            path: None,
            output: OutputFormat::Table,
            fix: false,
        };
        assert!(!args.output.should_use_json());
        assert!(!args.fix);
        assert!(args.path.is_none());
    }

    #[test]
    fn test_check_args_with_flags() {
        let args = CheckArgs {
            path: None,
            output: OutputFormat::Json,
            fix: true,
        };
        assert!(args.output.should_use_json());
        assert!(args.fix);
    }

    #[test]
    fn test_convention_check_fn_exists() {
        let _fn: fn(CheckArgs) -> lode_core::Result<()> = convention_check_with_output;
    }
}
