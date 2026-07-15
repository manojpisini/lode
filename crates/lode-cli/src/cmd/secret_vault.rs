#![deny(unsafe_code)]

use lode_core::secret_broker;

use crate::OutputFormat;
use crate::SecretVaultCommand;

pub(crate) fn secret_vault_command(command: SecretVaultCommand) -> lode_core::Result<()> {
    match command {
        SecretVaultCommand::Set { key, value, scope } => {
            secret_broker::set_secret(&key, &value, &scope)?;
            println!("  secret '{key}' set (scope: {scope})");
            Ok(())
        }
        SecretVaultCommand::Get { key, show } => {
            match secret_broker::get_secret(&key)? {
                Some(val) => {
                    if show {
                        println!("  {key}={val}");
                    } else {
                        println!("  {key}=[REDACTED] (use --show to reveal)");
                    }
                }
                None => println!("  secret '{key}' not found"),
            }
            Ok(())
        }
        SecretVaultCommand::List { output } => {
            let list = secret_broker::list_secrets_view()?;
            let table = list
                .iter()
                .map(|e| format!("  {}  [{}]", e.key, e.scope))
                .collect::<Vec<_>>()
                .join("\n");
            crate::print_output("secret-vault list", list, output, || table.clone());
            Ok(())
        }
        SecretVaultCommand::Remove { key } => {
            if secret_broker::remove_secret(&key)? {
                println!("  secret '{key}' removed");
            } else {
                println!("  secret '{key}' not found");
            }
            Ok(())
        }
        SecretVaultCommand::Grant {
            key,
            principal,
            permission,
        } => {
            secret_broker::grant_access(&key, &principal, &permission)?;
            println!("  granted {permission} access to '{key}' for {principal}");
            Ok(())
        }
        SecretVaultCommand::Revoke { key, principal } => {
            if secret_broker::revoke_access(&key, &principal)? {
                println!("  revoked access to '{key}' for {principal}");
            } else {
                println!("  no access found for {principal} on '{key}'");
            }
            Ok(())
        }
    }
}
