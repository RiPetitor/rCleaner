use std::process::Command;

pub fn run_command(name: &str, args: &[&str]) -> crate::error::Result<(bool, String)> {
    let output = Command::new(name).args(args).output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let success = output.status.success();

    Ok((success, format!("{}\n{}", stdout, stderr)))
}

pub fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}
