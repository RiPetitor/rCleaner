use thiserror::Error;

pub type Result<T> = std::result::Result<T, RcleanerError>;

#[derive(Debug, Error)]
pub enum RcleanerError {
    #[error("System detection error: {0}")]
    SystemDetection(String),

    #[error("Package manager error: {0}")]
    PackageManager(String),

    #[error("Cleaner error: {0}")]
    Cleaner(String),

    #[error("Safety rule error: {0}")]
    SafetyRule(String),

    #[error("Backup error: {0}")]
    Backup(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Filesystem copy error: {0}")]
    FsExtra(#[from] fs_extra::error::Error),

    #[error("Command execution error: {0}")]
    Command(String),

    #[error("Permission denied: {0}")]
    Permission(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Dependency error: {0}")]
    Dependency(String),

    #[error("Configuration error: {0}")]
    Config(String),
}
