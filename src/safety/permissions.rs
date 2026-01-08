use crate::error::{RcleanerError, Result};

pub fn is_root() -> bool {
    crate::utils::command::is_root()
}

pub fn check_permissions() -> Result<bool> {
    Ok(is_root())
}

pub fn require_root() -> Result<()> {
    if is_root() {
        Ok(())
    } else {
        Err(RcleanerError::Permission(
            "Root permissions required".to_string(),
        ))
    }
}

pub fn can_clean_path(path: &str) -> bool {
    if is_root() {
        return true;
    }

    if path.starts_with("/tmp") || path.starts_with("/var/tmp") {
        return true;
    }

    if let Ok(home) = std::env::var("HOME") {
        if path.starts_with(&home) {
            return true;
        }
    }

    false
}
