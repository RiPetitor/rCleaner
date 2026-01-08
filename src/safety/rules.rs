use crate::config::Config;
use crate::models::CleanupItem;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct SafetyRules {
    pub enabled: bool,
    pub only_root_can_disable: bool,
    pub whitelist: Vec<SafetyRule>,
    pub blacklist: Vec<SafetyRule>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SafetyRule {
    pub pattern: String,
    pub description: String,
    pub rule_type: SafetyRuleType,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyRuleType {
    ProtectSystemPackages,
    ProtectKernel,
    ProtectBootloader,
    ProtectUserHome,
    ProtectActiveApplications,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct SafetyRuleTemplate {
    pattern: &'static str,
    description: &'static str,
    rule_type: SafetyRuleType,
}

const DEFAULT_RULES: &[SafetyRuleTemplate] = &[
    SafetyRuleTemplate {
        pattern: "/boot/*",
        description: "Защита загрузчика",
        rule_type: SafetyRuleType::ProtectBootloader,
    },
    SafetyRuleTemplate {
        pattern: "/boot/efi/*",
        description: "Защита загрузчика",
        rule_type: SafetyRuleType::ProtectBootloader,
    },
    SafetyRuleTemplate {
        pattern: "/lib/modules/*",
        description: "Защита ядра",
        rule_type: SafetyRuleType::ProtectKernel,
    },
    SafetyRuleTemplate {
        pattern: "/usr/bin/*",
        description: "Защита системных бинарников",
        rule_type: SafetyRuleType::ProtectSystemPackages,
    },
    SafetyRuleTemplate {
        pattern: "/usr/sbin/*",
        description: "Защита системных бинарников",
        rule_type: SafetyRuleType::ProtectSystemPackages,
    },
    SafetyRuleTemplate {
        pattern: "/usr/lib/*",
        description: "Защита системных библиотек",
        rule_type: SafetyRuleType::ProtectSystemPackages,
    },
    SafetyRuleTemplate {
        pattern: "/usr/lib64/*",
        description: "Защита системных библиотек",
        rule_type: SafetyRuleType::ProtectSystemPackages,
    },
    SafetyRuleTemplate {
        pattern: "/bin/*",
        description: "Защита системных бинарников",
        rule_type: SafetyRuleType::ProtectSystemPackages,
    },
    SafetyRuleTemplate {
        pattern: "/sbin/*",
        description: "Защита системных бинарников",
        rule_type: SafetyRuleType::ProtectSystemPackages,
    },
    SafetyRuleTemplate {
        pattern: "/lib/*",
        description: "Защита системных библиотек",
        rule_type: SafetyRuleType::ProtectSystemPackages,
    },
    SafetyRuleTemplate {
        pattern: "/lib64/*",
        description: "Защита системных библиотек",
        rule_type: SafetyRuleType::ProtectSystemPackages,
    },
    SafetyRuleTemplate {
        pattern: "/etc/*",
        description: "Защита системной конфигурации",
        rule_type: SafetyRuleType::ProtectSystemPackages,
    },
    SafetyRuleTemplate {
        pattern: "/root/*",
        description: "Защита системных данных",
        rule_type: SafetyRuleType::ProtectSystemPackages,
    },
    SafetyRuleTemplate {
        pattern: "/var/lib/*",
        description: "Защита системного состояния",
        rule_type: SafetyRuleType::ProtectSystemPackages,
    },
];

impl SafetyRules {
    pub fn new() -> Self {
        Self {
            enabled: true,
            only_root_can_disable: true,
            whitelist: Vec::new(),
            blacklist: Vec::new(),
        }
    }

    pub fn from_config(config: &Config) -> Self {
        let mut rules = Self::new();
        rules.enabled = config.safety.enabled;
        rules.only_root_can_disable = config.safety.only_root_can_disable;

        for path in &config.rules.whitelist.paths {
            let normalized = normalize_pattern(path);
            rules.whitelist.push(SafetyRule {
                pattern: normalized.clone(),
                description: format!("Whitelist: {}", normalized),
                rule_type: SafetyRuleType::ProtectUserHome,
            });
        }

        for pattern in &config.rules.blacklist.patterns {
            let normalized = normalize_pattern(pattern);
            rules.blacklist.push(SafetyRule {
                pattern: normalized.clone(),
                description: format!("Blacklist: {}", normalized),
                rule_type: SafetyRuleType::ProtectSystemPackages,
            });
        }

        rules
    }

    pub fn check_item(&self, item: &CleanupItem) -> bool {
        if let Some(ref path) = item.path {
            for rule in DEFAULT_RULES.iter() {
                if self.matches_rule(path, rule.pattern) {
                    return false;
                }
            }

            for rule in &self.whitelist {
                if self.matches_rule(path, &rule.pattern) {
                    return false;
                }
            }

            for rule in &self.blacklist {
                if self.matches_rule(path, &rule.pattern) {
                    return false;
                }
            }
        }

        true
    }

    fn matches_rule(&self, path: &str, pattern: &str) -> bool {
        let expanded = normalize_pattern(pattern);

        if has_glob(&expanded) {
            if let Ok(re) = Regex::new(&glob_to_regex(&expanded)) {
                return re.is_match(path);
            }
        }

        if expanded.starts_with('/') {
            path.starts_with(&expanded)
        } else {
            path.contains(&expanded)
        }
    }
}

fn normalize_pattern(pattern: &str) -> String {
    expand_tilde(pattern.trim())
}

fn expand_tilde(value: &str) -> String {
    if value == "~" || value.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            if value == "~" {
                return home;
            }
            return format!("{home}{}", &value[1..]);
        }
    }
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CleanupCategory, CleanupItem, CleanupSource};

    fn item_with_path(path: &str) -> CleanupItem {
        CleanupItem {
            id: path.to_string(),
            name: "test".to_string(),
            path: Some(path.to_string()),
            size: 0,
            description: "test".to_string(),
            category: CleanupCategory::Cache,
            source: CleanupSource::FileSystem,
            selected: false,
            can_clean: true,
            dependencies: Vec::new(),
        }
    }

    #[test]
    fn test_default_rules_block_system_paths() {
        let config = Config::default();
        let rules = SafetyRules::from_config(&config);
        let item = item_with_path("/usr/bin/rcleaner-test");
        assert!(!rules.check_item(&item));
    }

    #[test]
    fn test_whitelist_blocks_path() {
        let mut config = Config::default();
        config.rules.whitelist.paths = vec!["/tmp/rcleaner-protect".to_string()];
        let rules = SafetyRules::from_config(&config);
        let item = item_with_path("/tmp/rcleaner-protect/file");
        assert!(!rules.check_item(&item));
    }

    #[test]
    fn test_blacklist_blocks_pattern() {
        let mut config = Config::default();
        config.rules.blacklist.patterns = vec!["*.log".to_string()];
        let rules = SafetyRules::from_config(&config);
        let item = item_with_path("/tmp/rcleaner-test.log");
        assert!(!rules.check_item(&item));
    }

    #[test]
    fn test_safe_path_allowed() {
        let config = Config::default();
        let rules = SafetyRules::from_config(&config);
        let item = item_with_path("/tmp/rcleaner-ok");
        assert!(rules.check_item(&item));
    }
}

fn has_glob(pattern: &str) -> bool {
    pattern.contains('*') || pattern.contains('?')
}

fn glob_to_regex(pattern: &str) -> String {
    let mut output = String::from("^");
    for ch in pattern.chars() {
        match ch {
            '*' => output.push_str(".*"),
            '?' => output.push('.'),
            _ => output.push_str(&regex::escape(&ch.to_string())),
        }
    }
    output.push('$');
    output
}
