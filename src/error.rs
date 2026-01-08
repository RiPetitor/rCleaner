//! Типы ошибок для rCleaner.

use thiserror::Error;

/// Тип Result с ошибкой [`RcleanerError`].
pub type Result<T> = std::result::Result<T, RcleanerError>;

/// Перечисление всех возможных ошибок в rCleaner.
#[derive(Debug, Error)]
pub enum RcleanerError {
    /// Ошибка определения системы.
    #[error("System detection error: {0}")]
    SystemDetection(String),

    /// Ошибка пакетного менеджера.
    #[error("Package manager error: {0}")]
    PackageManager(String),

    /// Ошибка модуля очистки.
    #[error("Cleaner error: {0}")]
    Cleaner(String),

    /// Ошибка правил безопасности.
    #[error("Safety rule error: {0}")]
    SafetyRule(String),

    /// Ошибка резервного копирования.
    #[error("Backup error: {0}")]
    Backup(String),

    /// Ошибка ввода-вывода.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Ошибка парсинга TOML.
    #[error("Parse error: {0}")]
    Parse(#[from] toml::de::Error),

    /// Ошибка сериализации TOML.
    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    /// Ошибка сериализации JSON.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Ошибка копирования файлов.
    #[error("Filesystem copy error: {0}")]
    FsExtra(#[from] fs_extra::error::Error),

    /// Ошибка выполнения команды.
    #[error("Command execution error: {0}")]
    Command(String),

    /// Отказ в доступе.
    #[error("Permission denied: {0}")]
    Permission(String),

    /// Объект не найден.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Неверный ввод.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Ошибка зависимостей пакетов.
    #[error("Dependency error: {0}")]
    Dependency(String),

    /// Ошибка конфигурации.
    #[error("Configuration error: {0}")]
    Config(String),
}
