mod rules;

use self::rules::SafetyRules;
use crate::config::Config;
use crate::error::Result;
use crate::models::CleanupItem;

pub struct SafetyChecker {
    config: Config,
    rules: SafetyRules,
}

impl SafetyChecker {
    pub fn new(config: Config) -> Self {
        let rules = SafetyRules::from_config(&config);
        Self { config, rules }
    }

    pub fn is_safe_to_clean(&self, item: &CleanupItem) -> Result<bool> {
        if !self.config.safety.enabled {
            if !self.config.safety.only_root_can_disable || crate::utils::command::is_root() {
                return Ok(false);
            }
        }

        Ok(self.rules.check_item(item))
    }
}
