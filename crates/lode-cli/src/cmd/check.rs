#![deny(unsafe_code)]

use lode_core::{load_global_config, fix_path, check_path, LodeError};
use crate::CheckArgs;

pub(crate) fn convention_check(args: CheckArgs) -> lode_core::Result<()> {
    let config = load_global_config()?;
    let path = args.path.unwrap_or(crate::current_dir()?);
    let report = if args.fix {
        fix_path(&path, &config)?
    } else {
        check_path(&path, &config)?
    };

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|error| LodeError::Message(error.to_string()))?
        );
    } else if report.violations.is_empty() {
        println!("convention ok: checked {}", report.checked);
        for (from, to) in &report.renamed {
            println!("renamed {from} -> {to}");
        }
    } else {
        for violation in &report.violations {
            println!("{} -> {}", violation.path, violation.expected_name);
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
    use crate::CheckArgs;

    #[test]
    fn test_check_args_defaults() {
        let args = CheckArgs {
            path: None,
            json: false,
            fix: false,
        };
        assert!(!args.json);
        assert!(!args.fix);
        assert!(args.path.is_none());
    }

    #[test]
    fn test_check_args_with_flags() {
        let args = CheckArgs {
            path: None,
            json: true,
            fix: true,
        };
        assert!(args.json);
        assert!(args.fix);
    }

    #[test]
    fn test_convention_check_fn_exists() {
        let _fn: fn(CheckArgs) -> lode_core::Result<()> = convention_check;
    }
}
