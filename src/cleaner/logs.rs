use crate::cleaner::base::Cleaner;
use crate::error::Result;
use crate::models::{CleanupCategory, CleanupItem, CleanupResult, CleanupSource};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct LogsCleaner;

impl Default for LogsCleaner {
    fn default() -> Self {
        Self::new()
    }
}

impl LogsCleaner {
    pub fn new() -> Self {
        Self {}
    }
}

impl Cleaner for LogsCleaner {
    fn name(&self) -> &str {
        "Logs Cleaner"
    }

    fn category(&self) -> CleanupCategory {
        CleanupCategory::Logs
    }

    fn scan(&self) -> Result<Vec<CleanupItem>> {
        let mut items = Vec::new();

        let log_path = PathBuf::from("/var/log");
        if let Ok(size) = calculate_directory_size(&log_path)
            && size > 0
        {
            items.push(CleanupItem {
                id: log_path.to_string_lossy().to_string(),
                name: "System logs".to_string(),
                path: Some(log_path.to_string_lossy().to_string()),
                size,
                description: "/var/log".to_string(),
                category: self.category(),
                source: CleanupSource::FileSystem,
                selected: false,
                can_clean: true,
                blocked_reason: None,
                dependencies: Vec::new(),
            });
        }

        if let Some((size, description)) = journal_usage() {
            items.push(CleanupItem {
                id: "systemd-journal".to_string(),
                name: "systemd journal".to_string(),
                path: None,
                size,
                description,
                category: self.category(),
                source: CleanupSource::FileSystem,
                selected: false,
                can_clean: true,
                blocked_reason: None,
                dependencies: Vec::new(),
            });
        }

        Ok(items)
    }

    fn clean(&self, items: &[CleanupItem], dry_run: bool) -> Result<CleanupResult> {
        let mut result = CleanupResult::default();

        for item in items {
            if !self.can_clean(item) {
                result.skipped_items += 1;
                continue;
            }

            if item.id == "systemd-journal" {
                if dry_run {
                    log::info!("[DRY RUN] Would vacuum systemd journal");
                    result.cleaned_items += 1;
                    result.freed_bytes += item.size;
                } else {
                    let output = std::process::Command::new("journalctl")
                        .args(["--vacuum-time=7d"])
                        .output();
                    match output {
                        Ok(output) if output.status.success() => {
                            result.cleaned_items += 1;
                            result.freed_bytes += item.size;
                        }
                        Ok(output) => {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            result.errors.push(stderr.trim().to_string());
                        }
                        Err(err) => {
                            result.errors.push(err.to_string());
                        }
                    }
                }
                continue;
            }

            let Some(ref path) = item.path else {
                result.skipped_items += 1;
                continue;
            };
            let path = Path::new(path);
            if dry_run {
                log::info!("[DRY RUN] Would clean: {}", path.display());
                result.cleaned_items += 1;
                result.freed_bytes += item.size;
                continue;
            }

            match remove_rotated_logs(path) {
                Ok(removed_bytes) => {
                    result.cleaned_items += 1;
                    result.freed_bytes += removed_bytes;
                }
                Err(err) => {
                    result.errors.push(format!("{}: {}", path.display(), err));
                }
            }
        }

        Ok(result)
    }
}

fn calculate_directory_size(path: &Path) -> Result<u64> {
    if !path.exists() {
        return Ok(0);
    }

    let mut total_size = 0u64;
    for entry in WalkDir::new(path).into_iter().flatten() {
        if let Ok(metadata) = entry.metadata()
            && metadata.is_file()
        {
            total_size += metadata.len();
        }
    }
    Ok(total_size)
}

fn journal_usage() -> Option<(u64, String)> {
    let output = std::process::Command::new("journalctl")
        .args(["--disk-usage"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let size = parse_journal_size(&stdout)?;
    Some((size, stdout.trim().to_string()))
}

fn parse_journal_size(output: &str) -> Option<u64> {
    for token in output.split_whitespace() {
        if token.chars().any(|c| c.is_ascii_digit())
            && token.chars().any(|c| c.is_ascii_alphabetic())
            && let Some(size) = parse_size_to_bytes(token)
        {
            return Some(size);
        }
    }
    None
}

fn parse_size_to_bytes(value: &str) -> Option<u64> {
    let trimmed = value.trim_end_matches(['.', ',']);
    let mut number = String::new();
    let mut unit = String::new();
    for ch in trimmed.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            number.push(ch);
        } else {
            unit.push(ch);
        }
    }
    let number: f64 = number.parse().ok()?;
    let multiplier = match unit.to_lowercase().as_str() {
        "k" | "kb" | "kib" => 1024.0,
        "m" | "mb" | "mib" => 1024.0 * 1024.0,
        "g" | "gb" | "gib" => 1024.0 * 1024.0 * 1024.0,
        "t" | "tb" | "tib" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => 1.0,
    };
    Some((number * multiplier) as u64)
}

fn remove_rotated_logs(path: &Path) -> Result<u64> {
    if !path.exists() {
        return Ok(0);
    }

    let mut removed_bytes = 0u64;
    let extensions = ["gz", "xz", "bz2", "zip", "old"];

    for entry in WalkDir::new(path).into_iter().flatten() {
        let entry_path = entry.path();
        if !entry_path.is_file() {
            continue;
        }

        let file_name = entry_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        let extension = entry_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let is_rotated = extensions.contains(&extension)
            || file_name.ends_with(".1")
            || file_name.ends_with(".2")
            || file_name.ends_with(".3")
            || file_name.ends_with(".4")
            || file_name.ends_with(".5");

        if is_rotated {
            if let Ok(metadata) = entry.metadata() {
                removed_bytes += metadata.len();
            }
            let _ = std::fs::remove_file(entry_path);
        }
    }

    Ok(removed_bytes)
}
