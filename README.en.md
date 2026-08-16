# Mono Launcher

_[English](README.en.md) · [Русский](README.md) · [Українська](README.uk.md) · [Deutsch](README.de.md) · [Беларуская](README.be.md)_

A desktop launcher for Minecraft modpacks distributed as `.mrpack` via GitHub Releases. Modrinth and CurseForge resources are also supported.

The perfect tool for modpack authors and modded-server creators: publish your pack on GitHub and players get it in one click — with news, screenshots, servers, bug reports to your Issues and automatic updates. Your own audience without stores or platforms in the middle.

## Why publish your pack in this launcher
- **One link — the pack and the launcher.** The universal invite link does everything for the player: the launcher is already installed — the pack is added and downloaded right away; no launcher — the bridge page offers "Download Mono Launcher", and after installation the pack is added just as automatically. No "download the mod, put it in a folder, configure it".
- **Social networks and servers right in the launcher.** `socials.json` puts your Discord/Telegram buttons into the pack header, `servers.json` — the official servers with online status and a one-click "Play on server" button.
- **A color scheme under your brand.** `theme.json` smoothly repaints the whole launcher UI in the pack's colors — the player sees your style from the first screen.
- **News and updates in the player's feed.** Pack releases and Discussion posts land directly in the launcher's news feed; the player sees a new version right away and updates in one click.
- **Bug reports with the environment pre-filled.** The "Report a bug" button opens Issues with the environment filled in (pack version, Minecraft, launcher, OS) — fewer clarifying questions, faster fixes.
- **Trust and integrity.** All files are verified against hashes from `modrinth.index.json`; installing mods from Modrinth/CurseForge is built in; playtime stats and Discord RPC included.
- **Public with no middlemen.** Publishing happens through your GitHub Releases — no stores or platforms in the middle.

## Features
- **Multiple modpacks** — each with its own tab: releases with changelogs, files (mods/resource packs/shaders/worlds) with enable-disable and search, and the game console.
- **Content from the pack repository**: "Screenshots" tab (the player's local folder) and "Servers" tab (`servers.json` + your own from `servers.dat`), stars and social links (`socials.json`) in the header — as colored buttons with the author's color (`color` in `socials.json`), server statuses with online counts ("12/100" + player names in the tooltip), "Play on server", the pack banner (`banner.png`), and a launcher theme matching the pack style (`theme.json` — smoothly repaints the UI when opened).
- **News**: pack releases and discussions + the launcher's own releases. Launcher updates install automatically (tauri-plugin-updater, release signed with minisign).
- **Your own pack in a minute**: the "Create pack" button opens an example; pasting a repository link adds the pack to the launcher (id = `owner-repo`). An invite link (`mono://add-pack?...` or `n1orio.github.io/mono-launcher/?url=...`) adds the pack in one click.
- **Paid packs via Boosty**: if the pack has `boostyBlog` set, the player links their Boosty token — a subscription to the publisher's blog unlocks downloading and launching (checked via the Boosty API, no backend of your own, with an offline grace period of up to 3 days).
- **Pack catalog**: the "Catalog" tab — a curated list of packs with descriptions and tags (`catalog.json` in the launcher repository), one-click install from the catalog; authors propose packs via an issue with a template.
- **Minimum RAM**: `minRam` in the publisher's `pack.json` (MB) — a badge on the pack, a hint in settings and protection against launching when RAM is insufficient.
- **Bug reports**: the "Report a bug" button opens a form targeting the pack repository's Issues, pre-filled with the environment (pack version, Minecraft, launcher, OS).
- **File integrity**: all files are verified against sha1/sha512 from `modrinth.index.json`; files from untrusted sources and `.jar` from `overrides` are marked as custom and shown in a warning banner.
- **Login**: offline account or Microsoft (device code flow). **Multiple accounts**: a list of saved profiles in Settings with one-click switching (every login goes into the list and becomes active).
- **Statistics**: an "N h" pill in the pack header (playtime); for a freshly installed pack — "Not played yet" until the first launch.
- Dark/light theme, Russian/English interface, Discord RPC, playtime, automatic detection of Java 21+ and JRE directories.

## Tech stack
- **Backend:** Rust + [Tauri 2.0](https://tauri.app)
- **Frontend:** Nuxt 3 (Vue 3 + TypeScript + Tailwind CSS), SSR disabled — an SPA over IPC
- **Pack source:** `.mrpack` from GitHub Releases

## Structure
```
mono-launcher/
├── app.vue / nuxt.config.ts / package.json / tsconfig.json
├── pages/index.vue          # the single page (all UI)
├── composables/useLauncher.ts  # state + logic, useI18n.ts — dynamic locales
├── catalog.json             # pack catalog ("Catalog" tab in the launcher)
├── lib/                     # bridge.ts (typed Tauri IPC) + types.ts
├── assets/css/main.css      # global CSS, Tailwind, theme variables (var(--*))
├── scripts/
│   ├── dev-headless.sh       # run headless (Xvfb) without a monitor
│   └── make-updater-json.mjs # build latest.json for auto-update
└── src-tauri/                # Rust backend
    ├── Cargo.toml / tauri.conf.json
    └── src/
        ├── main.rs / lib.rs  # Tauri commands, deep links (mono://), news, API cache
        ├── config.rs         # PACKS (built-in packs) + paths
        ├── mrpack.rs         # .mrpack, modrinth.index.json, custom mods, versions
        ├── auth.rs           # offline + Microsoft OAuth2
        ├── license.rs        # paid packs: Boosty subscription (player token)
        ├── game.rs           # launch profiles (NeoForge/Forge/Fabric/Quilt), Java launch
        ├── jre.rs            # Java discovery/selection
        ├── files.rs          # version files (mods/resource packs/...)
        ├── discord_rp.rs     # Discord Rich Presence
        └── lib.rs            # registers all commands in run()
```

## Quick start
```bash
npm install
npm run tauri dev      # run in dev mode (vite on :1420 and Rust)
npm run tauri build    # build the binary (nuxt build → dist/ → tauri)
npm run tauri:appimage # Linux AppImage (with NO_STRIP=true)
```

## For pack authors: publishing and distribution

### Step 1. Build the pack as `.mrpack`
In Prism Launcher: right-click the instance → **Export** → **Modrinth Pack (.mrpack)**.
You get a `modpack.mrpack` file — a zip with `modrinth.index.json` (Minecraft version, mod loader, file list with hashes). The file name can be anything with the `.mrpack` extension, but we recommend `modpack.mrpack`.

### Step 2. Create a public GitHub repository
The launcher calls the GitHub API without authorization — the repository **must be public**.
The pack id `owner-repo` comes from the repository name.

### Step 3. Publish a release with two files
GitHub → **Releases** → **Create a new release**. Tag = pack version (e.g. `1.0.0`); release notes become the changelog in the launcher. Attach assets to the release:

| File | Required | Purpose |
|------|:--------:|---------|
| `modpack.mrpack` | yes | the pack itself |
| `pack.json` | yes | metadata: name, paid status, minimum RAM |

⚠️ The release must be **published** (the "Publish release" button), not left as a draft — drafts are invisible to the launcher.

Example `pack.json`:
```json
{ "name": "My Pack", "boostyBlog": "my_blog", "minRam": 8192 }
```
- `name` — the pack name (without it the repository name is used).
- `boostyBlog` — the Boosty blog nickname: the pack becomes **paid** (see below); no field = free.
- `minRam` — minimum RAM in MB (valid range 256–65536): a "≥ N GB" badge on the pack and protection against launching when RAM is insufficient.
- snake_case variants are also accepted: `boosty_blog`, `min_ram`.

### Step 4. Test adding
In the launcher: the **"Add pack"** button → paste the repository URL (or a direct `.mrpack` link).
The launcher checks the releases: it finds the `.mrpack` and the `pack.json` beside it — if everything is correct, the pack appears in the list.

### Step 5. Distribute
- **Universal invite link** — the best way (details in the next section):
  a player follows the link — the pack is added to the launcher in one click.
- **Catalog**: the "Propose a pack" button (issue with a template) or a PR to `catalog.json` in this repository — the pack appears in the "Catalog" tab with a description, tags and an "Add" button.
- **Built-in packs** — by agreement with the launcher author: `PACKS` in `src-tauri/src/config.rs`.

### Universal invite link (for players and advertising)
One link does everything: for a player with the launcher installed it opens the launcher and adds the pack in one click; for a player without the launcher — a page with a "Download Mono Launcher" button.

**Link format** — two variants, both work the same:

```
mono://add-pack?url=<encoded>&name=<encoded>&blog=<encoded>
https://n1orio.github.io/mono-launcher/?url=<encoded>&name=<encoded>
```

**Parameters** (values percent-encoded, i.e. `encodeURIComponent`):

| Parameter | Required | What it passes |
|-----------|:--------:|----------------|
| `url` | yes | The pack repository link: `https://github.com/<owner>/<repo>` |
| `name` | no | The pack name (if absent — taken from `pack.json` or the repository name) |
| `blog` | no | Boosty blog nickname — the pack is added as **paid** (has priority over `pack.json`) |

**How to get a ready link (the easy way):**
add your pack to the launcher, open its tab and press
**"Copy invite link"** in the header — the launcher builds the link
with all parameters itself (including `blog=` for paid packs).

**How to build it by hand** (if you have no launcher at hand): encode the values and plug them into the template.
Example for the repository `https://github.com/n1orio/mono-pack-example`:

```
https://n1orio.github.io/mono-launcher/?url=https%3A%2F%2Fgithub.com%2Fn1orio%2Fmono-pack-example&name=My%20Pack&blog=my-blog
```

Here `https%3A%2F%2F...` is the encoded `https://...`, and `My%20Pack` is `My Pack` (space → `%20`).
The second variant (the `mono://` scheme) also accepts parameters, but it is better not to publish it on the web —
the browser will show an "unknown protocol" error: the web version (`https://n1orio.github.io/mono-launcher/?...`)
will itself try to open `mono://`, and if there is no launcher it will offer a download.

**What the player sees:**
1. Launcher installed → the app opens, the pack is validated
   (the repository must exist, and its **published** releases must contain
   `.mrpack` + `pack.json` — otherwise the player sees an error) and is added to the list.
2. Launcher not installed → a web bridge page opens with the pack name,
   an "Open in launcher" button and a "Download Mono Launcher" link.

**Where to publish the link:** the description and first comment of your release,
your repository README, Discord/Telegram channels, video description and Twitter —
the link is short and needs no explanation.

### Updates
Publish a new release with a new `modpack.mrpack` and `pack.json` — the launcher notices it itself and shows players an update button (the release tag = the new version, the changelog comes from the notes).

### Additional content (optional, in the repository root)
- `servers.json` — official pack servers ("Servers" tab): `[{ "name": "Name", "ip": "play.example.com", "port": 25565, "desc": "Description" }]`
- `socials.json` — colored social buttons in the header: object `{ "Discord": "https://…", "Telegram": "https://…" }` or an array; max 8, https only, each can have a `color: "#rrggbb"`.
- `banner.png` — the banner in the pack tab header.
- `theme.json` — the launcher repaints to match the pack style: `{ "bg": "#rrggbb", "panel": "…", "input": "…", "border": "…", "tx": "…", "txStrong": "…", "txMuted": "…", "accent": "…", "accentStrong": "…", "accentHover": "…", "accentDeep": "…" }` — all values are hex `#rrggbb`.

### Pre-publication checklist
- [ ] Repository is public
- [ ] Release is published (not a draft), tag = pack version
- [ ] The release contains `modpack.mrpack` and `pack.json`
- [ ] `pack.json` — valid JSON with the `name` field (recommended)
- [ ] The pack was added to the launcher via the repository link

## Pack catalog
The "Catalog" tab shows a curated list of packs from `catalog.json` in the root of this
repository (updated via raw.githubusercontent, without GitHub API quotas). A catalog entry:

```json
{
  "name": "My Pack",
  "url": "https://github.com/my-name/my-pack",
  "description": "A short description...",
  "author": "my-name",
  "tags": ["adventure", "magic"],
  "boostyBlog": "my_blog",
  "minRam": 8192
}
```

- `url` — a GitHub repository (or a direct `.mrpack` link), same as when adding by link.
- `boostyBlog` — if the pack is paid (see below), so the subscription is created right away.
- `minRam` — minimum RAM in MB.
- "Add" installs the pack in one click; for already added packs the button changes to "Open".
- Authors: you can propose a pack with the "Propose a pack" button (issue with a template)
  or via a PR to `catalog.json` in this repository.

## Paid packs via Boosty (without your own backend)
The subscription check runs on the player's side, so the publisher needs no server:

1. The player has their personal Boosty token: Boosty → Settings → **Apps** → "Create an app" (the token is issued immediately).
2. In the launcher (the subscription panel in the pack header) the player pastes the token — the launcher checks
   the subscription to your blog via `api.boosty.to` (`user/me` + `user/<id>/subscriptions`).
3. The binding is stored in `licenses.json` in the launcher root; after a successful check
   a **grace period of up to 3 days** applies — no network check needed at every launch.
4. Unsubscribing on Boosty → access is closed after the next check.
5. A pack from your `pack.json` and the invite link automatically carry
   `boostyBlog` to players — put it into `pack.json` before publishing (or specify it
   in the user's `packs.json`: `"boostyBlog": "my_blog"`).

Limitations: checking requires network; the player's Boosty token is their secret, the launcher stores
it only locally (`licenses.json`) and sends it nowhere except `api.boosty.to`.

## Data storage
```
~/.local/share/MonoLauncher/
├── packs/<pack_id>/                # data of an individual pack
│   ├── versions/<versionId>/       # game profile of a specific version
│   │   ├── .mono-installed.json     # install marker (versionId, name, sourceTag)
│   │   ├── .mono-index.json         # copy of modrinth.index.json
│   │   ├── .mono-custom.json        # custom mods (untrusted sources / overrides)
│   │   └── mods/ config/ overrides # pack contents
│   ├── active.json                 # active version
│   └── mrpack-cache/               # downloaded .mrpack
├── libraries/                      # Minecraft/mod-loader libraries
└── runtime/                        # bundled Java (optional)
```
User packs are also registered in `packs.json` in the launcher root.

## Tauri commands (main ones)
| Command | Description |
|---------|-------------|
| `list_packs` / `add_pack_command` / `remove_pack_command` | List / add / remove a pack |
| `check_for_updates(packId?)` | Check for a new `.mrpack` version on GitHub |
| `install_mrpack(packId?, tag?)` | Download the version, extract, install mods + `overrides`, verify hashes, progress via `download-progress` |
| `list_versions(packId?)` | GitHub releases (tag, date, changelog) + installed versions + active |
| `switch_version(packId?, versionId)` | Switch the active version |
| `get_status(packId?)` | Active version state, session, RAM, custom mods |
| `get_news` | News: pack releases/discussions + launcher releases |
| `pack_repo_content(packId?)` | Pack stars, servers (`servers.json`), socials (`socials.json`) |
| `fetch_catalog` | Pack catalog from `catalog.json` ("Catalog" tab) |
| `list_screenshots(packId?)` / `list_servers(packId?)` | Screenshots from the `screenshots` folder and servers from `servers.dat` of the installed version |
| `ping_server(address, port?)` | Minecraft server status (online/players/version/ping) |
| `list_game_files_*` / `toggle_game_file` | Version files: list, icon, enable-disable |
| `login_offline_command` / `ms_device_code` / `ms_poll` | Offline login / Microsoft OAuth2 |
| `launch_game_command(packId?, ram, session)` | Launch Java with the specified RAM |
| `list_java` / `ensure_java` | List found Java / install JRE |
| `set_boosty(packId, token)` / `license_status(packId)` / `clear_license(packId)` | Bind Boosty / subscription status / unbind |
| `system_info` | System/available memory (for the recommended RAM) |
| `open_external` / `open_game_folder` / `get_skin` | Open URL/folder, Mojang skin |

## How `.mrpack` installation works
1. `modpack.mrpack` is downloaded from GitHub Releases (the selected tag).
2. The archive is extracted into a temporary folder.
3. `modrinth.index.json` is read: Minecraft version, mod loader, `files` array.
4. All files from `files` are downloaded in parallel (`tokio` + `reqwest`, 8-connection limit)
   with SHA-1/SHA-512 verification; a mismatch deletes the file and stops the install.
5. `overrides/` is copied into the version profile; the install marker and index are written; the version becomes active.

## Security
- Mods are considered trusted only from the Modrinth/CurseForge CDNs (`cdn.modrinth.com`,
  `dl.modrinth.com`, `mediafiles.forgecdn.net`).
- Other files and `.jar` from `overrides` — "custom": written to `.mono-custom.json`
  of the version and shown in a warning banner. Installation is not blocked (the author's choice),
  but the source is highlighted.

## Launching the game
- NeoForge / Forge / Fabric / Quilt (launch profile from the mod-loader installer).
- Java 21+; auto-discovery in PATH and typical directories (`jre.rs`) or manual selection.
- Wayland with an empty/white window: `WEBKIT_DISABLE_COMPOSITING_MODE=1` and
  `WEBKIT_DISABLE_DMABUF_RENDERER=1` are set automatically.
- Headless UI debugging: `scripts/dev-headless.sh`.