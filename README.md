# NIO Launcher

Фирменный моно-лаунчер для сборки Minecraft, распространяемой в формате `.mrpack` через GitHub Releases.

## Стек
- **Backend:** Rust + [Tauri 2.0](https://tauri.app)
- **Frontend:** Nuxt 3 (Vue 3 + TypeScript + Tailwind CSS)
- **Источник сборки:** `.mrpack` из GitHub Releases

## Структура
```
nio-launcher/
├── app.vue / nuxt.config.ts / package.json / tsconfig.json
├── pages/index.vue          # единственная страница (весь UI)
├── composables/useLauncher.ts  # состояние + логика (порт из App.tsx)
├── lib/                     # bridge.ts (Tauri IPC) + types.ts
├── assets/css/main.css      # глобальный CSS + Tailwind
├── scripts/
│   └── dev-headless.sh       # запуск в headless (Xvfb) без монитора
└── src-tauri/                # Rust-бэкенд
    ├── Cargo.toml
    └── src/
        ├── main.rs / lib.rs  # команды Tauri
        ├── config.rs         # PACKS (список сборок) + пути
        ├── mrpack.rs         # парсинг modrinth.index.json, скачивание модов, версии
        ├── auth.rs           # оффлайн + Microsoft OAuth2
        └── game.rs           # профили запуска (NeoForge/Forge/Fabric/Quilt), запуск Java
```

## Быстрый старт
```bash
npm install
npm run tauri dev      # запуск в dev-режиме
npm run tauri build    # сборка бинарника
npm run tauri:appimage # Linux AppImage (с NO_STRIP=true)
```

## Как распространяется сборка
1. Новая версия собирается в `.mrpack` вручную (Prism Launcher) и выкладывается на GitHub Releases
   с прикреплённым файлом `Untold.legends.mrpack`.
2. `versionId` в `modrinth.index.json` — версия сборки; в UI она выбирается по тегу релиза.
3. Ченджлог релиза показывается прямо из его заметок.

## Устройство хранения данных
```
~/.local/share/NioLauncher/
├── packs/<pack_id>/                # данные отдельной сборки
│   ├── versions/<versionId>/       # игровой профиль конкретной версии
│   │   ├── .nio-installed.json     # маркер установки (versionId, name, sourceTag)
│   │   ├── .nio-index.json         # копия modrinth.index.json
│   │   └── mods/ config/ overrides # содержимое сборки
│   ├── active.json                 # активная версия
│   └── mrpack-cache/               # скачанный .mrpack
├── libraries/                      # библиотеки Minecraft/модлоадера
└── runtime/                        # встроенная Java (опционально)
```

## Tauri-команды
| Команда | Описание |
|---------|----------|
| `list_packs` | Список поддерживаемых сборок |
| `check_for_updates(packId?)` | Проверка новой версии `.mrpack` на GitHub |
| `install_mrpack(packId?, tag?)` | Скачивание версии, распаковка, установка модов + `overrides`, прогресс через событие `download-progress` |
| `list_versions(packId?)` | Релизы GitHub (тег, дата, ченджлог) + установленные версии + активная |
| `switch_version(packId?, versionId)` | Переключение активной версии (по тегу или versionId) |
| `get_status(packId?)` | Состояние активной версии, текущая сессия, RAM |
| `login_offline_command` | Оффлайн-логин |
| `login_microsoft_command` | Вход через Microsoft OAuth2 (device code) |
| `launch_game_command(packId?, ram, session)` | Запуск Java с указанным ОЗУ |
| `system_info` | Системная/доступная память (нужно для рекомендуемой RAM) |

## Как работает установка `.mrpack`
1. Скачивается `modpack.mrpack` с GitHub Releases (для выбранного тега — по URL релиза).
2. Архив распаковывается во временную папку.
3. Читается `modrinth.index.json`: версия Minecraft, модлоадер, массив `files`.
4. Все файлы из `files` скачиваются параллельно (`tokio` + `reqwest`, лимит 8 соединений) с проверкой SHA-1/SHA-512.
5. Папка `overrides/` копируется в профиль версии; пишется маркер установки и индекс; версия становится активной.

## Запуск игры
- Поддерживаются NeoForge / Forge / Fabric / Quilt (профиль запуска берётся из установщика модлоадера).
- Нужна Java 21+; не встроена — используется `java` из PATH (или `~/NioLauncher/runtime/bin`).
- На Wayland при пустом/белом окне: `WEBKIT_DISABLE_COMPOSITING_MODE=1` и `WEBKIT_DISABLE_DMABUF_RENDERER=1` ставятся автоматически.
- Headless-отладка UI: `scripts/dev-headless.sh`.