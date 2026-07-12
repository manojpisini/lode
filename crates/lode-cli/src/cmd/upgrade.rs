#![deny(unsafe_code)]

use std::env;

use camino::Utf8PathBuf;
use lode_core::LodeError;

pub fn upgrade(
    check: bool,
    manifest: Option<Utf8PathBuf>,
    dry_run: bool,
    rollback: bool,
) -> lode_core::Result<()> {
    if rollback {
        return crate::rollback_staged_upgrade(dry_run);
    }

    let manifest_path = manifest.unwrap_or_else(crate::default_upgrade_manifest_path);
    if check {
        println!("lode {} is installed", env!("CARGO_PKG_VERSION"));
        if manifest_path.exists() {
            let manifest = crate::read_upgrade_manifest(&manifest_path)?;
            let candidate = crate::upgrade_candidate_path(&manifest_path, &manifest)?;
            let checksum = crate::file_checksum(&candidate)?;
            let status = if checksum == manifest.checksum {
                "verified"
            } else {
                "checksum-mismatch"
            };
            println!(
                "staged_upgrade\t{}\t{}\t{}",
                manifest.version, candidate, status
            );
        } else {
            println!("staged_upgrade\tnone");
        }
        println!(
            "network upgrade checks are disabled; provide --manifest for local staged upgrades"
        );
        return Ok(());
    }

    if !manifest_path.exists() {
        return Err(LodeError::Message(format!(
            "upgrade manifest not found: {manifest_path}; place latest.json in cache/upgrade or pass --manifest"
        )));
    }

    let manifest = crate::read_upgrade_manifest(&manifest_path)?;
    let candidate = crate::upgrade_candidate_path(&manifest_path, &manifest)?;
    let candidate_checksum = crate::file_checksum(&candidate)?;
    if candidate_checksum != manifest.checksum {
        return Err(LodeError::Message(format!(
            "upgrade checksum mismatch for {candidate}: expected {}, found {}",
            manifest.checksum, candidate_checksum
        )));
    }
    let current_executable = env::current_exe().map_err(|source| LodeError::Io {
        path: "current_exe".into(),
        source,
    })?;
    let current_executable = Utf8PathBuf::from_path_buf(current_executable).map_err(|path| {
        LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
    })?;
    let current_checksum =
        crate::file_checksum(&current_executable).unwrap_or_else(|_| "unavailable".to_string());
    let state = crate::UpgradeState {
        schema_version: 3,
        version: manifest.version.clone(),
        candidate: candidate.clone(),
        checksum: candidate_checksum,
        current_executable: current_executable.to_string(),
        current_checksum,
        staged_at: crate::now_timestamp(),
        activated: false,
    };

    if dry_run {
        println!("would verify staged upgrade {}", state.version);
        println!("would record upgrade state at {}", crate::upgrade_state_path()?);
        println!("candidate\t{}", state.candidate);
        println!("current_executable\t{}", state.current_executable);
        return Ok(());
    }

    crate::write_upgrade_state(&state)?;
    println!("upgrade staged\t{}", state.version);
    println!("candidate\t{}", state.candidate);
    println!("state\t{}", crate::upgrade_state_path()?);
    println!("activate manually after review; rollback with `lode upgrade --rollback`");
    Ok(())
}
