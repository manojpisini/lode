use std::fmt;

#[derive(Debug)]
pub enum McpError {
    NotFound(String),
    InvalidParams(String),
    Internal(String),
    Io(std::io::Error),
    Lode(lode_core::LodeError),
    Serde(serde_json::Error),
    Toml(toml::de::Error),
    TomlSer(toml::ser::Error),
}

impl fmt::Display for McpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "Not found: {msg}"),
            Self::InvalidParams(msg) => write!(f, "Invalid params: {msg}"),
            Self::Internal(msg) => write!(f, "Internal error: {msg}"),
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Lode(e) => {
                let redacted = lode_core::redact(&e.to_string());
                write!(f, "Lode error: {redacted}")
            }
            Self::Serde(e) => write!(f, "JSON error: {e}"),
            Self::Toml(e) => write!(f, "TOML parse error: {e}"),
            Self::TomlSer(e) => write!(f, "TOML serialize error: {e}"),
        }
    }
}

impl McpError {
    pub fn code(&self) -> i64 {
        match self {
            Self::NotFound(_) => -32602,
            Self::InvalidParams(_) => -32602,
            Self::Internal(_) => -32603,
            Self::Io(_) => -32603,
            Self::Lode(_) => -32000,
            Self::Serde(_) => -32700,
            Self::Toml(_) => -32000,
            Self::TomlSer(_) => -32000,
        }
    }
}

impl From<std::io::Error> for McpError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<lode_core::LodeError> for McpError {
    fn from(e: lode_core::LodeError) -> Self {
        Self::Lode(e)
    }
}

impl From<serde_json::Error> for McpError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e)
    }
}

impl From<toml::de::Error> for McpError {
    fn from(e: toml::de::Error) -> Self {
        Self::Toml(e)
    }
}

impl From<toml::ser::Error> for McpError {
    fn from(e: toml::ser::Error) -> Self {
        Self::TomlSer(e)
    }
}

#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn not_found_code() {
        let err = McpError::NotFound("x".to_string());
        assert_eq!(err.code(), -32602);
    }

    #[test]
    fn invalid_params_code() {
        let err = McpError::InvalidParams("x".to_string());
        assert_eq!(err.code(), -32602);
    }

    #[test]
    fn internal_code() {
        let err = McpError::Internal("x".to_string());
        assert_eq!(err.code(), -32603);
    }

    #[test]
    fn io_code() {
        let err = McpError::Io(std::io::Error::new(std::io::ErrorKind::Other, "x"));
        assert_eq!(err.code(), -32603);
    }

    #[test]
    fn lode_code() {
        let err = McpError::Lode(lode_core::LodeError::Message("x".to_string()));
        assert_eq!(err.code(), -32000);
    }

    #[test]
    fn serde_code() {
        let err = McpError::Serde(serde_json::from_str::<serde_json::Value>("").unwrap_err());
        assert_eq!(err.code(), -32700);
    }

    #[test]
    fn not_found_display() {
        let err = McpError::NotFound("file".to_string());
        assert_eq!(err.to_string(), "Not found: file");
    }

    #[test]
    fn display_messages_are_readable() {
        let cases: Vec<McpError> = vec![
            McpError::NotFound("x".into()),
            McpError::InvalidParams("x".into()),
            McpError::Internal("x".into()),
            McpError::Io(std::io::Error::new(std::io::ErrorKind::Other, "x")),
            McpError::Lode(lode_core::LodeError::Message("x".into())),
        ];
        for case in cases {
            assert!(!case.to_string().is_empty());
        }
    }
}
