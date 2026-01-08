use crate::error::Result;
use std::path::Path;

pub async fn run_snap_command(args: &[&str]) -> Result<String> {
    use tokio::process::Command;

    let output = Command::new("snap")
        .args(args)
        .output()
        .await?;

    if !output.status.success() {
        return Err(crate::error::RcleanerError::Command(format!(
            "snap command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn is_snap_available() -> bool {
    std::path::Path::new("/usr/bin/snap").exists()
        || std::path::Path::new("/usr/local/bin/snap").exists()
}
