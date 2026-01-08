use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub safety: SafetyConfig,
    pub profiles: ProfilesConfig,
    pub rules: RulesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    pub enabled: bool,
    pub only_root_can_disable: bool,
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilesConfig {
    pub safe: ProfileConfig,
    pub aggressive: ProfileConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub auto_confirm: bool,
    pub keep_recent_kernels: usize,
    pub keep_recent_deployments: usize,
    pub max_backup_size_gb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesConfig {
    pub whitelist: WhitelistConfig,
    pub blacklist: BlacklistConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistConfig {
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistConfig {
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
    pub fn load(path: &PathBuf) -> crate::error::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &PathBuf) -> crate::error::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn default_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".config")
            .join("rcleaner")
            .join("config.toml")
    }

    pub fn current_profile(&self) -> &ProfileConfig {
        if self.safety.level.to_lowercase() == "aggressive" {
            &self.profiles.aggressive
        } else {
            &self.profiles.safe
        }
    }
}
