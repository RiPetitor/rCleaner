# rCleaner

Safe, fast TUI system cleaner for Linux — built for Atomic and classic desktop distributions.

<h3 align="center">rCleaner - v1.0.0</h3>

<p align="center">
  <img src="https://github.com/RiPetitor/rCleaner/blob/master/rCleaner.png" alt="rCleaner" width="900">
</p>

Language: [English](#english) | [Русский](#русский)

## English

### What it is
rCleaner is a terminal UI system cleaner that removes clutter while keeping critical system areas protected.

### Key features
- 6 cleanup categories: Cache, Apps, Temp, Logs, Old Packages, Old Kernels
- Works on Atomic (rpm-ostree) and classic desktop distributions
- Package managers: APT, DNF, RPM, Pacman, Flatpak, Snap, rpm-ostree
- Safety-first rules with protected system paths, whitelist/blacklist, and root-only safety override
- Safe / Aggressive profiles for different cleanup styles
- Dry-run mode to preview changes
- Automatic backups before cleanup with SHA-256 checksums
- App sources: Flatpak, Snap, Docker, Podman

### What's new in v1.0
- **i18n**: Russian / English interface switching, saved in config
- **Modern UI**: RGB color theme, rounded borders, badges, accent highlights
- **Mouse support**: click to select items, scroll wheel navigation
- **Vim-style navigation**: j/k keys, 1-6 tab switching
- **CLI arguments**: `--dry-run`, `--config <path>`, `--verbose` (via clap)
- **Parallel scanning**: all 6 cleaners run concurrently with rayon
- **Non-blocking cleanup**: runs in background thread, Esc to cancel
- **Settings redesign**: Tab between blocks, arrow keys to navigate, Enter to toggle
- **Improved scanners**: package sizes (APT/DNF/Pacman), pacman cache, /lib/modules kernel detection, core dumps, pip/npm/cargo caches, recently-used.xbel
- **38 tests** including 17 Store reducer tests

### Experience
- Focused TUI interface with tabs, search, and bulk selection
- Transparent size estimates and results after cleaning
- Dynamic page size adapts to terminal height

### Usage

```
rcleaner [OPTIONS]

Options:
  -n, --dry-run          Run in dry-run mode (no actual deletion)
  -c, --config <FILE>    Path to config file
  -v, --verbose          Enable verbose logging
  -h, --help             Print help
  -V, --version          Print version
```

### Hotkeys

| Key | Action |
|-----|--------|
| Tab / 1-6 | Switch category |
| j/k / Up/Down | Navigate items |
| Space | Toggle selection |
| A | Select / deselect all |
| Enter | Start cleanup |
| S | Open settings |
| R | Rescan |
| / | Search |
| Q | Quit |

---

## Русский

### Что это
rCleaner — TUI-очиститель для Linux, который убирает мусор и бережно относится к системе.

### Возможности
- 6 категорий очистки: Кэш, Приложения, Временные файлы, Логи, Старые пакеты, Старые ядра
- Поддержка Atomic (rpm-ostree) и классических desktop-дистрибутивов
- Пакетные менеджеры: APT, DNF, RPM, Pacman, Flatpak, Snap, rpm-ostree
- Безопасные правила: защита системных путей, whitelist/blacklist, переключатель безопасности только для root
- Профили Safe / Aggressive
- Dry-run для предварительного просмотра
- Автоматические бэкапы перед очисткой с контрольными суммами SHA-256
- Источники приложений: Flatpak, Snap, Docker, Podman

### Что нового в v1.0
- **i18n**: переключение интерфейса Русский / English, сохраняется в конфиге
- **Современный UI**: RGB-тема, скруглённые рамки, бейджи, акцентные цвета
- **Поддержка мыши**: клик для выбора, прокрутка колёсиком
- **Vim-навигация**: клавиши j/k, переключение вкладок 1-6
- **CLI аргументы**: `--dry-run`, `--config <path>`, `--verbose` (clap)
- **Параллельное сканирование**: все 6 сканеров работают одновременно (rayon)
- **Фоновая очистка**: запускается в отдельном потоке, Esc для отмены
- **Настройки**: навигация Tab между блоками, стрелки внутри, Enter для переключения
- **Улучшенные сканеры**: размеры пакетов (APT/DNF/Pacman), кэш pacman, обнаружение ядер через /lib/modules, дампы ядра, кэши pip/npm/cargo, recently-used.xbel
- **38 тестов**, включая 17 тестов Store reducer

### Интерфейс
- Вкладки, поиск и массовый выбор
- Прозрачная оценка объёма и итогов очистки
- Динамический размер страницы по высоте терминала

### Использование

```
rcleaner [ОПЦИИ]

Опции:
  -n, --dry-run          Запуск в режиме dry-run (без удаления)
  -c, --config <ФАЙЛ>    Путь к файлу конфигурации
  -v, --verbose          Подробное логирование
  -h, --help             Справка
  -V, --version          Версия
```

### Горячие клавиши

| Клавиша | Действие |
|---------|----------|
| Tab / 1-6 | Переключение категории |
| j/k / Up/Down | Навигация по элементам |
| Space | Выбрать / снять выбор |
| A | Выбрать / снять все |
| Enter | Начать очистку |
| S | Настройки |
| R | Пересканировать |
| / | Поиск |
| Q | Выход |
