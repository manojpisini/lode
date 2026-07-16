use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use serde::{Deserialize, Serialize};

use crate::error::{LodeError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretVault {
    pub secrets: HashMap<String, SecretEntry>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SecretEntry {
    pub key: String,
    pub value: String,
    pub scope: String,
    pub created_at: u64,
    pub grants: Vec<Grant>,
}

impl std::fmt::Debug for SecretEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretEntry")
            .field("key", &self.key)
            .field("value", &"***redacted***")
            .field("scope", &self.scope)
            .field("created_at", &self.created_at)
            .field("grants", &self.grants)
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grant {
    pub principal: String,
    pub permission: String,
    pub granted_at: u64,
    pub expires_at: Option<u64>,
}

fn vault_path() -> Result<PathBuf> {
    let dir = crate::install::global_asset_dir("state")?;
    Ok(PathBuf::from(dir.join("secret-vault.json")))
}

fn vault_key_path() -> Result<PathBuf> {
    let dir = crate::install::global_asset_dir("state")?;
    Ok(PathBuf::from(dir.join("secret-vault.key")))
}

const MAGIC: &[u8] = b"LODEVAULT\n";

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::from_secs(0))
        .as_secs()
}

fn load_or_create_key() -> Result<[u8; 32]> {
    let key_path = vault_key_path()?;
    if key_path.exists() {
        let raw = fs::read(&key_path).map_err(|source| LodeError::Io {
            path: key_path.clone(),
            source,
        })?;
        if raw.len() == 32 {
            let mut key = [0u8; 32];
            key.copy_from_slice(&raw);
            return Ok(key);
        }
    }

    let mut key = [0u8; 32];
    getrandom::getrandom(&mut key).expect("getrandom failed");

    if let Some(parent) = key_path.parent() {
        fs::create_dir_all(parent).map_err(|source| LodeError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::write(&key_path, key).map_err(|source| LodeError::Io {
        path: key_path.clone(),
        source,
    })?;
    #[cfg(unix)]
    {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&key_path, Permissions::from_mode(0o600)).ok();
    }

    Ok(key)
}

fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>> {
    let mut nonce_bytes = [0u8; 12];
    getrandom::getrandom(&mut nonce_bytes)
        .map_err(|e| LodeError::Message(format!("rng error: {e}")))?;
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| LodeError::Message(format!("key error: {e}")))?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| LodeError::Message(format!("encryption error: {e}")))?;
    let mut out = nonce_bytes.to_vec();
    out.extend(ciphertext);
    Ok(out)
}

fn decrypt(key: &[u8; 32], data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < 12 {
        return Err(LodeError::Message("corrupt vault: too short".into()));
    }
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| LodeError::Message(format!("key error: {e}")))?;
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| LodeError::Message("vault decryption failed".into()))
}

pub fn load_vault() -> Result<SecretVault> {
    let path = vault_path()?;
    if !path.exists() {
        return Ok(SecretVault {
            secrets: HashMap::new(),
        });
    }
    let data = fs::read(&path).map_err(|source| LodeError::Io {
        path: path.clone(),
        source,
    })?;

    if data.starts_with(b"{") {
        let vault: SecretVault =
            serde_json::from_slice(&data).map_err(|e| LodeError::Message(e.to_string()))?;
        save_vault(&vault)?;
        return Ok(vault);
    }

    if !data.starts_with(MAGIC) {
        return Err(LodeError::Message(
            "corrupt vault file: missing magic header".into(),
        ));
    }
    let encrypted = &data[MAGIC.len()..];
    let key = load_or_create_key()?;
    let decrypted = decrypt(&key, encrypted)?;
    serde_json::from_slice(&decrypted).map_err(|e| LodeError::Message(e.to_string()))
}

pub fn save_vault(vault: &SecretVault) -> Result<()> {
    let path = vault_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LodeError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let key = load_or_create_key()?;
    let raw = serde_json::to_vec(vault).map_err(|e| LodeError::Message(e.to_string()))?;
    let encrypted = encrypt(&key, &raw)?;
    let mut out = MAGIC.to_vec();
    out.extend(encrypted);
    fs::write(&path, &out).map_err(|source| LodeError::Io {
        path: path.clone(),
        source,
    })?;
    Ok(())
}

pub fn set_secret(key: &str, value: &str, scope: &str) -> Result<()> {
    let mut vault = load_vault()?;
    vault.secrets.insert(
        key.to_string(),
        SecretEntry {
            key: key.to_string(),
            value: value.to_string(),
            scope: scope.to_string(),
            created_at: now_secs(),
            grants: Vec::new(),
        },
    );
    save_vault(&vault)?;
    Ok(())
}

pub fn get_secret(key: &str) -> Result<Option<String>> {
    let vault = load_vault()?;
    Ok(vault.secrets.get(key).map(|e| e.value.clone()))
}

pub fn get_secret_for(key: &str, principal: &str) -> Result<Option<String>> {
    let vault = load_vault()?;
    let entry = match vault.secrets.get(key) {
        Some(e) => e,
        None => return Ok(None),
    };
    if !entry.grants.is_empty() {
        let now = now_secs();
        let valid = entry
            .grants
            .iter()
            .any(|g| g.principal == principal && g.expires_at.is_none_or(|exp| now < exp));
        if !valid {
            return Err(LodeError::Message(format!(
                "access denied: '{principal}' is not granted access to '{key}'"
            )));
        }
    }
    Ok(Some(entry.value.clone()))
}

pub fn list_secrets() -> Result<Vec<SecretEntry>> {
    let vault = load_vault()?;
    let mut entries: Vec<_> = vault.secrets.into_values().collect();
    entries.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(entries)
}

#[derive(Debug, Clone, Serialize)]
pub struct SecretEntryView {
    pub key: String,
    pub scope: String,
    pub created_at: u64,
}

pub fn list_secrets_view() -> Result<Vec<SecretEntryView>> {
    let vault = load_vault()?;
    let mut entries: Vec<_> = vault
        .secrets
        .into_values()
        .map(|e| SecretEntryView {
            key: e.key,
            scope: e.scope,
            created_at: e.created_at,
        })
        .collect();
    entries.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(entries)
}

pub fn list_secrets_for(principal: &str) -> Result<Vec<SecretEntryView>> {
    let vault = load_vault()?;
    let mut entries: Vec<_> = vault
        .secrets
        .into_values()
        .filter(|e| e.grants.is_empty() || e.grants.iter().any(|g| g.principal == principal))
        .map(|e| SecretEntryView {
            key: e.key,
            scope: e.scope,
            created_at: e.created_at,
        })
        .collect();
    entries.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(entries)
}

pub fn remove_secret(key: &str) -> Result<bool> {
    let mut vault = load_vault()?;
    let existed = vault.secrets.remove(key).is_some();
    if existed {
        save_vault(&vault)?;
    }
    Ok(existed)
}

pub fn grant_access(secret_key: &str, principal: &str, permission: &str) -> Result<()> {
    let mut vault = load_vault()?;
    let entry = vault
        .secrets
        .get_mut(secret_key)
        .ok_or_else(|| LodeError::Message(format!("secret not found: {secret_key}")))?;
    entry.grants.push(Grant {
        principal: principal.to_string(),
        permission: permission.to_string(),
        granted_at: now_secs(),
        expires_at: None,
    });
    save_vault(&vault)?;
    Ok(())
}

pub fn revoke_access(secret_key: &str, principal: &str) -> Result<bool> {
    let mut vault = load_vault()?;
    let entry = vault
        .secrets
        .get_mut(secret_key)
        .ok_or_else(|| LodeError::Message(format!("secret not found: {secret_key}")))?;
    let len_before = entry.grants.len();
    entry.grants.retain(|g| g.principal != principal);
    let removed = entry.grants.len() < len_before;
    save_vault(&vault)?;
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vault() -> SecretVault {
        let mut secrets = HashMap::new();
        secrets.insert(
            "db-pass".into(),
            SecretEntry {
                key: "db-pass".into(),
                value: "s3cret".into(),
                scope: "project".into(),
                created_at: 0,
                grants: vec![Grant {
                    principal: "admin".into(),
                    permission: "read".into(),
                    granted_at: 0,
                    expires_at: None,
                }],
            },
        );
        SecretVault { secrets }
    }

    #[test]
    fn test_get_secret_found() {
        let vault = make_vault();
        let entry = vault.secrets.get("db-pass").unwrap();
        assert_eq!(entry.value, "s3cret");
    }

    #[test]
    fn test_grant_and_revoke() {
        let mut vault = make_vault();
        let entry = vault.secrets.get_mut("db-pass").unwrap();
        entry.grants.push(Grant {
            principal: "dev".into(),
            permission: "read".into(),
            granted_at: 1,
            expires_at: None,
        });
        assert_eq!(entry.grants.len(), 2);
        entry.grants.retain(|g| g.principal != "dev");
        assert_eq!(entry.grants.len(), 1);
    }

    #[test]
    fn test_list_empty_vault() {
        let vault = SecretVault {
            secrets: HashMap::new(),
        };
        let entries: Vec<_> = vault.secrets.into_values().collect();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_aes_gcm_roundtrip() {
        let key = [42u8; 32];
        let data = b"hello world";
        let encrypted = encrypt(&key, data).unwrap();
        assert_ne!(&encrypted, data);
        let decrypted = decrypt(&key, &encrypted).unwrap();
        assert_eq!(&decrypted, data);
    }

    #[test]
    fn test_aes_gcm_long_data() {
        let key = [7u8; 32];
        let data = vec![0xABu8; 200];
        let encrypted = encrypt(&key, &data).unwrap();
        assert_ne!(encrypted, data);
        let decrypted = decrypt(&key, &encrypted).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_aes_gcm_wrong_key_fails() {
        let key1 = [1u8; 32];
        let key2 = [2u8; 32];
        let data = b"secret data";
        let encrypted = encrypt(&key1, data).unwrap();
        assert!(decrypt(&key2, &encrypted).is_err());
    }

    #[test]
    fn test_get_secret_for_granted() {
        let vault = make_vault();
        let entry = vault.secrets.get("db-pass").unwrap();
        assert!(entry.grants.iter().any(|g| g.principal == "admin"));
        assert!(!entry.grants.iter().any(|g| g.principal == "bob"));
    }
}
