use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Action {
    Init,
    Exit,

    UpdateCache,

    ChangeTab(usize),
    SelectItem(usize),
    ToggleSelection,

    StartCleanup,
    ConfirmCleanup,
    CancelCleanup,
    CleanupProgress(f64),

    OpenSettings,
    ChangeSafetyLevel(SafetyLevel),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyLevel {
    Safe,
    Aggressive,
}
