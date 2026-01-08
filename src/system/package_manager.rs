use crate::error::Result;

pub trait PackageManager {
    fn name(&self) -> &str;
    fn version(&self) -> Result<String>;
    fn list_installed(&self) -> Result<Vec<String>>;
    fn remove_packages(&self, packages: &[String], dry_run: bool) -> Result<()>;
}
