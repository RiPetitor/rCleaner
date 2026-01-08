//! Trait и утилиты для работы с пакетными менеджерами.

use crate::error::{RcleanerError, Result};
use std::process::{Command, ExitStatus};

/// Trait для пакетных менеджеров.
///
/// Реализуется для rpm, dnf, apt, pacman, flatpak, snap и rpm-ostree.
pub trait PackageManager {
    /// Возвращает имя пакетного менеджера.
    fn name(&self) -> &str;

    /// Возвращает версию пакетного менеджера.
    fn version(&self) -> Result<String>;

    /// Возвращает список установленных пакетов.
    fn list_installed(&self) -> Result<Vec<String>>;

    /// Проверяет зависимости пакета.
    ///
    /// Возвращает список пакетов, которые зависят от указанного.
    fn check_dependencies(&self, package: &str) -> Result<Vec<String>>;

    /// Удаляет пакеты.
    ///
    /// # Arguments
    ///
    /// * `packages` - список пакетов для удаления
    /// * `dry_run` - если `true`, только симуляция
    fn remove_packages(&self, packages: &[String], dry_run: bool) -> Result<()>;
}

/// Результат выполнения команды.
pub(crate) struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: ExitStatus,
}

/// Выполняет команду и возвращает результат.
pub(crate) fn run_command(program: &str, args: &[&str]) -> Result<CommandOutput> {
    let output = Command::new(program).args(args).output()?;
    Ok(CommandOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        status: output.status,
    })
}

/// Создаёт ошибку для неуспешной команды.
pub(crate) fn command_failed(program: &str, output: &CommandOutput) -> RcleanerError {
    let message = if output.stderr.trim().is_empty() {
        output.stdout.trim().to_string()
    } else {
        output.stderr.trim().to_string()
    };
    RcleanerError::Command(format!("{program} command failed: {message}"))
}
