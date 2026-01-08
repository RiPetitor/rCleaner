//! Терминальный интерфейс (TUI) для rCleaner.
//!
//! Использует Flux-архитектуру:
//! - [`Action`] - действия пользователя
//! - [`State`] - состояние приложения
//! - [`Store`] - хранилище состояния
//! - [`Dispatcher`] - диспетчер действий
//! - [`App`] - главное приложение

pub mod action;
pub mod app;
pub mod dispatcher;
pub mod screens;
pub mod state;
pub mod store;
pub mod widgets;

pub use action::Action;
pub use app::App;
pub use dispatcher::Dispatcher;
pub use state::State;
pub use store::Store;
