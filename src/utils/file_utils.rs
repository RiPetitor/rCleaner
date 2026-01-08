use std::path::{Path, PathBuf};

pub fn expand_home(path: &str) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let expanded = path.replace("~", &home);
    PathBuf::from(expanded)
}

pub fn ensure_dir_exists(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_expand_home() {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let path = expand_home("~/rcleaner-test");
        assert!(path.to_string_lossy().starts_with(&home));
    }

    #[test]
    fn test_ensure_dir_exists() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut path = std::env::temp_dir();
        path.push(format!("rcleaner-dir-{nanos}-{}", std::process::id()));

        ensure_dir_exists(&path).unwrap();
        assert!(path.exists());
    }
}
