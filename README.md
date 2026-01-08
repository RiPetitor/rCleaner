# rCleaner

Safe, fast TUI system cleaner for Linux — built for Atomic and classic desktop distributions.

Current version: 0.9.0

Language: [English](#english) | [Русский](#русский)

## English

### What it is
rCleaner is a terminal UI system cleaner that removes clutter while keeping critical system areas protected.

### Key features
- 6 cleanup categories: Cache, Apps, Temp, Logs, Old Packages, Old Kernels
- Works on Atomic (rpm-ostree) and classic desktop distributions
- Safety-first rules with protected system paths, whitelist/blacklist, and root-only safety override
- Safe / Aggressive profiles for different cleanup styles
- Dry-run mode to preview changes
- Automatic backups before cleanup
- App sources: Flatpak, Snap, Docker, Podman
- Fast scanning, clear summaries, and progress feedback

### Experience
- Focused TUI interface with tabs, search, and bulk selection
- Transparent size estimates and results after cleaning

### Preview
```
┌──────────────────────────────────────────────────────────────────────────┐
│ Safety: SAFE           rCleaner v0.9.0            Fedora 40 Atomic | KDE │
├──────────────────────────────────────────────────────────────────────────┤
│ [Cache] [Apps] [Temp] [Logs] [Packages] [Kernels]                        │
├──────────────────────────────────────────────────────────────────────────┤
│ [x] ~/.cache/mozilla/...                         1.2 GB                  │
│ [ ] ~/.cache/thumbnails                          450 MB                  │
│ [x] /var/cache/flatpak                           200 MB                  │
│ [ ] ~/.cache/Steam                               3.4 GB                  │
│                                                                          │
│ Selected: 1.85 GB (12%)      Total: 15.2 GB                              │
├──────────────────────────────────────────────────────────────────────────┤
│ [Tab] Next  [/] Search  [A] All  [Enter] Clean  [S] Settings  [Q] Quit   │
└──────────────────────────────────────────────────────────────────────────┘
```

## Русский

### Что это
rCleaner — TUI-очиститель для Linux, который убирает мусор и бережно относится к системе.

### Возможности
- 6 категорий очистки: Кэш, Приложения, Временные файлы, Логи, Старые пакеты, Старые ядра
- Поддержка Atomic (rpm-ostree) и классических desktop-дистрибутивов
- Безопасные правила: защита системных путей, whitelist/blacklist, переключатель безопасности только для root
- Профили Safe / Aggressive
- Dry-run для предварительного просмотра
- Автоматические бэкапы перед очисткой
- Источники приложений: Flatpak, Snap, Docker, Podman
- Быстрое сканирование, понятная статистика и прогресс

### Интерфейс
- Вкладки, поиск и массовый выбор
- Прозрачная оценка объёма и итогов очистки

### Превью
```
┌──────────────────────────────────────────────────────────────────────────┐
│ Safety: SAFE           rCleaner v0.9.0            Fedora 40 Atomic | KDE │
├──────────────────────────────────────────────────────────────────────────┤
│ [Cache] [Apps] [Temp] [Logs] [Packages] [Kernels]                        │
├──────────────────────────────────────────────────────────────────────────┤
│ [x] ~/.cache/mozilla/...                         1.2 GB                  │
│ [ ] ~/.cache/thumbnails                          450 MB                  │
│ [x] /var/cache/flatpak                           200 MB                  │
│ [ ] ~/.cache/Steam                               3.4 GB                  │
│                                                                          │
│ Selected: 1.85 GB (12%)      Total: 15.2 GB                              │
├──────────────────────────────────────────────────────────────────────────┤
│ [Tab] Next  [/] Search  [A] All  [Enter] Clean  [S] Settings  [Q] Quit   │
└──────────────────────────────────────────────────────────────────────────┘
```
