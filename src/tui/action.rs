use crate::models::{CleanupItem, CleanupResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Screen {
    Main,
    Confirm,
    Progress,
    Settings,
    Results,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Init,
    Exit,

    Refresh,
    SetItems(Vec<CleanupItem>),
    SetStatus(Option<String>),

    ChangeTab(usize),
    NextTab,
    PrevTab,
    SelectItem(usize),
    SelectNext,
    SelectPrev,
    ToggleSelection,
    ToggleAllVisible,

    OpenConfirm,
    OpenSettings,
    BackToMain,

    StartCleanup,
    CancelCleanup,
    CleanupProgress { progress: f64, step: Option<String> },
    FinishCleanup(CleanupResult),

    ChangeSafetyLevel(SafetyLevel),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyLevel {
    Safe,
    Aggressive,
}
