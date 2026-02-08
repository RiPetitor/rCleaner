//! Действия для Flux-архитектуры.

use crate::models::{CleanupItem, CleanupResult};
use serde::{Deserialize, Serialize};

/// Экраны приложения.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Screen {
    Main,
    Confirm,
    Progress,
    Settings,
    Results,
}

/// Тип редактирования в настройках.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingsEdit {
    Whitelist,
    Blacklist,
}

/// Действия пользователя и системы.
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
    SelectPageDown(usize),
    SelectPageUp(usize),
    SelectFirst,
    SelectLast,

    ToggleSelection,
    ToggleAllVisible,

    StartSearch,
    EndSearch,
    ClearSearch,
    AppendSearch(char),
    BackspaceSearch,

    BeginSettingsEdit(SettingsEdit, String),
    EndSettingsEdit,
    AppendSettingsInput(char),
    BackspaceSettingsInput,

    OpenConfirm,
    OpenSettings,
    NextSettingsBlock,
    SettingsRowNext(usize),
    SettingsRowPrev,
    BackToMain,

    StartCleanup,
    CancelCleanup,
    CleanupProgress { progress: f64, step: Option<String> },
    FinishCleanup(CleanupResult),

    ChangeSafetyLevel(SafetyLevel),
}

/// Уровень безопасности.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyLevel {
    Safe,
    Aggressive,
}
