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
            Self::NotFound(_) | Self::InvalidParams(_) => -32602,
            Self::Internal(_) | Self::Io(_) => -32603,
            Self::Lode(_) | Self::Toml(_) | Self::TomlSer(_) => -32000,
            Self::Serde(_) => -32700,
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
