use crate::error::Result;
use std::path::Path;

pub async fn run_rpm_ostree_command(args: &[&str]) -> Result<String> {
    use tokio::process::Command;

    let output = Command::new("rpm-ostree")
        .args(args)
        .output()
        .await?;

    if !output.status.success() {
        return Err(crate::error::RcleanerError::Command(format!(
            "rpm-ostree command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn is_rpm_ostree_available() -> bool {
    std::path::Path::new("/usr/bin/rpm-ostree").exists()
        || std::path::Path::new("/usr/local/bin/rpm-ostree").exists()
}
