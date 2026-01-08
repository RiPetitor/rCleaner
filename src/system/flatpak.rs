use crate::error::Result;
use std::path::Path;

pub async fn run_flatpak_command(args: &[&str]) -> Result<String> {
    use tokio::process::Command;

    let output = Command::new("flatpak")
        .args(args)
        .output()
        .await?;

    if !output.status.success() {
        return Err(crate::error::RcleanerError::Command(format!(
            "flatpak command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn is_flatpak_available() -> bool {
    std::path::Path::new("/usr/bin/flatpak").exists()
        || std::path::Path::new("/usr/local/bin/flatpak").exists()
}
