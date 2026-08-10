# NIO Launcher

Десктопный лаунчер для сборок Minecraft, распространяемых в формате `.mrpack` через GitHub Releases.

Отличное решение для авторов сборок и создателей модовых серверов: публикуете сборку на GitHub — и у игроков она появляется в один клик, с новостями, скриншотами, серверами, баг-репортами в ваши Issues и автоматическими обновлениями. Своя аудитория без магазинов и платформ в середине.

## Возможности
- **Несколько сборок** — каждая со своей вкладкой: релизы с ченджлогами, файлы (моды/ресурспаки/шейдеры/миры) с включением-выключением и поиском, консоль игры.
- **Контент из репозитория сборки**: вкладки «Скриншоты» (локальная папка игрока) и «Сервера» (`servers.json` + свои из `servers.dat`), звёзды и соцсети (`socials.json`) в шапке, статусы серверов, «Играть на сервере», новости сборки включаются/выключаются в один клик.
- **Новости**: релизы и дискуссии сборок + релизы самого лаунчера. Обновления лаунчера устанавливаются автоматически (tauri-plugin-updater, релиз подписан minisign).
- **Своя сборка за минуту**: кнопка «Создать сборку» открывает пример, ввод ссылки на репозиторий добавляет сборку в лаунчер (id = `owner-repo`). Ссылка-приглашение (`niol://add-pack?...` или `n1orio.github.io/nio-launcher/?url=...`) добавляет сборку в один клик.
- **Баг-репорты**: кнопка «Сообщить о баге» открывает форму Issues репозитория сборки с предзаполненным окружением (версия сборки, Minecraft, лаунчер, ОС).
- **Целостность файлов**: все файлы сверяются с sha1/sha512 из `modrinth.index.json`; файлы из недоверенных источников и `.jar` из `overrides` помечаются как кастомные и показываются в предупреждающем баннере.
- **Вход**: оффлайн-аккаунт или Microsoft (device code flow).
- Тёмная/светлая тема, русский/английский интерфейс, Discord RPC, playtime, автодетект Java 21+ и JRE-каталогов.

## Стек
- **Backend:** Rust + [Tauri 2.0](https://tauri.app)
- **Frontend:** Nuxt 3 (Vue 3 + TypeScript + Tailwind CSS), SSR выключен — SPA поверх IPC
- **Источник сборок:** `.mrpack` из GitHub Releases

## Структура
```
nio-launcher/
├── app.vue / nuxt.config.ts / package.json / tsconfig.json
├── pages/index.vue          # единственная страница (весь UI)
├── composables/useLauncher.ts  # состояние + логика, useI18n.ts — ru/en-словари
├── lib/                     # bridge.ts (типизированный Tauri IPC) + types.ts
├── assets/css/main.css      # глобальный CSS, Tailwind, переменные темы (var(--*))
├── scripts/
│   ├── dev-headless.sh       # запуск в headless (Xvfb) без монитора
│   └── make-updater-json.mjs # сборка latest.json для автообновления
└── src-tauri/                # Rust-бэкенд
    ├── Cargo.toml / tauri.conf.json
    └── src/
        ├── main.rs / lib.rs  # команды Tauri, deep links (niol://), новости, кэш API
        ├── config.rs         # PACKS (встроенные сборки) + пути
        ├── mrpack.rs         # .mrpack, modrinth.index.json, кастомные моды, версии
        ├── auth.rs           # оффлайн + Microsoft OAuth2
        ├── game.rs           # профили запуска (NeoForge/Forge/Fabric/Quilt), запуск Java
        ├── jre.rs            # поиск/выбор Java
        ├── files.rs          # файлы версии (моды/ресурспаки/…)
        ├── discord_rp.rs     # Discord Rich Presence
        └── lib.rs            # регистрация всех команд в run()
```

## Быстрый старт
```bash
npm install
npm run tauri dev      # запуск в dev-режиме (vite на :1420 и Rust)
npm run tauri build    # сборка бинарника (nuxt build → dist/ → таури)
npm run tauri:appimage # Linux AppImage (с NO_STRIP=true)
```

## Как распространяется сборка
1. Сборка собирается в `.mrpack` в Prism Launcher и выкладывается на GitHub Releases
   (тег релиза = версия сборки). В релизе также `pack.json` — название и описание.
2. Встроенные сборки — в `src-tauri/src/config.rs` (`PACKS`); любые другие добавляются
   через «Добавить сборку» (валидация: релизы репозитория содержат `.mrpack`).
3. Ченджлог релиза показывается из заметок GitHub.

## Хранение данных
```
~/.local/share/NioLauncher/
├── packs/<pack_id>/                # данные отдельной сборки
│   ├── versions/<versionId>/       # игровой профиль конкретной версии
│   │   ├── .nio-installed.json     # маркер установки (versionId, name, sourceTag)
│   │   ├── .nio-index.json         # копия modrinth.index.json
│   │   ├── .nio-custom.json        # кастомные моды (недоверенные источники / overrides)
│   │   └── mods/ config/ overrides # содержимое сборки
│   ├── active.json                 # активная версия
│   └── mrpack-cache/               # скачанный .mrpack
├── libraries/                      # библиотеки Minecraft/модлоадера
└── runtime/                        # встроенная Java (опционально)
```
Пользовательские сборки также регистрируются в `packs.json` в корне лаунчера.

## Tauri-команды (основные)
| Команда | Описание |
|---------|----------|
| `list_packs` / `add_pack_command` / `remove_pack_command` | Список / добавить / удалить сборку |
| `check_for_updates(packId?)` | Проверка новой версии `.mrpack` на GitHub |
| `install_mrpack(packId?, tag?)` | Скачивание версии, распаковка, установка модов + `overrides`, проверка хэшей, прогресс через `download-progress` |
| `list_versions(packId?)` | Релизы GitHub (тег, дата, ченджлог) + установленные версии + активная |
| `switch_version(packId?, versionId)` | Переключение активной версии |
| `get_status(packId?)` | Состояние активной версии, сессия, RAM, кастомные моды |
| `get_news` | Новости: релизы/дискуссии сборок + релизы лаунчера |
| `pack_repo_content(packId?)` | Звёзды, сервера (`servers.json`), соцсети (`socials.json`) сборки |
| `list_screenshots(packId?)` / `list_servers(packId?)` | Скриншоты папки `screenshots` и сервера `servers.dat` установленной версии |
| `ping_server(address, port?)` | Статус Minecraft-сервера (онлайн/игроки/версия/пинг) |
| `list_game_files_*` / `toggle_game_file` | Файлы версии: список, иконка, включение-выключение |
| `login_offline_command` / `ms_device_code` / `ms_poll` | Оффлайн-логин / Microsoft OAuth2 |
| `launch_game_command(packId?, ram, session)` | Запуск Java с указанным ОЗУ |
| `list_java` / `ensure_java` | Список найденных Java / установка JRE |
| `system_info` | Системная/доступная память (для рекомендуемой RAM) |
| `open_external` / `open_game_folder` / `get_skin` | Открыть URL/папку, скин Mojang |

## Как работает установка `.mrpack`
1. Скачивается `modpack.mrpack` с GitHub Releases (выбранный тег).
2. Архив распаковывается во временную папку.
3. Читается `modrinth.index.json`: версия Minecraft, модлоадер, массив `files`.
4. Все файлы из `files` скачиваются параллельно (`tokio` + `reqwest`, лимит 8 соединений)
   с проверкой SHA-1/SHA-512; несовпадение удаляет файл и останавливает установку.
5. `overrides/` копируется в профиль версии; пишется маркер установки и индекс; версия становится активной.

## Безопасность
- Моды считаются доверенными только с CDN Modrinth/CurseForge (`cdn.modrinth.com`,
  `dl.modrinth.com`, `mediafiles.forgecdn.net`).
- Остальные файлы и `.jar` из `overrides` — «кастомные»: записываются в `.nio-custom.json`
  версии и показываются в баннере предупреждения. Установка не блокируется (выбор автора),
  но источник подсвечивается.

## Запуск игры
- NeoForge / Forge / Fabric / Quilt (профиль запуска из установщика модлоадера).
- Java 21+; автопоиск в PATH и типовых каталогах (`jre.rs`) или ручной выбор.
- Wayland при пустом/белом окне: `WEBKIT_DISABLE_COMPOSITING_MODE=1` и
  `WEBKIT_DISABLE_DMABUF_RENDERER=1` выставляются автоматически.
- Headless-отладка UI: `scripts/dev-headless.sh`.