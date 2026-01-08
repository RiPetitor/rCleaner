use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupItem {
    pub id: String,
    pub name: String,
    pub path: Option<String>,
    pub size: u64,
    pub description: String,
    pub category: CleanupCategory,
    pub source: CleanupSource,
    pub selected: bool,
    pub can_clean: bool,
    #[serde(default)]
    pub blocked_reason: Option<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CleanupCategory {
    Cache,
    Applications,
    TempFiles,
    Logs,
    OldPackages,
    OldKernels,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CleanupSource {
    FileSystem,
    PackageManager(String),
    Container(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CleanupResult {
    pub cleaned_items: usize,
    pub freed_bytes: u64,
    pub skipped_items: usize,
    pub errors: Vec<String>,
}
