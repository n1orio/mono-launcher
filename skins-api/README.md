# NIO Launcher — Skin API (Cloudflare Worker)

Публичный скин-сервис для оффлайн-игроков. Загруженный в лаунчере скин
становится доступен по нику и UUID — в одиночной игре и на серверах,
подключённых через `authlib-injector`.

## Деплой

0. Нужен аккаунт Cloudflare (бесплатный, без карты):

```bash
npx wrangler login
npx wrangler kv namespace create SKINS
# В выводе будет id — вставьте его в wrangler.toml (kv_namespaces[0].id)
npx wrangler deploy
```

Текущий адрес: `https://nio-skins.skins-api.workers.dev` (уже прописан в
`src-tauri/src/skins.rs` → `SKINS_API_URL`).

> **Внимание (РФ)**: `*.workers.dev` и `*.pages.dev` часто блокируются на
> уровне сети (SNI-фильтрация). Если игроки не могут достучаться до API —
> добавьте к worker'у свой домен (Cloudflare Dashboard → Worker →
> Settings → Domains & Routes) и обновите `SKINS_API_URL` на него.

Протестировать:

```bash
curl -X PUT https://nio-skins.xxx.workers.dev/skins/Steve \
  -H "X-Skin-Model: classic" --data-binary @skin.png
curl https://nio-skins.xxx.workers.dev/skins/Steve
```

## API

| Метод | Путь | Описание |
|---|---|---|
| `PUT` | `/skins/<ник>` | Загрузить скин (тело — PNG, заголовок `X-Skin-Model: classic\|slim`) |
| `GET` | `/skins/<ник>` | Вернуть PNG скина (или 204, если нет) |
| `DELETE` | `/skins/<ник>` | Удалить скин |
| `POST` | `/skins/<ник>/textures` | Вернуть JSON-пакет textures для этого ника (тот же, что в профиле) |
| `GET` | `/health` | Проверка |

### Yggdrasil / authlib-injector

- `GET /api/authlib-injector/checker` — метаданные API (для клиента)
- `POST /api/yggdrasil/authenticate` — оффлайн-вход: принимает любой пароль,
  UUID считается как v3-DNS от ника (как в лаунчере)
- `POST /api/yggdrasil/sessionserver/session/minecraft/profile/<uuid>?unsigned=false`
  — профиль с textures (для сервера)
- `POST /api/yggdrasil/sessionserver/session/minecraft/join` — 204
- `POST /api/yggdrasil/refresh` — 204 (сессии не храним)
- `POST /api/yggdrasil/validate` — 204

Сервер и клиент Minecraft общаются с этим сервисом только через
`authlib-injector` (jar от yushijinhun). Ничего другого не требуется.

## Для разработчиков серверов

Скины игроков работают на сервере, если он подключён к этому API через
`authlib-injector`.

1. Скачайте jar:

```bash
curl -LO https://github.com/yushijinhun/authlib-injector/releases/download/1.2.5/authlib-injector-1.2.5.jar
```

2. Добавьте в скрипт запуска сервера (перед `-jar`):

```
java -javaagent:authlib-injector-1.2.5.jar=<URL скин-API> -jar server.jar nogui
```

3. `online-mode=false` НЕ включайте отключение проверки: при использовании
   authlib-injector сервер должен работать в online-режиме, но проверять
   сессии он будет через наш API, а не через Mojang. Игроки лаунчера NIO
   (и любые игроки с тем же authlib-injector URL) будут заходить со своими
   скинами; остальные — получат скин Steve.

Ограничение: скин виден только тем серверам, которые используют тот же URL
скин-API. Это свойство любой централизованной схемы authlib-injector.

## Лимиты (бесплатный план Cloudflare)

- 100 000 запросов в день на аккаунт
- KV: 100 000 чтений / 1 000 записей в день, 1 GB данных
- CPU: 10 мс на запрос (наш ответ — доли мс: чтение KV + JSON)

На 1 запрос профиля Minecraft-сервер делает 1 чтение KV. 100k чтений в день
≈ 3k+ заходов игроков в день. При росте — платный план ($5/мес за 10 млн
запросов) или кэширование ответов на серверах.