// Copyright (C) 2026 RiPetitor
// SPDX-License-Identifier: GPL-3.0-or-later

//! Интернационализация (i18n) для rCleaner.
//!
//! Компактные переводы — короткие строки, не перегружающие TUI.

use std::sync::atomic::{AtomicU8, Ordering};

static LANG: AtomicU8 = AtomicU8::new(0); // 0 = En, 1 = Ru

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En = 0,
    Ru = 1,
}

impl Lang {
    pub fn label(self) -> &'static str {
        match self {
            Lang::En => "EN",
            Lang::Ru => "RU",
        }
    }
}

pub fn current_lang() -> Lang {
    match LANG.load(Ordering::Relaxed) {
        1 => Lang::Ru,
        _ => Lang::En,
    }
}

pub fn set_lang(lang: Lang) {
    LANG.store(lang as u8, Ordering::Relaxed);
}

pub fn toggle_lang() -> Lang {
    let next = match current_lang() {
        Lang::En => Lang::Ru,
        Lang::Ru => Lang::En,
    };
    set_lang(next);
    next
}

/// Макрос для получения перевода: `t!("key")`.
/// Каждый ключ возвращает &'static str.
macro_rules! define_translations {
    ( $( $key:ident => $en:expr, $ru:expr; )* ) => {
        $(
            #[inline]
            pub fn $key() -> &'static str {
                match current_lang() {
                    Lang::En => $en,
                    Lang::Ru => $ru,
                }
            }
        )*
    };
}

// ── UI labels ──────────────────────────────────────────────

define_translations! {
    // Tabs
    tab_cache        => "Cache",       "Кэш";
    tab_apps         => "Apps",        "Прилож.";
    tab_temp         => "Temp",        "Врем.";
    tab_logs         => "Logs",        "Логи";
    tab_packages     => "Packages",    "Пакеты";
    tab_kernels      => "Kernels",     "Ядра";
    tab_system       => "System",      "Система";

    // Main screen
    items            => "Items",       "Элементы";
    details          => "Details",     "Детали";
    summary          => "Summary",     "Сводка";
    search           => "Search",      "Поиск";
    no_items         => "No items found. Press [R] to rescan.", "Ничего не найдено. [R] — пересканировать.";
    scanning         => "Scanning...", "Сканирование...";
    scan_complete    => "Scan complete.", "Скан завершён.";
    scan_failed      => "Scan failed. See logs.", "Ошибка скана. См. логи.";
    loaded_cache     => "Loaded cached results.", "Загружены кэшированные результаты.";
    no_selected      => "No items selected.", "Ничего не выбрано.";
    scan_in_progress => "Scan already in progress.", "Скан уже выполняется.";
    type_to_search   => "Type to search...", "Введите запрос...";
    matches_label    => "matches",     "совпад.";
    search_mode      => "Search mode", "Режим поиска";
    selected_label   => "Selected",    "Выбрано";
    total_label      => "Total",       "Всего";
    size_label       => "Size",        "Размер";

    // Details panel
    status_selected  => "Status: Selected", "Статус: Выбран";
    status_available => "Status: Available", "Статус: Доступен";
    status_blocked   => "Blocked",     "Заблокирован";
    safety_rules     => "Safety rules", "Правила безопасности";
    source_fs        => "Source: Filesystem", "Источник: ФС";
    select_to_see    => "Select an item to see details.", "Выберите элемент для деталей.";

    // Hotkeys
    key_category     => "[Tab] Category",  "[Tab] Категория";
    key_select       => "[Space] Select",  "[Space] Выбрать";
    key_all          => "[A] All",         "[A] Все";
    key_clean        => "[Enter] Clean",   "[Enter] Очистка";
    key_settings     => "[S] Settings",    "[S] Настройки";
    key_refresh      => "[R] Refresh",     "[R] Обновить";
    key_search       => "[/] Search",      "[/] Поиск";
    key_quit         => "[Q] Quit",        "[Q] Выход";

    // Confirm screen
    confirm_title    => "Confirm Cleanup", "Подтвердите очистку";
    items_to_clean   => "Items to clean",  "К удалению";
    dry_run_label    => "DRY RUN",         "ТЕСТ";
    execute_label    => "EXECUTE",         "ВЫПОЛНИТЬ";
    items_count      => "Items",           "Элементов";
    mode_label       => "Mode",            "Режим";
    key_confirm      => "[Y] Confirm",     "[Y] Да";
    key_cancel       => "[N] Cancel",      "[N] Нет";
    key_back         => "[Esc] Back",      "[Esc] Назад";
    and_more         => "and",             "и ещё";

    // Progress
    progress_label   => "Progress",        "Прогресс";
    current_step     => "Current Step",    "Текущий шаг";
    working          => "Working...",      "Работаем...";
    key_cancel_esc   => "[Esc] Cancel",    "[Esc] Отмена";

    // Results
    results_title    => "Results",         "Результаты";
    cleaned          => "Cleaned:",        "Очищено:";
    freed            => "Freed:",          "Освобождено:";
    skipped          => "Skipped:",        "Пропущено:";
    errors_label     => "Errors:",         "Ошибки:";
    errors_title     => "Errors",          "Ошибки";
    no_errors        => "No errors.",      "Без ошибок.";
    no_results       => "No cleanup results available.", "Результатов нет.";
    key_enter_back   => "[Enter] Back",    "[Enter] Назад";

    // Settings
    settings_title   => "Settings",        "Настройки";
    config_title     => "Configuration",   "Конфигурация";
    safety_label     => "Safety:",         "Безопасность:";
    root_only        => "Root-only:",      "Только root:";
    auto_confirm     => "Auto confirm:",   "Автоподтверждение:";
    dry_run_setting  => "Dry run:",        "Тест-режим:";
    temp_age         => "Temp age:",       "Возраст врем.:";
    config_label     => "Config:",         "Конфиг:";
    safety_level     => "Safety Level",    "Уровень безоп.";
    safe_desc        => "conservative, recommended", "консервативный, рекомендуется";
    aggressive_desc  => "faster, fewer checks", "быстрее, меньше проверок";
    rules_title      => "Rules",           "Правила";
    whitelist_label  => "Whitelist",       "Белый список";
    blacklist_label  => "Blacklist",       "Чёрный список";
    edit_title       => "Edit",            "Редактирование";
    key_tab_block    => "[Tab] Block",     "[Tab] Блок";
    key_arrows       => "[↑↓] Navigate",  "[↑↓] Навигация";
    key_enter_toggle => "[Enter] Toggle",  "[Enter] Переключить";
    key_esc_back     => "[Esc] Back",      "[Esc] Назад";
    key_save         => "[Enter] Save",    "[Enter] Сохранить";
    key_esc_cancel   => "[Esc] Cancel",    "[Esc] Отмена";
    lang_setting     => "Language:",       "Язык:";

    // Cleaners
    cl_cache         => "Cache Cleaner",       "Очистка кэша";
    cl_apps          => "Applications Cleaner", "Очистка приложений";
    cl_temp          => "Temp Files Cleaner",   "Очистка врем. файлов";
    cl_logs          => "Logs Cleaner",         "Очистка логов";
    cl_packages      => "Old Packages Cleaner", "Очистка пакетов";
    cl_kernels       => "Old Kernels Cleaner",  "Очистка ядер";
    cl_system        => "System Packages Cleaner", "Очистка сист. пакетов";

    // Status messages
    config_saved     => "Config saved.",    "Конфиг сохранён.";
    rules_updated    => "Rules updated.",   "Правила обновлены.";
    safety_updated   => "Safety updated.",  "Безопасность обновлена.";
    policy_updated   => "Safety policy updated.", "Политика обновлена.";
    dryrun_updated   => "Dry-run updated.", "Тест-режим обновлён.";
    config_fail      => "Failed to save config.", "Ошибка сохранения конфига.";
    root_required    => "Root required to disable safety.", "Нужен root для отключения безопасности.";
    root_change      => "Root required to change this setting.", "Нужен root для изменения.";
    invalid_input    => "Invalid input",   "Неверный ввод";
    default_config   => "Using default config.", "Используется конфиг по умолчанию.";
    config_not_saved => "Using defaults (config not saved).", "По умолчанию (конфиг не сохранён).";

    // Descriptions
    desc_user_cache  => "User cache",           "Пользовательский кэш";
    desc_thumbnails  => "Thumbnails",           "Миниатюры";
    desc_firefox     => "Firefox cache",        "Кэш Firefox";
    desc_chrome      => "Chrome cache",         "Кэш Chrome";
    desc_chromium    => "Chromium cache",        "Кэш Chromium";
    desc_brave       => "Brave cache",          "Кэш Brave";
    desc_shader      => "Shader cache",         "Кэш шейдеров";
    desc_pip         => "pip cache",            "Кэш pip";
    desc_npm         => "npm cache",            "Кэш npm";
    desc_cargo       => "Cargo registry cache", "Кэш Cargo registry";
    desc_tmp         => "Temporary files",      "Временные файлы";
    desc_trash       => "Trash",                "Корзина";
    desc_coredumps   => "Core dumps",           "Дампы ядра";
    desc_recent_docs => "Recent documents list", "Список недавних документов";
    desc_old_downloads => "Old downloads",      "Старые загрузки";
    desc_sys_logs    => "System logs",          "Системные логи";
    desc_journal     => "systemd journal",      "Журнал systemd";
}
