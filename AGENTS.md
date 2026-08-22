# AGENTS.md — Mono Launcher

Desktop Minecraft modpack launcher. **Tauri 2** (Rust backend) + **Nuxt 3 / Vue 3 + TypeScript + Tailwind** (Vite SPA). Pack files distributed via `mono-launcher-storage` (separate host).

## Commands
| Command | Description |
|---------|-------------|
| `npm run dev` | Nuxt dev server on **port 1420** (SSR off) |
| `npm run tauri dev` | Dev app (Nuxt + Rust). Vite devUrl at `localhost:1420` |
| `npm run build` | **Typecheck + build** (`nuxt typecheck && nuxt build`). Output to `dist/` (what Tauri bundles). **CI test** (workflow invokes this to validate types) |
| `npm run tauri build` | Release binary; runs `npm run build` first via `beforeBuildCommand` |
| `npm run tauri:appimage` | Linux AppImage; requires `NO_STRIP=true` (set in script) |
| `cargo check` / `cargo clippy` | Rust checks from `src-tauri/` |
| `cargo test --lib` | Unit tests (mrpack, curseforge, modrinth, skins, ping, nbt, license, jre, files, lib) — **run before committing Rust** |
| `scripts/dev-headless.sh` | Headless UI smoke (Xvfb `:99`, software GL + WebKit workarounds) |

## Critical Gotchas
- **Build race**: running `tauri dev` watcher wipes `dist/` — never run `npm run build` while dev is alive
- **SSR off** + Nitro `preset: static` → SPA talking to Rust at runtime. Browser-only APIs (`window`, `navigator.clipboard`) used in `useLauncher.ts`; guard Tauri IPC with `isTauri()`
- **Port 1420 fixed** — Tauri `devUrl` depends on it (`vite.server.strictPort`)
- **Typecheck quirk**: Nuxt's generated `tsconfig.json` includes `../**/*` pulling `src-tauri/target`; root `tsconfig.json` overrides `exclude` — extend if adding large dirs. Strictness comes from `typescript.strict: true` in `nuxt.config.ts`; `nuxt typecheck` (vue-tsc) is what gates `npm run build`
- **Signing required**: `bundle.createUpdaterArtifacts: true` → build fails without minisign key. Local: `export TAURI_SIGNING_PRIVATE_KEY="$(cat ~/.tauri/mono-launcher.key)"` + `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
- **Single-instance before deep-link**: in `lib.rs` `run()`, `tauri-plugin-single-instance` must register before `tauri-plugin-deep-link`

## Layout
- `pages/index.vue` — entire UI (single page, ~8800 lines). `composables/useLauncher.ts` = all state + handlers (~2840 lines)
- `lib/bridge.ts` — typed Tauri IPC wrapper (`invoke`/`listen`). **Add new commands here**
- `lib/types.ts` — shared TypeScript types
- `lib/{misc,labels,format,changelog,iconCache}.ts` — pure UI helpers (extracted from `useLauncher.ts`). Put new reusable, side-effect-free helpers here, not in the composable
- `locales/` = `en.json`, `ru.json`, `uk.json`
- `composables/useI18n.ts` — **dynamic locales**: all `locales/*.json` loaded via `import.meta.glob` — drop a new JSON, language appears in UI automatically. Flat keys via `t()`. Each file has `"__meta__": {"author", "version"}` (translator + launcher version the translation targets) shown in the launcher settings. **Add every new key to EVERY locale JSON in `locales/` (incl. `__meta__` presence check)**
- `assets/css/main.css` — Tailwind entry + **theme CSS variables** (`--bg`, `--panel`, `--input`, `--border`, `--tx*`, `--accent*`, alpha variants). **Never hardcode colors**; theme toggled via `.theme-light` on `documentElement`, persisted in `localStorage.mono.theme`
- `src-tauri/src/` — Rust backend. `lib.rs` registers all `#[tauri::command]` in `run()`. Modules: `config.rs`, `mrpack.rs`, `auth.rs`, `game.rs`, `license.rs`, `curseforge.rs`, `modrinth.rs`, `skins.rs`, `files.rs`, `jre.rs`, `nbt.rs`, `ping.rs`, `discord_rp.rs`, `export.rs`

## Pack System
- **User packs only** — stored in `packs.json` in launcher root. No built-in packs (removed). `config::all_packs()` / `find_pack()` read only user packs
- `PackDescriptor`: `id`, `name`, `url` (direct `.mrpack` URL), `builtin` (always false), `kind` ("remote"/"local"), `author` (None), `boostyBlog`, `minRam`, `icon`, `banner`
- Every pack command takes `pack_id` (JS `packId`, nullable → default pack)
- **Adding packs**: direct `.mrpack` URLs only (no GitHub validation). URL deduplication in `add_pack_impl`

## Mono Backend Catalog
- `packCatalog()` → `GET /packs/` (no auth) — public catalog of all uploaded packs
- `packMine(token)` → `GET /packs/mine` (auth) — current user's packs
- `packDetail(token, id)` → `GET /packs/{id}` — full detail with versions[], news[], my_rating
- `packUpdate(token, id, body)` → `PUT /packs/{id}` — owner-only metadata update
- `packDelete(token, id)` → `DELETE /packs/{id}` — owner-only, deletes storage files too
- `packAddVersion(token, id, filePath, version, changelog)` → multipart upload of new .mrpack version
- `packDeleteVersion(token, id, versionId)` → owner-only
- `packAddNews(token, id, kind, title, body)` → add news to a pack
- `packDeleteNews(token, id, newsId)` → owner-only
- `packRate(token, id, value)` → like (1) or dislike (-1)
- `uploadPack(token, filePath, name, desc, ...)` → import/upload a new pack
- Screenshots: `POST /{id}/screenshots` (multipart → storage `/upload/image`), `DELETE /{id}/screenshots/{index}`

## Catalog Tab
The catalog tab has 4 source sub-tabs: `mono` (backend catalog), `author` (your packs), `modrinth`, `curseforge`. Clicking a mono catalog card opens a **detail view** (`catalogDetail`) with tabs: description, screenshots, versions, news. The detail view uses `packDetailCmd()` from the backend (accepts empty token for unauthenticated access).

## Servers Tab
- Two groups: pack servers (from `meta.servers` in backend) + player's `servers.dat` (parsed via `nbt.rs`: gzip+NBT, list-compounds)
- Ping via `ping_server_command` (Minecraft 1.7+ status, protocol 767, fallback legacy 0xFE)
- Refreshes every 45s while tab open (`serverPingTimer`)
- "Играть на сервере" → `launch_game_command(serverAddress)`

## Key Features
- **Playtime/Achievements**: `.mono-playtime.json` per version. `AppStatus` has `playtime_seconds`, `total_playtime_seconds`, `played_packs`
- **Multi-account**: `accounts.json` (`Accounts{active, list}`, `AccountEntry{id, username, uuid, access_token, user_type}`). Each login upserts + becomes active
- **Ely.by auth**: Device code flow (`ely_device_code_command`/`ely_poll_command`). Adds `-javaagent:authlib-injector.jar=ely.by` at launch
- **CurseForge**: API v1 needs `x-api-key` (file `<root>/curseforge-key.txt` / `MONO_CURSEFORGE_KEY`). Search opens in separate Tauri WebviewWindow (`search`, URL `?win=search&...`). Results sent to main via `mods-changed` event
- **Offline skins**: `skins-api/` (Cloudflare Worker). Local `skin.png`/`skin.json` + upload/delete. UUID = v3 DNS from lowercase nickname. Launch adds `-javaagent:...=<SKINS_API_URL>`
- **Min RAM**: `minRam` (MB) in pack config → clamped 256–65536. Gate at launch (`ram*1024 < minRam` blocks)
- **Bug reports**: Modal with env + last 60 lines of `launch.log` (`getLaunchLog`). Buttons: copy / open GitHub Issues
- **Boosty licenses**: Optional `boostyBlog` in pack config → paid. Player adds token in Settings; checked via `api.boosty.to` (user/me + subscriptions). `licenses.json` stores `{pack_id: {blog, token, expires_at, cached_until}}`. **Grace = 3 days** after success. Gates: `install_mrpack` + `launch_game_command` (call `licenses::ensure_license` always)
- **Deep links**: Scheme `mono` (`mono://add-pack?url=<direct-mrpack-url>&name=&blog=`). Handled via `deep-link://new-url` listener + argv scan at startup. URL-based dedup (no GitHub owner/repo parsing). UI notified via `pack-added` event
- **Custom mods / integrity**: Trusted hosts = Modrinth CDNs + `mediafiles.forgecdn.net`/`edge.forgecdn.net`. All files hash-verified (sha1/sha512). Untrusted + `overrides` jars → `.mono-custom.json` per version → `AppStatus.custom_mods` + warning banner (disable via `warn-custom-mods.txt`)
- **Modrinth search/tags**: `modrinth_search_command(query, kind, limit?, filters?)` with `SearchFilters{categories, versions, environment, index}`. `environment` maps to OR-groups `client_side:*`/`server_side:*` (NOT `environment:client`). `modrinth_tags_command(kind)` → loaders/categories/versions. Kinds beyond mods: `openModSearch(kind)` opens modal, `MOD_KIND_FOLDER` maps to install folder. Datapacks go to `saves/<world>/datapacks/`
- **News**: `get_news_command` returns empty vec (placeholder for backend `GET /packs/news` endpoint)
- **Mono auth**: `monoRegister`, `monoLogin`, `monoProfile`, `monoLogout` IPC commands. Profile stored in localStorage. Required for author panel, rate, upload
- **UI**: Each pack has its own nav tab (`openPackTab`); no sidebar `<select>`
- **Per-pack data**: `~/.local/share/MonoLauncher/packs/<pack_id>/` (versions/, active.json, mrpack-cache/); shared runtime in root

## CI / Release
- Workflow triggers on `launcher-v*` tags (`.github/workflows/build-launcher.yml`)
- Matrix: Windows (NSIS), macOS (DMG), Linux (AppImage, deb, rpm)
- Requires `TAURI_SIGNING_PRIVATE_KEY` + `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` + `MONO_CURSEFORGE_KEY` secrets
- `update-manifest` job runs `scripts/make-updater-json.mjs` to create `latest.json` from signed assets
- Release body updated from `CHANGELOG.md` via `scripts/release-notes.mjs`

## Adding Tauri Commands
1. Define in relevant Rust module (`src-tauri/src/*.rs`)
2. Register in `generate_handler!` in `src-tauri/src/lib.rs`
3. Add TypeScript wrapper in `lib/bridge.ts`
4. Update `lib/types.ts` if new types

## Testing
- **Rust**: `cargo test --lib` (only unit tests exist)
- **TypeScript**: `npm run build` (typecheck via `nuxt typecheck`)
- No TS test framework
- E2E deep-link (Linux): after `tauri build`, verify `xdg-mime query default x-scheme-handler/mono`, then `xdg-open "mono://add-pack?..."` under Xvfb, inspect `~/.local/share/MonoLauncher/packs.json`

## Versioning
- Launcher versioned via `launcher-v*` tags
- Pack `.mrpack` uploaded to mono backend (via upload flow), served by storage
