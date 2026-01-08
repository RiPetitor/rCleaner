use crate::config::Config;
use crate::models::CleanupItem;
use lazy_static::lazy_static;

#[derive(Debug, Clone)]
pub struct SafetyRules {
    pub enabled: bool,
    pub whitelist: Vec<SafetyRule>,
    pub blacklist: Vec<SafetyRule>,
}

#[derive(Debug, Clone)]
pub struct SafetyRule {
    pub pattern: String,
    pub description: String,
    pub rule_type: SafetyRuleType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SafetyRuleType {
    ProtectSystemPackages,
    ProtectKernel,
    ProtectBootloader,
    ProtectUserHome,
    ProtectActiveApplications,
}

lazy_static! {
    static ref DEFAULT_RULES: Vec<SafetyRule> = vec![
        SafetyRule {
            pattern: "/boot/*".to_string(),
            description: "Защита загрузчика".to_string(),
            rule_type: SafetyRuleType::ProtectBootloader,
        },
        SafetyRule {
            pattern: "/usr/bin/*".to_string(),
            description: "Защита системных бинарников".to_string(),
            rule_type: SafetyRuleType::ProtectSystemPackages,
        },
    ];
}

impl SafetyRules {
    pub fn new() -> Self {
        Self {
            enabled: true,
            whitelist: Vec::new(),
            blacklist: Vec::new(),
        }
    }

    pub fn from_config(config: &Config) -> Self {
        let mut rules = Self::new();
        rules.enabled = config.safety.enabled;

        for path in &config.rules.whitelist.paths {
            rules.whitelist.push(SafetyRule {
                pattern: path.clone(),
                description: format!("Whitelist: {}", path),
                rule_type: SafetyRuleType::ProtectUserHome,
            });
        }

        for pattern in &config.rules.blacklist.patterns {
            rules.blacklist.push(SafetyRule {
                pattern: pattern.clone(),
                description: format!("Blacklist: {}", pattern),
                rule_type: SafetyRuleType::ProtectSystemPackages,
            });
        }

        rules
    }

    pub fn check_item(&self, item: &CleanupItem) -> bool {
        if let Some(ref path) = item.path {
            for rule in DEFAULT_RULES.iter() {
                if self.matches_rule(path, &rule.pattern) {
                    return false;
                }
            }

            for rule in &self.blacklist {
                if self.matches_rule(path, &rule.pattern) {
                    return false;
                }
            }

            for rule in &self.whitelist {
                if self.matches_rule(path, &rule.pattern) {
                    return true;
                }
            }
        }

        true
    }

    fn matches_rule(&self, path: &str, pattern: &str) -> bool {
        use regex::Regex;

        if let Ok(re) = Regex::new(pattern) {
            re.is_match(path)
        } else {
            path.contains(pattern)
        }
    }
}
