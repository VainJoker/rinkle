use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML deserialization error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("invalid config: {0}")]
    InvalidConfig(String),

    #[error("version error: {0}")]
    Version(String),

    #[error("conflict handling failed: {0}")]
    Conflict(String),

}
