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
| `cargo test --lib` | Unit tests (10 modules: mrpack, curseforge, modrinth, skins, ping, nbt, license, jre, **files**, **lib**) — **run before committing Rust** |
| `scripts/dev-headless.sh` | Headless UI smoke (Xvfb `:99`, software GL + WebKit workarounds) |

## Critical Gotchas
- **Build race**: running `tauri dev` watcher wipes `dist/` — never run `npm run build` while dev is alive
- **SSR off** + Nitro `preset: static` → SPA talking to Rust at runtime. Browser-only APIs (`window`, `navigator.clipboard`) used in `useLauncher.ts`; guard Tauri IPC with `isTauri()`
- **Port 1420 fixed** — Tauri `devUrl` depends on it (`vite.server.strictPort`)
- **Typecheck quirk**: Nuxt's generated `tsconfig.json` includes `../**/*` pulling `src-tauri/target`; root `tsconfig.json` overrides `exclude` — extend if adding large dirs. Strictness comes from `typescript.strict: true` in `nuxt.config.ts`; `nuxt typecheck` (vue-tsc) is what gates `npm run build`
- **Signing required**: `bundle.createUpdaterArtifacts: true` → build fails without minisign key. Local: `export TAURI_SIGNING_PRIVATE_KEY="$(cat ~/.tauri/mono-launcher.key)"` + `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
- **Single-instance before deep-link**: in `lib.rs` `run()`, `tauri-plugin-single-instance` must register before `tauri-plugin-deep-link`

## Layout
- `pages/index.vue` — entire UI (single page). `composables/useLauncher.ts` = all state + handlers
- `lib/bridge.ts` — typed Tauri IPC wrapper (`invoke`/`listen`). **Add new commands here**
- `lib/types.ts` — shared TypeScript types
- `lib/{misc,labels,format,changelog,iconCache}.ts` — pure UI helpers (recently extracted from `useLauncher.ts`). Put new reusable, side-effect-free helpers here, not in the composable
- `locales/` = `en.json`, `ru.json`, `uk.json`
- `composables/useI18n.ts` — **dynamic locales**: all `locales/*.json` loaded via `import.meta.glob` — drop a new JSON, language appears in UI automatically. Flat keys via `t()`. Each file has `"__meta__": {"author", "version"}` (translator + launcher version the translation targets) shown in the launcher settings. **Add every new key to EVERY locale JSON in `locales/` (incl. `__meta__` presence check)**
- `assets/css/main.css` — Tailwind entry + **theme CSS variables** (`--bg`, `--panel`, `--input`, `--border`, `--tx*`, `--accent*`, alpha variants). **Never hardcode colors**; theme toggled via `.theme-light` on `documentElement`, persisted in `localStorage.mono.theme`
- `src-tauri/src/` — Rust backend. `lib.rs` registers all `#[tauri::command]` in `run()`. Modules: `config.rs`, `mrpack.rs`, `auth.rs`, `game.rs`, `license.rs`, `curseforge.rs`, `modrinth.rs`, `skins.rs`, `files.rs`, `jre.rs`, `nbt.rs`, `ping.rs`, `discord_rp.rs`

## Pack System
- **Built-in**: `PACKS` in `src-tauri/src/config.rs`
- **User-added**: `packs.json` in launcher root (id = `owner-repo` slug). `config::all_packs()` / `find_pack` merge both
- `PackDescriptor`: `id`, `name`, `url`, `builtin`, `kind` ("remote"/"local"), `author` (GitHub owner), `boostyBlog`, `minRam`, `icon`, `banner`
- Every pack command takes `pack_id` (JS `packId`, nullable → default)
- `parse_github_repo` returns `None` for `USER`/`REPO` placeholders (skips GitHub API)

## Repo Content (`pack_repo_content_command`)
- GitHub stars via API
- `servers.json` (`[{name, ip, port?, desc?}]`) + `socials.json` (object/array, max 8, https-only, optional `color` `#rrggbb`)
- `banner.png` + `icon.png` (fetched raw at HEAD from `raw.githubusercontent.com`, no API quota)
- `theme.json` (10 hex keys) → applied as inline CSS vars with 0.6s fade (`html.pack-theme-fade`)
- Cached 15 min in `ApiCache.meta`

## Servers Tab
- Two groups: pack servers (`servers.json`) + player's `servers.dat` (parsed via `nbt.rs`: gzip+NBT, list-compounds)
- Ping via `ping_server_command` (Minecraft 1.7+ status, protocol 767, fallback legacy 0xFE)
- Refreshes every 45s while tab open (`serverPingTimer`)
- "Играть на сервере" → `launch_game_command(serverAddress)`

## Key Features
- **Playtime/Achievements**: `.mono-playtime.json` per version. `AppStatus` has `playtime_seconds`, `total_playtime_seconds`, `played_packs`
- **Multi-account**: `accounts.json` (`Accounts{active, list}`, `AccountEntry{id, username, uuid, access_token, user_type}`). Each login upserts + becomes active
- **Ely.by auth**: Device code flow (`ely_device_code_command`/`ely_poll_command`). Adds `-javaagent:authlib-injector.jar=ely.by` at launch
- **CurseForge**: API v1 needs `x-api-key` (file `<root>/curseforge-key.txt` / `MONO_CURSEFORGE_KEY`). Search opens in separate Tauri WebviewWindow (`search`, URL `?win=search&...`). Results sent to main via `mods-changed` event
- **Offline skins**: `skins-api/` (Cloudflare Worker). Local `skin.png`/`skin.json` + upload/delete. UUID = v3 DNS from lowercase nickname. Launch adds `-javaagent:...=<SKINS_API_URL>`
- **Min RAM**: `minRam` (MB) in `pack.json`/`packs.json`/`PACKS` → clamped 256–65536. Gate at launch (`ram*1024 < minRam` blocks)
- **Catalog**: static `catalog.json` in repo root (raw.githubusercontent, 15min cache). Entry: `name`, `url`, `description`, `author`, `boostyBlog`, `minRam`, `tags`
- **Bug reports**: Modal with env + last 60 lines of `launch.log` (`getLaunchLog`). Buttons: copy / open GitHub Issues
- **Boosty licenses**: Optional `boostyBlog` in pack config → paid. Player adds token in Settings; checked via `api.boosty.to` (user/me + subscriptions). `licenses.json` stores `{pack_id: {blog, token, expires_at, cached_until}}`. **Grace = 3 days** after success. Gates: `install_mrpack` + `launch_game_command` (call `licenses::ensure_license` always)
- **Deep links**: Scheme `mono` (`mono://add-pack?url=&name=&blog=`). Handled via `deep-link://new-url` listener + argv scan at startup. Duplicates filtered via `HANDLED_LINKS`. UI notified via `pack-added` event
- **Universal invite**: `https://n1orio.github.io/mono-launcher/?url=&name=` (gh-pages branch). Auto-opens `mono://`, falls back to download CTA
- **Custom mods / integrity**: Trusted hosts = Modrinth CDNs + `mediafiles.forgecdn.net`/`edge.forgecdn.net`. All files hash-verified (sha1/sha512). Untrusted + `overrides` jars → `.mono-custom.json` per version → `AppStatus.custom_mods` + warning banner (disable via `warn-custom-mods.txt`)
- **Modrinth search/tags**: `modrinth_search_command(query, kind, limit?, filters?)` with `SearchFilters{categories, versions, environment, index}`. `environment` maps to OR-groups `client_side:*`/`server_side:*` (NOT `environment:client`). `modrinth_tags_command(kind)` → loaders/categories/versions. Kinds beyond mods: `openModSearch(kind)` opens modal, `MOD_KIND_FOLDER` maps to install folder. Datapacks go to `saves/<world>/datapacks/`
- **News**: Pack releases + discussions + launcher's own releases (`NEWS_REPO = n1orio/mono-launcher`, `pack_id: "launcher"`). Launcher news items show "open release page" button
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
- Pack `.mrpack` built manually in Prism Launcher, uploaded to GitHub Releases by hand