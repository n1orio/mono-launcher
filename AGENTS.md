# AGENTS.md

Desktop Minecraft modpack launcher. **Tauri 2** (Rust backend) + **Nuxt 3 / Vue 3 + TypeScript + Tailwind** (Vite-based frontend). Modpack is distributed as a `.mrpack` hosted on GitHub Releases.

## Commands
- `npm run dev` — Nuxt dev server on port **1420** (SSR is off — this is a SPA inside Tauri).
- `npm run tauri dev` — dev app (starts Nuxt dev then Rust). Vite devUrl served at `localhost:1420`.
- `npm run build` — `nuxt typecheck && nuxt build`. This is the **typecheck/CI test**. Output (SSG/static SPA) lands in `dist/`, which is what Tauri bundles.
- `npm run tauri build` — release binary; runs `npm run build` first (via `beforeBuildCommand`).
- `npm run tauri:appimage` — Linux AppImage; requires `NO_STRIP=true` (already set in script). AppImages fail to strip properly on many toolchains.
- Local Rust checks: `cargo clippy` / `cargo check` from `src-tauri/`.

There is **no test framework**; the only automated check is the TypeScript typecheck through `npm run build` (`nuxt typecheck`, powered by `vue-tsc`). For Rust, run `cargo check` in `src-tauri/`.

## Layout
- `pages/index.vue` — the whole UI (the only page). `composables/useLauncher.ts` holds all state + handlers (the logic ported from the old `App.tsx`).
- `lib/` — `bridge.ts` (typed wrapper over `@tauri-apps/api` `invoke`/`listen`) and `types.ts`. **Add new Tauri commands here.**
- `app.vue` — root shell (just `<NuxtPage/>`). `assets/css/main.css` — global CSS + original Tailwind entry.
- `src-tauri/src/` — Rust backend. `lib.rs` registers all `#[tauri::command]`s in `run()`. Modules: `config.rs` (`PACKS` list + paths), `mrpack.rs` (download/install), `auth.rs` (offline + MS OAuth), `game.rs` (Java launch).
- `scripts/dev-headless.sh` — run UI headless under Xvfb (no monitor).

## Gotchas
- This is `ssr: false` + Nitro `preset: static` producing `dist/`. It is a desktop SPA talking to the Rust backend at runtime: **browser-only APIs** (`window`, `navigator.clipboard`, etc.) are used in `composables/useLauncher.ts`; guard all Tauri IPC calls with `isTauri()`.
- Dev/typecheck quirks: Nuxt's generated `tsconfig.json` (`include: ../**/*`) will pull in `src-tauri/target` binary JS. The root `tsconfig.json` overrides `exclude` to keep typecheck clean — if you touch build artifacts or add large dirs, extend that exclude list.
- `PACKS` in `src-tauri/src/config.rs` is the single source of truth for packs (id + name + `.mrpack` URL). Every pack-related command takes `pack_id` (JS camelCase `packId`, nullable → default pack). GitHub endpoints are derived from a pack's URL; `parse_github_repo` returns `None` for `USER`/`REPO` placeholders (skips GitHub API).
- Per-pack data lives in `~/.local/share/NioLauncher/packs/<pack_id>/` (versions/, active.json, mrpack-cache/); shared runtime data (libraries, assets, launch.log) is in the root — see README.
- Release changelogs (tag + body + published_at) come from the GitHub `releases` API — see `fetch_releases` / `GhVersion` in `lib.rs`.
- When you add/rename a Rust command, update `lib/bridge.ts` AND the `generate_handler!` list in `src-tauri/src/lib.rs`.
- Tauri 2 IPC commands are invoked camelCase on the JS/Vue side → snake_case in Rust (`launch_game_command`).
- Port 1420 must stay fixed — Tauri devUrl depends on it (`vite.server.strictPort`).
- Don't rely on git here — this directory has `.github/` but is not currently a git repo.
- Version the launcher with `launcher-v*` tags (`.github/workflows/build-launcher.yml`). The pack `.mrpack` is built manually in Prism Launcher and uploaded to GitHub Releases by hand.