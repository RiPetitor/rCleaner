use crate::config::Config;
use crate::error::{RcleanerError, Result};
use crate::models::CleanupItem;
use chrono::{DateTime, Utc};
use fs_extra::dir::CopyOptions;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use walkdir::WalkDir;

pub async fn create_backup_manager() -> Result<BackupManager> {
    BackupManager::from_config()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backup {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub items: Vec<BackupItem>,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupItem {
    pub original_path: String,
    pub backup_path: String,
    pub size: u64,
    pub checksum: String,
}

pub struct BackupManager {
    pub backup_dir: PathBuf,
    pub max_size: u64,
}

impl BackupManager {
    pub fn new(backup_dir: PathBuf, max_size: u64) -> Result<Self> {
        fs::create_dir_all(&backup_dir)?;
        Ok(Self {
            backup_dir,
            max_size,
        })
    }

    pub fn from_config() -> Result<Self> {
        let config = Config::load(&Config::default_path()).unwrap_or_default();

        let profile = if config.safety.level.to_lowercase() == "aggressive" {
            &config.profiles.aggressive
        } else {
            &config.profiles.safe
        };

        let max_size = profile.max_backup_size_gb as u64 * 1024 * 1024 * 1024;
        let backup_dir = default_backup_dir();

        Self::new(backup_dir, max_size)
    }

    pub fn create_backup(&self, items: &[CleanupItem]) -> Result<Option<Backup>> {
        let mut candidates = collect_paths(items);
        if candidates.is_empty() {
            return Ok(None);
        }

        candidates.sort_by_key(|path| path.as_os_str().len());
        let mut selected = Vec::new();
        for path in candidates {
            if selected.iter().any(|root: &PathBuf| path.starts_with(root)) {
                continue;
            }
            selected.push(path);
        }

        let estimated_size = estimate_total_size(&selected)?;
        self.ensure_capacity(estimated_size)?;

        let id = generate_backup_id();
        let backup_root = self.backup_dir.join(&id);
        fs::create_dir_all(&backup_root)?;

        let mut backup_items = Vec::new();
        let mut total_size = 0u64;

        for path in selected {
            if !path.exists() {
                continue;
            }

            let backup_path = backup_root.join(sanitize_path(&path));
            let (size, checksum) = backup_path_data(&path, &backup_path)?;
            total_size += size;

            backup_items.push(BackupItem {
                original_path: path.to_string_lossy().to_string(),
                backup_path: backup_path.to_string_lossy().to_string(),
                size,
                checksum,
            });
        }

        let backup = Backup {
            id: id.clone(),
            timestamp: Utc::now(),
            items: backup_items,
            size: total_size,
        };

        write_metadata(&backup_root, &backup)?;
        self.enforce_max_size()?;

        Ok(Some(backup))
    }

    pub fn list_backups(&self) -> Result<Vec<Backup>> {
        let mut backups = Vec::new();
        if !self.backup_dir.exists() {
            return Ok(backups);
        }

        for entry in fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let metadata_path = entry.path().join("metadata.json");
            if let Ok(content) = fs::read_to_string(metadata_path)
                && let Ok(backup) = serde_json::from_str::<Backup>(&content)
            {
                backups.push(backup);
            }
        }

        Ok(backups)
    }

    pub fn load_backup(&self, backup_id: &str) -> Result<Backup> {
        let metadata_path = self.backup_dir.join(backup_id).join("metadata.json");
        let content = fs::read_to_string(metadata_path)?;
        let backup = serde_json::from_str::<Backup>(&content)?;
        Ok(backup)
    }

    pub fn delete_backup(&self, backup_id: &str) -> Result<()> {
        let backup_path = self.backup_dir.join(backup_id);
        if backup_path.exists() {
            fs::remove_dir_all(backup_path)?;
        }
        Ok(())
    }

    fn enforce_max_size(&self) -> Result<()> {
        if self.max_size == 0 {
            return Ok(());
        }

        let mut backups = self.list_backups()?;
        backups.sort_by_key(|backup| backup.timestamp);

        let mut total: u64 = backups.iter().map(|backup| backup.size).sum();
        for backup in backups {
            if total <= self.max_size {
                break;
            }
            self.delete_backup(&backup.id)?;
            total = total.saturating_sub(backup.size);
        }

        Ok(())
    }

    fn ensure_capacity(&self, incoming_size: u64) -> Result<()> {
        if self.max_size == 0 {
            return Ok(());
        }

        if incoming_size > self.max_size {
            return Err(RcleanerError::Backup(format!(
                "Backup size {} exceeds limit {}",
                incoming_size, self.max_size
            )));
        }

        let mut backups = self.list_backups()?;
        backups.sort_by_key(|backup| backup.timestamp);
        let mut total: u64 = backups.iter().map(|backup| backup.size).sum();

        for backup in backups {
            if total + incoming_size <= self.max_size {
                break;
            }
            self.delete_backup(&backup.id)?;
            total = total.saturating_sub(backup.size);
        }

        if total + incoming_size > self.max_size {
            return Err(RcleanerError::Backup(format!(
                "Insufficient backup capacity for size {}",
                incoming_size
            )));
        }

        Ok(())
    }
}

fn default_backup_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("rcleaner")
            .join("backups")
    } else {
        PathBuf::from("./backups")
    }
}

fn collect_paths(items: &[CleanupItem]) -> Vec<PathBuf> {
    items
        .iter()
        .filter(|item| item.can_clean)
        .filter_map(|item| item.path.as_ref())
        .map(PathBuf::from)
        .collect()
}

fn generate_backup_id() -> String {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S");
    format!("backup-{timestamp}-{}", std::process::id())
}

fn sanitize_path(path: &Path) -> PathBuf {
    let mut sanitized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::RootDir => {}
            Component::CurDir => {}
            Component::ParentDir => {}
            Component::Normal(part) => sanitized.push(part),
            _ => {}
        }
    }
    sanitized
}

fn write_metadata(backup_root: &Path, backup: &Backup) -> Result<()> {
    let metadata_path = backup_root.join("metadata.json");
    let content = serde_json::to_string_pretty(backup)?;
    fs::write(metadata_path, content)?;
    Ok(())
}

fn backup_path_data(source: &Path, dest: &Path) -> Result<(u64, String)> {
    if source.is_dir() {
        fs::create_dir_all(dest)?;
        let mut options = CopyOptions::new();
        options.copy_inside = true;
        options.overwrite = true;
        fs_extra::dir::copy(source, dest, &options)?;
    } else if source.is_file() {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, dest)?;
    } else {
        return Err(RcleanerError::NotFound(format!(
            "Backup source not found: {}",
            source.display()
        )));
    }

    let size = calculate_path_size(source)?;
    let checksum = hash_path(source)?;
    Ok((size, checksum))
}

fn estimate_total_size(paths: &[PathBuf]) -> Result<u64> {
    let mut total = 0u64;
    for path in paths {
        total = total.saturating_add(calculate_path_size(path)?);
    }
    Ok(total)
}

fn calculate_path_size(path: &Path) -> Result<u64> {
    if path.is_file() {
        let metadata = fs::metadata(path)?;
        return Ok(metadata.len());
    }

    if !path.exists() {
        return Ok(0);
    }

    let mut total = 0u64;
    for entry in WalkDir::new(path).into_iter().flatten() {
        if let Ok(metadata) = entry.metadata()
            && metadata.is_file()
        {
            total += metadata.len();
        }
    }
    Ok(total)
}

fn hash_path(path: &Path) -> Result<String> {
    let mut hasher = Sha256::new();

    if path.is_file() {
        hash_file(path, &mut hasher)?;
    } else if path.is_dir() {
        let mut entries: Vec<PathBuf> = WalkDir::new(path)
            .into_iter()
            .flatten()
            .filter(|entry| entry.path().is_file())
            .map(|entry| entry.path().to_path_buf())
            .collect();
        entries.sort();

        for entry in entries {
            let relative = entry.strip_prefix(path).unwrap_or(&entry);
            hasher.update(relative.to_string_lossy().as_bytes());
            hash_file(&entry, &mut hasher)?;
        }
    }

    Ok(to_hex(&hasher.finalize()))
}

fn hash_file(path: &Path, hasher: &mut Sha256) -> Result<()> {
    let mut file = File::open(path)?;
    let mut buffer = [0u8; 8192];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(())
}

fn to_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push_str(&format!("{:02x}", byte));
    }
    output
}
