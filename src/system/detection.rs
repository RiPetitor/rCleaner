use crate::error::Result;

pub fn detect_system() -> Result<SystemInfo> {
    todo!("Implement system detection")
}

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub system_type: SystemType,
    pub available_managers: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum SystemType {
    AtomicRpmOstree,
    Desktop(String),
}
