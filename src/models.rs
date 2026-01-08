//! Основные модели данных для rCleaner.

use serde::{Deserialize, Serialize};

/// Элемент для очистки.
///
/// Представляет файл, директорию или пакет, который может быть удалён.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupItem {
    /// Уникальный идентификатор элемента.
    pub id: String,
    /// Отображаемое имя.
    pub name: String,
    /// Путь на файловой системе (если применимо).
    pub path: Option<String>,
    /// Размер в байтах.
    pub size: u64,
    /// Описание элемента.
    pub description: String,
    /// Категория очистки.
    pub category: CleanupCategory,
    /// Источник элемента.
    pub source: CleanupSource,
    /// Выбран ли элемент для очистки.
    pub selected: bool,
    /// Можно ли удалить элемент (проверка безопасности).
    pub can_clean: bool,
    /// Причина блокировки (если заблокирован).
    #[serde(default)]
    pub blocked_reason: Option<String>,
    /// Список зависимых пакетов.
    pub dependencies: Vec<String>,
}

/// Категория очистки.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CleanupCategory {
    /// Кэш браузеров и приложений.
    Cache,
    /// Приложения (Flatpak, Snap).
    Applications,
    /// Временные файлы.
    TempFiles,
    /// Журналы и логи.
    Logs,
    /// Старые и неиспользуемые пакеты.
    OldPackages,
    /// Старые ядра.
    OldKernels,
}

/// Источник элемента для очистки.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CleanupSource {
    /// Файловая система.
    FileSystem,
    /// Пакетный менеджер (имя менеджера).
    PackageManager(String),
    /// Контейнер (Flatpak, Snap).
    Container(String),
}

/// Результат очистки.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CleanupResult {
    /// Количество очищенных элементов.
    pub cleaned_items: usize,
    /// Освобождено байт.
    pub freed_bytes: u64,
    /// Количество пропущенных элементов.
    pub skipped_items: usize,
    /// Список ошибок.
    pub errors: Vec<String>,
}
