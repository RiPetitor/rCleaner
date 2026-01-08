//! Действия для Flux-архитектуры.

use crate::models::{CleanupItem, CleanupResult};
use serde::{Deserialize, Serialize};

/// Экраны приложения.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Screen {
    /// Главный экран со списком элементов.
    Main,
    /// Экран подтверждения очистки.
    Confirm,
    /// Экран прогресса очистки.
    Progress,
    /// Экран настроек.
    Settings,
    /// Экран результатов.
    Results,
}

/// Тип редактирования в настройках.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingsEdit {
    /// Редактирование whitelist.
    Whitelist,
    /// Редактирование blacklist.
    Blacklist,
}

/// Действия пользователя и системы.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    /// Инициализация приложения.
    Init,
    /// Выход из приложения.
    Exit,

    /// Обновить список элементов.
    Refresh,
    /// Установить элементы.
    SetItems(Vec<CleanupItem>),
    /// Установить статусное сообщение.
    SetStatus(Option<String>),

    /// Переключить на вкладку.
    ChangeTab(usize),
    /// Следующая вкладка.
    NextTab,
    /// Предыдущая вкладка.
    PrevTab,
    /// Выбрать элемент по индексу.
    SelectItem(usize),
    /// Выбрать следующий элемент.
    SelectNext,
    /// Выбрать предыдущий элемент.
    SelectPrev,
    /// Переключить выбор текущего элемента.
    ToggleSelection,
    /// Переключить выбор всех видимых элементов.
    ToggleAllVisible,
    /// Начать поиск.
    StartSearch,
    /// Завершить поиск.
    EndSearch,
    /// Очистить поисковый запрос.
    ClearSearch,
    /// Добавить символ к поисковому запросу.
    AppendSearch(char),
    /// Удалить последний символ из поиска.
    BackspaceSearch,
    /// Начать редактирование настроек.
    BeginSettingsEdit(SettingsEdit, String),
    /// Завершить редактирование настроек.
    EndSettingsEdit,
    /// Добавить символ к вводу настроек.
    AppendSettingsInput(char),
    /// Удалить последний символ из ввода настроек.
    BackspaceSettingsInput,

    /// Открыть экран подтверждения.
    OpenConfirm,
    /// Открыть настройки.
    OpenSettings,
    /// Вернуться на главный экран.
    BackToMain,

    /// Начать очистку.
    StartCleanup,
    /// Отменить очистку.
    CancelCleanup,
    /// Обновить прогресс очистки.
    CleanupProgress {
        /// Прогресс (0.0..1.0).
        progress: f64,
        /// Текущий шаг.
        step: Option<String>,
    },
    /// Завершить очистку.
    FinishCleanup(CleanupResult),

    /// Изменить уровень безопасности.
    ChangeSafetyLevel(SafetyLevel),
}

/// Уровень безопасности.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyLevel {
    /// Безопасный режим (рекомендуется).
    Safe,
    /// Агрессивный режим.
    Aggressive,
}
