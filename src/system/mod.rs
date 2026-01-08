//! Модули для работы с системой и пакетными менеджерами.
//!
//! - [`detection`] - определение типа системы
//! - [`package_manager`] - базовый trait для пакетных менеджеров
//! - [`rpm_ostree`] - поддержка rpm-ostree (Atomic Desktop)
//! - [`rpm`] - поддержка RPM
//! - [`dnf`] - поддержка DNF
//! - [`apt`] - поддержка APT
//! - [`pacman`] - поддержка Pacman
//! - [`flatpak`] - поддержка Flatpak
//! - [`snap`] - поддержка Snap

pub mod apt;
pub mod detection;
pub mod dnf;
pub mod flatpak;
pub mod package_manager;
pub mod pacman;
pub mod rpm;
pub mod rpm_ostree;
pub mod snap;
