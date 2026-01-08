//! Управление конфигурацией rCleaner.
//!
//! Конфигурация хранится в TOML файле `~/.config/rcleaner/config.toml`.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Главная структура конфигурации.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Настройки безопасности.
    pub safety: SafetyConfig,
    /// Профили очистки.
    pub profiles: ProfilesConfig,
    /// Правила whitelist/blacklist.
    pub rules: RulesConfig,
}

/// Настройки безопасности.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    /// Включена ли система безопасности.
    pub enabled: bool,
    /// Может ли только root отключить безопасность.
    pub only_root_can_disable: bool,
    /// Уровень безопасности: "safe" или "aggressive".
    pub level: String,
}

/// Профили очистки (safe и aggressive).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilesConfig {
    /// Безопасный профиль.
    pub safe: ProfileConfig,
    /// Агрессивный профиль.
    pub aggressive: ProfileConfig,
}

/// Настройки профиля очистки.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    /// Автоматическое подтверждение очистки.
    pub auto_confirm: bool,
    /// Сколько последних ядер сохранять.
    pub keep_recent_kernels: usize,
    /// Сколько последних deployments сохранять (rpm-ostree).
    pub keep_recent_deployments: usize,
    /// Максимальный размер бэкапа в ГБ.
    pub max_backup_size_gb: usize,
}

/// Правила whitelist и blacklist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesConfig {
    /// Белый список защищённых путей.
    pub whitelist: WhitelistConfig,
    /// Чёрный список паттернов для блокировки.
    pub blacklist: BlacklistConfig,
}

/// Конфигурация белого списка.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistConfig {
    /// Защищённые пути.
    pub paths: Vec<String>,
}

/// Конфигурация чёрного списка.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistConfig {
    /// Паттерны для блокировки.
    pub patterns: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            safety: SafetyConfig {
                enabled: true,
                only_root_can_disable: true,
                level: "safe".to_string(),
            },
            profiles: ProfilesConfig {
                safe: ProfileConfig {
                    auto_confirm: false,
                    keep_recent_kernels: 2,
                    keep_recent_deployments: 2,
                    max_backup_size_gb: 10,
                },
                aggressive: ProfileConfig {
                    auto_confirm: true,
                    keep_recent_kernels: 1,
                    keep_recent_deployments: 1,
                    max_backup_size_gb: 5,
                },
            },
            rules: RulesConfig {
                whitelist: WhitelistConfig {
                    paths: vec![
                        "~/.config".to_string(),
                        "~/Documents".to_string(),
                        "~/Projects".to_string(),
                    ],
                },
                blacklist: BlacklistConfig {
                    patterns: vec!["*.tmp".to_string(), "*.log".to_string()],
                },
            },
        }
    }
}

impl Config {
    /// Загружает конфигурацию из файла.
    ///
    /// # Errors
    ///
    /// Возвращает ошибку, если файл не существует или имеет неверный формат.
    pub fn load(path: &PathBuf) -> crate::error::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Сохраняет конфигурацию в файл.
    ///
    /// Создаёт родительские директории, если они не существуют.
    pub fn save(&self, path: &PathBuf) -> crate::error::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Возвращает путь к конфигурации по умолчанию.
    ///
    /// `~/.config/rcleaner/config.toml`
    pub fn default_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".config")
            .join("rcleaner")
            .join("config.toml")
    }

    /// Возвращает текущий активный профиль на основе `safety.level`.
    pub fn current_profile(&self) -> &ProfileConfig {
        if self.safety.level.to_lowercase() == "aggressive" {
            &self.profiles.aggressive
        } else {
            &self.profiles.safe
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_config_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut path = std::env::temp_dir();
        path.push(format!("rcleaner-config-{nanos}-{}", std::process::id()));
        path.push("nested");
        path.push("config.toml");
        path
    }

    #[test]
    fn test_save_and_load_config() {
        let path = temp_config_path();
        let mut config = Config::default();
        config.safety.level = "aggressive".to_string();
        config.profiles.aggressive.max_backup_size_gb = 3;

        config.save(&path).unwrap();
        assert!(path.exists());

        let loaded = Config::load(&path).unwrap();
        assert_eq!(loaded.safety.level, "aggressive");
        assert_eq!(loaded.profiles.aggressive.max_backup_size_gb, 3);
    }

    #[test]
    fn test_current_profile() {
        let mut config = Config::default();
        assert!(!config.current_profile().auto_confirm);
        config.safety.level = "aggressive".to_string();
        assert!(config.current_profile().auto_confirm);
    }
}
