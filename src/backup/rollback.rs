use crate::backup::manager::BackupManager;
use crate::error::Result;
use std::fs;
use std::path::Path;

pub async fn perform_rollback(backup_id: &str) -> Result<()> {
    let manager = BackupManager::from_config()?;
    let backup = manager.load_backup(backup_id)?;

    for item in backup.items {
        let original = Path::new(&item.original_path);
        let backup_path = Path::new(&item.backup_path);

        if backup_path.is_dir() {
            restore_directory(backup_path, original)?;
        } else if backup_path.is_file() {
            restore_file(backup_path, original)?;
        }
    }

    Ok(())
}

fn restore_file(backup_path: &Path, original: &Path) -> Result<()> {
    if let Some(parent) = original.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(backup_path, original)?;
    Ok(())
}

fn restore_directory(backup_path: &Path, original: &Path) -> Result<()> {
    fs::create_dir_all(original)?;
    let mut options = fs_extra::dir::CopyOptions::new();
    options.copy_inside = true;
    options.overwrite = true;
    fs_extra::dir::copy(backup_path, original, &options)?;
    Ok(())
}
