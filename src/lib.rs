//! # rCleaner
//!
//! Терминальное приложение для очистки системного и пользовательского мусора на Linux.
//!
//! ## Возможности
//!
//! - Очистка кэша браузеров и приложений
//! - Удаление временных файлов
//! - Очистка журналов (logs)
//! - Удаление старых пакетов и ядер
//! - Поддержка Flatpak и Snap
//! - Поддержка rpm-ostree (Atomic Desktop)
//!
//! ## Модули
//!
//! - [`config`] - управление конфигурацией
//! - [`cleaner`] - модули очистки
//! - [`safety`] - правила безопасности
//! - [`system`] - определение системы и пакетные менеджеры
//! - [`tui`] - терминальный интерфейс
//! - [`backup`] - резервное копирование

pub mod config;
pub mod error;

pub use error::{RcleanerError, Result};

pub mod backup;
pub mod cleaner;
pub mod models;
pub mod safety;
pub mod system;
pub mod tui;
pub mod utils;

/// Версия приложения.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Название приложения.
pub const NAME: &str = "rCleaner";
