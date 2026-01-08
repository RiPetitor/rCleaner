# rCleaner

TUI system cleaner for Linux with support for Atomic and Desktop distributions.

Language: [English](#english) | [Русский](#русский)

## English

### What it is
rCleaner is a terminal UI application for scanning and cleaning common system and user clutter.
It is built to support both Atomic (rpm-ostree) and classic Desktop distributions.

### Features
- TUI with 6 cleanup tabs: Cache, Apps, Temp, Logs, Old Packages, Old Kernels
- Safety rules: whitelist/blacklist, root-only disable, protected system paths
- Modular cleaners and package-manager abstraction
- Configurable profiles (safe/aggressive)
- Dry-run hooks in interfaces (implementation in progress)

### Status
Early development. Some modules are stubs and behavior may change.

### Requirements
- Linux
- Rust stable (edition 2024)
- A real terminal (TTY) for the TUI

### Quick start
```bash
cargo run
```

If you run without a TTY (for example, from a non-interactive shell), rCleaner exits with a clear
message instead of crashing.

### Keybindings
| Key | Action |
| --- | --- |
| Q | Quit |
| R | Refresh |
| Tab / Shift+Tab | Next / previous tab |
| Up / Down | Navigate items |
| Space | Toggle selection |
| Enter | Start cleanup |

### Configuration
Config file path:
`~/.config/rcleaner/config.toml`

If the file is missing, defaults are used (the file is not auto-created yet).

Example:
```toml
[safety]
enabled = true
only_root_can_disable = true
level = "safe"

[profiles.safe]
auto_confirm = false
keep_recent_kernels = 2
keep_recent_deployments = 2
max_backup_size_gb = 10

[profiles.aggressive]
auto_confirm = true
keep_recent_kernels = 1
keep_recent_deployments = 1
max_backup_size_gb = 5

[rules.whitelist]
paths = ["~/.config", "~/Documents", "~/Projects"]

[rules.blacklist]
patterns = ["*.tmp", "*.log"]
```

### Safety notes
rCleaner is designed to avoid dangerous paths by default, but cleaning is still a destructive
operation. Review selections carefully and test in a safe environment.

### Development
```bash
cargo check
cargo test
```

## Русский

### Что это
rCleaner — терминальное (TUI) приложение для поиска и очистки мусора в системе и пользовательских
каталогах. Поддерживаются Atomic (rpm-ostree) и классические Desktop-дистрибутивы.

### Возможности
- TUI с 6 вкладками: Кэш, Приложения, Временные файлы, Журналы, Старые пакеты, Старые ядра
- Правила безопасности: whitelist/blacklist, отключение только для root, защита системных путей
- Модульные очистители и абстракция пакетных менеджеров
- Профили безопасности (safe/aggressive)
- Dry-run в интерфейсах (реализация в процессе)

### Статус
Ранняя стадия разработки. Некоторые модули являются заглушками, поведение может меняться.

### Требования
- Linux
- Rust stable (edition 2024)
- Реальный терминал (TTY) для TUI

### Быстрый старт
```bash
cargo run
```

Если запускать без TTY (например, из неинтерактивного окружения), rCleaner завершится с понятным
сообщением вместо паники.

### Горячие клавиши
| Клавиша | Действие |
| --- | --- |
| Q | Выход |
| R | Обновить |
| Tab / Shift+Tab | Следующая / предыдущая вкладка |
| Up / Down | Навигация по списку |
| Space | Выбор элемента |
| Enter | Запуск очистки |

### Конфигурация
Путь к конфигу:
`~/.config/rcleaner/config.toml`

Если файла нет, используются значения по умолчанию (автосоздание пока не реализовано).

Пример:
```toml
[safety]
enabled = true
only_root_can_disable = true
level = "safe"

[profiles.safe]
auto_confirm = false
keep_recent_kernels = 2
keep_recent_deployments = 2
max_backup_size_gb = 10

[profiles.aggressive]
auto_confirm = true
keep_recent_kernels = 1
keep_recent_deployments = 1
max_backup_size_gb = 5

[rules.whitelist]
paths = ["~/.config", "~/Documents", "~/Projects"]

[rules.blacklist]
patterns = ["*.tmp", "*.log"]
```

### Замечания по безопасности
rCleaner старается избегать критичных путей, но очистка — это разрушительная операция. Внимательно
проверяйте выбор и тестируйте в безопасной среде.

### Разработка
```bash
cargo check
cargo test
```
