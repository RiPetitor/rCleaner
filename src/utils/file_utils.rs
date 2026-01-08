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
