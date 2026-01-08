pub fn is_root() -> bool {
    crate::utils::command::is_root()
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
