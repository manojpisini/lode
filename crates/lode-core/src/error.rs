use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, LodeError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Ok = 0,
    Error = 1,
    Violations = 2,
    Exists = 4,
    Schema = 6,
    VulnOrSecret = 7,
}

#[derive(Debug, Error)]
pub enum LodeError {
    #[error("{0}")]
    Message(String),

    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse TOML at {path}: {source}")]
    TomlDeserialize {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("failed to serialize TOML: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("schema version mismatch: expected {expected}, found {found}")]
    SchemaMismatch { expected: u32, found: u32 },

    #[error("project is already initialised at {path}")]
    AlreadyInitialised { path: PathBuf },

    #[error("{count} convention violation(s) found")]
    Violations { count: usize },

    #[error("{count} secret finding(s) found")]
    SecretFindings { count: usize },
}

impl LodeError {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            Self::SchemaMismatch { .. } => ExitCode::Schema,
            Self::AlreadyInitialised { .. } => ExitCode::Exists,
            Self::Violations { .. } => ExitCode::Violations,
            Self::SecretFindings { .. } => ExitCode::VulnOrSecret,
            Self::Message(_)
            | Self::Io { .. }
            | Self::TomlDeserialize { .. }
            | Self::TomlSerialize(_) => ExitCode::Error,
        }
    }
}
