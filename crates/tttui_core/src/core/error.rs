use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse config: {0}")]
    ConfigParse(String),
    #[error("invalid config: {0}")]
    InvalidConfig(String),
}
