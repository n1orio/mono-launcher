# Mono Launcher

_[Deutsch](README.de.md) · [English](README.en.md) · [Русский](README.md) · [Українська](README.uk.md) · [Беларуская](README.be.md)_

Ein Desktop-Launcher für Minecraft-Modpacks, die als `.mrpack` über GitHub Releases verbreitet werden. Ressourcen von Modrinth und CurseForge werden ebenfalls unterstützt.

Die ideale Lösung für Modpack-Autoren und Ersteller modifizierter Server: Veröffentlichen Sie Ihr Pack auf GitHub — und Spieler erhalten es mit einem Klick, inklusive News, Screenshots, Servern, Bug-Reporten in Ihre Issues und automatischen Updates. Ihre eigene Zielgruppe ohne Shops und Plattformen dazwischen.

## Warum Ihr Pack in diesem Launcher veröffentlichen
- **Ein Link — Pack und Launcher.** Der universelle Einladungslink macht alles für den Spieler: Launcher schon da — das Pack wird sofort hinzugefügt und heruntergeladen; kein Launcher — die Brücken-Seite bietet „Mono Launcher herunterladen“ an, und nach der Installation wird das Pack ebenso automatisch hinzugefügt. Kein „Mod herunterladen, in Ordner legen, konfigurieren“.
- **Soziale Netzwerke und Server direkt im Launcher.** `socials.json` setzt Ihre Discord/Telegram-Buttons in die Pack-Kopfzeile, `servers.json` — die offiziellen Server mit Online-Status und einer „Auf Server spielen“-Schaltfläche mit einem Klick.
- **Ein Farbschema unter Ihrer Marke.** `theme.json` färbt die gesamte Launcher-Oberfläche sanft in den Farben des Packs um — der Spieler sieht Ihren Stil ab dem ersten Bildschirm.
- **News und Updates im Feed des Spielers.** Pack-Releases und Discussion-Beiträge landen direkt im Nachrichten-Feed des Launchers; eine neue Version sieht der Spieler sofort und aktualisiert sie mit einem Klick.
- **Bug-Reports mit ausgefüllter Umgebung.** Die Schaltfläche „Bug melden“ öffnet Issues mit ausgefüllter Umgebung (Pack-Version, Minecraft, Launcher, OS) — weniger Rückfragen, schnellere Fixes.
- **Vertrauen und Integrität.** Alle Dateien werden gegen Hashes aus `modrinth.index.json` geprüft; die Installation von Mods von Modrinth/CurseForge ist integriert; Spielzeit-Statistik und Discord RPC sind enthalten.
- **Öffentlich und ohne Zwischenhändler.** Die Veröffentlichung läuft über Ihre GitHub Releases — ohne Shops und Plattformen dazwischen.

## Funktionen
- **Mehrere Modpacks** — jedes mit eigener Registerkarte: Releases mit Changelogs, Dateien (Mods/Ressourcenpakete/Shader/Welten) mit Aktivieren-Deaktivieren und Suche sowie die Spielkonsole.
- **Inhalt aus dem Pack-Repository**: „Screenshots“-Registerkarte (lokaler Spielerordner) und „Server“-Registerkarte (`servers.json` + eigene aus `servers.dat`), Sterne und soziale Netzwerke (`socials.json`) in der Kopfzeile — als farbige Schaltflächen mit der Autor-Farbe (`color` in `socials.json`), Serverstatus mit Online-Zahl („12/100“ + Nicknamen im Tooltip), „Auf Server spielen“, das Pack-Banner (`banner.png`) und ein Launcher-Design passend zum Pack (`theme.json` — färbt die Oberfläche beim Öffnen sanft um).
- **News**: Pack-Releases und -Diskussionen + die Releases des Launchers. Launcher-Updates installieren sich automatisch (tauri-plugin-updater, Release mit minisign signiert).
- **Ihr eigenes Pack in einer Minute**: Die Schaltfläche „Pack erstellen“ öffnet ein Beispiel; das Einfügen eines Repository-Links fügt das Pack in den Launcher ein (id = `owner-repo`). Ein Einladungslink (`mono://add-pack?...` oder `n1orio.github.io/mono-launcher/?url=...`) fügt das Pack mit einem Klick hinzu.
- **Kostenpflichtige Packs über Boosty**: Ist `boostyBlog` gesetzt, verknüpft der Spieler seinen Boosty-Token — das Abo des Verlags-Blogs gibt Download und Start frei (über die Boosty-API geprüft, ohne eigenes Backend, mit einer Offline-Schonfrist von bis zu 3 Tagen).
- **Pack-Katalog**: Die „Katalog“-Registerkarte — eine kuratierte Liste von Packs mit Beschreibungen und Tags (`catalog.json` im Launcher-Repository), Ein-Klick-Installation aus dem Katalog; Autoren schlagen Packs über ein Issue mit Vorlage vor.
- **Mindest-Arbeitsspeicher**: `minRam` in `pack.json` des Verlags (MB) — ein Badge am Pack, ein Hinweis in den Einstellungen und Schutz vor Start bei zu wenig RAM.
- **Bug-Reports**: Die Schaltfläche „Bug melden“ öffnet eine Formularseite zu den Issues des Pack-Repositories, vorausgefüllt mit der Umgebung (Pack-Version, Minecraft, Launcher, OS).
- **Dateiintegrität**: Alle Dateien werden gegen sha1/sha512 aus `modrinth.index.json` geprüft; Dateien aus nicht vertrauenswürdigen Quellen und `.jar` aus `overrides` werden als benutzerdefiniert markiert und in einem Warnbanner angezeigt.
- **Anmeldung**: Offline-Konto oder Microsoft (Device-Code-Flow). **Mehrere Konten**: eine Liste gespeicherter Profile in den Einstellungen mit Ein-Klick-Umschaltung (jede Anmeldung landet in der Liste und wird aktiv).
- **Statistik**: Eine „N h“-Pille in der Pack-Kopfzeile (Spielzeit); bei einem frisch installierten Pack — „Noch nie gestartet“ bis zum ersten Start.
- Dunkles/helles Design, russische/englische Oberfläche, Discord RPC, Spielzeit, automatische Erkennung von Java 21+ und JRE-Verzeichnissen.

## Technik
- **Backend:** Rust + [Tauri 2.0](https://tauri.app)
- **Frontend:** Nuxt 3 (Vue 3 + TypeScript + Tailwind CSS), SSR deaktiviert — ein SPA über IPC
- **Pack-Quelle:** `.mrpack` über GitHub Releases

## Struktur
```
mono-launcher/
├── app.vue / nuxt.config.ts / package.json / tsconfig.json
├── pages/index.vue          # die einzige Seite (gesamte UI)
├── composables/useLauncher.ts  # Zustand + Logik, useI18n.ts — dynamische Locales
├── catalog.json             # Pack-Katalog („Katalog“-Registerkarte im Launcher)
├── lib/                     # bridge.ts (typisierter Tauri-IPC) + types.ts
├── assets/css/main.css      # globales CSS, Tailwind, Designvariablen (var(--*))
├── scripts/
│   ├── dev-headless.sh       # Headless-Start (Xvfb) ohne Monitor
│   └── make-updater-json.mjs # latest.json für Auto-Update bauen
└── src-tauri/                # Rust-Backend
    ├── Cargo.toml / tauri.conf.json
    └── src/
        ├── main.rs / lib.rs  # Tauri-Befehle, Deep Links (mono://), News, API-Cache
        ├── config.rs         # PACKS (eingebaute Packs) + Pfade
        ├── mrpack.rs         # .mrpack, modrinth.index.json, eigene Mods, Versionen
        ├── auth.rs           # offline + Microsoft OAuth2
        ├── license.rs        # kostenpflichtige Packs: Boosty-Abo (Spielertoken)
        ├── game.rs           # Startprofile (NeoForge/Forge/Fabric/Quilt), Java-Start
        ├── jre.rs            # Java-Suche/Auswahl
        ├── files.rs          # Versionsdateien (Mods/Ressourcenpakete/…)
        ├── discord_rp.rs     # Discord Rich Presence
        └── lib.rs            # registriert alle Befehle in run()
```

## Schnellstart
```bash
npm install
npm run tauri dev      # Start im Dev-Modus (vite auf :1420 und Rust)
npm run tauri build    # Binary bauen (nuxt build → dist/ → tauri)
npm run tauri:appimage # Linux AppImage (mit NO_STRIP=true)
```

## Für Pack-Autoren: Veröffentlichung und Vertrieb

### Schritt 1. Das Pack als `.mrpack` bauen
In Prism Launcher: Rechtsklick auf die Instanz → **Exportieren** → **Modrinth Pack (.mrpack)**.
So erhalten Sie eine `modpack.mrpack`-Datei — ein Zip mit `modrinth.index.json` (Minecraft-Version, Mod-Loader, Dateiliste mit Hashes). Der Dateiname kann beliebig mit der Endung `.mrpack` sein, wir empfehlen aber `modpack.mrpack`.

### Schritt 2. Ein öffentliches GitHub-Repository erstellen
Der Launcher ruft die GitHub-API ohne Autorisierung auf — das Repository **muss öffentlich** sein.
Aus dem Repository-Namen ergibt sich die Pack-ID `owner-repo`.

### Schritt 3. Einen Release mit zwei Dateien veröffentlichen
GitHub → **Releases** → **Create a new release**. Tag = Pack-Version (z. B. `1.0.0`); die Release-Notizen werden der Changelog im Launcher. Hängen Sie Assets an den Release an:

| Datei | Pflicht | Zweck |
|-------|:-------:|-------|
| `modpack.mrpack` | ja | das Pack selbst |
| `pack.json` | ja | Metadaten: Name, Bezahlstatus, Mindestarbeitsspeicher |

⚠️ Der Release muss **veröffentlicht** werden („Publish release“-Button), nicht als Entwurf bleiben — Entwürfe sind für den Launcher unsichtbar.

Beispiel `pack.json`:
```json
{ "name": "My Pack", "boostyBlog": "my_blog", "minRam": 8192 }
```
- `name` — der Pack-Name (ohne ihn wird der Repository-Name verwendet).
- `boostyBlog` — der Boosty-Blog-Nickname: das Pack wird **kostenpflichtig** (siehe unten); ohne Feld — kostenlos.
- `minRam` — Mindestarbeitspeicher in MB (gültiger Bereich 256–65536): ein „≥ N GB“-Badge am Pack und Schutz vor Start bei zu wenig RAM.
- Auch snake_case-Varianten werden akzeptiert: `boosty_blog`, `min_ram`.

### Schritt 4. Das Hinzufügen prüfen
Im Launcher: die Schaltfläche **„Pack hinzufügen“** → Repository-URL (oder einen direkten `.mrpack`-Link) einfügen.
Der Launcher prüft die Releases: Er findet die `.mrpack`-Datei und daneben `pack.json` — ist alles korrekt, erscheint das Pack in der Liste.

### Schritt 5. Verteilen
- **Universeller Einladungslink** — der beste Weg (Details im nächsten Abschnitt):
  Ein Spieler folgt dem Link — das Pack wird mit einem Klick in den Launcher aufgenommen.
- **Katalog**: Mit der Schaltfläche „Pack vorschlagen“ (Issue mit Vorlage) oder per PR zu `catalog.json` in diesem Repository — das Pack erscheint in der „Katalog“-Registerkarte mit Beschreibung, Tags und einem „Hinzufügen“-Button.
- **Eingebaute Packs** — in Absprache mit dem Launcher-Autor: `PACKS` in `src-tauri/src/config.rs`.

### Universeller Einladungslink (für Spieler und Werbung)
Ein Link macht alles: Bei einem Spieler mit installiertem Launcher öffnet sich der Launcher und das Pack wird mit einem Klick hinzugefügt; bei einem Spieler ohne Launcher — eine Seite mit einem „Mono Launcher herunterladen“-Button.

**Linkformat** — zwei Varianten funktionieren gleich:

```
mono://add-pack?url=<codiert>&name=<codiert>&blog=<codiert>
https://n1orio.github.io/mono-launcher/?url=<codiert>&name=<codiert>
```

**Parameter** (Werte percent-encoded, d. h. `encodeURIComponent`):

| Parameter | Pflicht | Was er übergibt |
|-----------|:-------:|-----------------|
| `url` | ja | Der Pack-Repository-Link: `https://github.com/<owner>/<repo>` |
| `name` | nein | Der Pack-Name (fehlt er, wird er aus `pack.json` oder dem Repository-Namen genommen) |
| `blog` | nein | Boosty-Blog-Nickname — das Pack wird **kostenpflichtig** hinzugefügt (hat Vorrang vor `pack.json`) |

**Wie Sie einen fertigen Link bekommen (am einfachsten):**
Fügen Sie Ihr Pack in den Launcher ein, öffnen Sie seine Registerkarte und drücken Sie
**„Einladungslink kopieren“** in der Kopfzeile — der Launcher baut den Link
mit allen Parametern selbst (inklusive `blog=` für kostenpflichtige Packs).

**Wie Sie ihn von Hand bauen** (ohne Launcher): codieren Sie die Werte und setzen Sie sie in die Vorlage ein.
Beispiel für das Repository `https://github.com/n1orio/mono-pack-example`:

```
https://n1orio.github.io/mono-launcher/?url=https%3A%2F%2Fgithub.com%2Fn1orio%2Fmono-pack-example&name=My%20Pack&blog=my-blog
```

Hier ist `https%3A%2F%2F...` die codierte `https://...`-Adresse, und `My%20Pack` ist `My Pack` (Leerzeichen → `%20`).
Die zweite Variante (Schema `mono://`) akzeptiert ebenfalls Parameter, sollte aber besser nicht im Web veröffentlicht werden —
der Browser zeigt „unbekanntes Protokoll“: Die Webversion (`https://n1orio.github.io/mono-launcher/?...`)
probieren selbst, `mono://` zu öffnen, und bietet bei fehlendem Launcher einen Download an.

**Was der Spieler sieht:**
1. Launcher installiert → die App öffnet sich, das Pack wird validiert
   (das Repository muss existieren, und in seinen **veröffentlichten** Releases müssen
   `.mrpack` + `pack.json` liegen — sonst sieht der Spieler einen Fehler) und zur Liste hinzugefügt.
2. Kein Launcher installiert → eine Webbrücken-Seite öffnet sich mit dem Pack-Namen,
   einem „Im Launcher öffnen“-Button und einem „Mono Launcher herunterladen“-Link.

**Wo der Link veröffentlicht werden sollte:** Beschreibung und erster Kommentar Ihres Releases,
README Ihres Repositories, Discord/Telegram-Kanäle, Videobeschreibungen und Twitter —
der Link ist kurz und braucht keine Erklärung.

### Updates
Veröffentlichen Sie einen neuen Release mit neuer `modpack.mrpack`- und `pack.json`-Datei — der Launcher merkt das selbst und zeigt den Spielern einen Update-Button (das Release-Tag = die neue Version, der Changelog kommt aus den Notizen).

### Zusätzlicher Inhalt (optional, im Repository-Root)
- `servers.json` — offizielle Server des Packs („Server“-Registerkarte): `[{ "name": "Name", "ip": "play.example.com", "port": 25565, "desc": "Beschreibung" }]`
- `socials.json` — farbige Social-Media-Buttons in der Kopfzeile: Objekt `{ "Discord": "https://…", "Telegram": "https://…" }` oder ein Array; maximal 8, nur https, jeder kann eine `color: "#rrggbb"` haben.
- `banner.png` — das Banner in der Kopfzeile der Pack-Registerkarte.
- `theme.json` — der Launcher färbt sich passend zum Pack um: `{ "bg": "#rrggbb", "panel": "…", "input": "…", "border": "…", "tx": "…", "txStrong": "…", "txMuted": "…", "accent": "…", "accentStrong": "…", "accentHover": "…", "accentDeep": "…" }` — alle Werte sind hex `#rrggbb`.

### Checkliste vor der Veröffentlichung
- [ ] Repository ist öffentlich
- [ ] Release ist veröffentlicht (kein Entwurf), Tag = Pack-Version
- [ ] Im Release liegen `modpack.mrpack` und `pack.json`
- [ ] `pack.json` — gültiges JSON mit dem Feld `name` (empfohlen)
- [ ] Das Pack wurde über den Repository-Link im Launcher hinzugefügt

## Pack-Katalog
Die „Katalog“-Registerkarte zeigt eine kuratierte Liste von Packs aus `catalog.json` im Root dieses
Repositories (aktualisiert über raw.githubusercontent, ohne GitHub-API-Kontingente). Ein Katalog-Eintrag:

```json
{
  "name": "My Pack",
  "url": "https://github.com/my-name/my-pack",
  "description": "Eine kurze Beschreibung...",
  "author": "my-name",
  "tags": ["Abenteuer", "Magie"],
  "boostyBlog": "my_blog",
  "minRam": 8192
}
```

- `url` — ein GitHub-Repository (oder ein direkter `.mrpack`-Link), wie beim Hinzufügen per Link.
- `boostyBlog` — wenn das Pack kostenpflichtig ist (siehe unten), damit das Abo sofort angelegt wird.
- `minRam` — Mindestarbeitspeicher in MB.
- „Hinzufügen“ installiert das Pack mit einem Klick; bei bereits hinzugefügten ändert sich der Button zu „Öffnen“.
- Autoren: Sie können ein Pack mit „Pack vorschlagen“ (Issue mit Vorlage) vorschlagen
  oder per PR zu `catalog.json` in diesem Repository.

## Kostenpflichtige Packs über Boosty (ohne eigenes Backend)
Die Abo-Prüfung läuft im Namen des Spielers, sodass der Verlag keinen Server braucht:

1. Der Spieler hat seinen persönlichen Boosty-Token: Boosty → Einstellungen → **Apps** → „App erstellen“ (der Token wird sofort ausgegeben).
2. Im Launcher (Abo-Panel in der Pack-Kopfzeile) fügt der Spieler den Token ein — der Launcher prüft
   das Abo Ihres Blogs über `api.boosty.to` (`user/me` + `user/<id>/subscriptions`).
3. Die Verknüpfung wird in `licenses.json` im Launcher-Root gespeichert; nach einer erfolgreichen Prüfung
   gilt eine **Schonfrist von bis zu 3 Tagen** — eine Netzwerkprüfung ist nicht bei jedem Start nötig.
4. Kündigung auf Boosty → nach der nächsten Prüfung wird der Zugang geschlossen.
5. Ein Pack aus Ihrer `pack.json` und der Einladungslink übertragen automatisch
   `boostyBlog` an die Spieler — legen Sie es vor der Veröffentlichung in `pack.json` (oder tragen Sie es
   in `packs.json` des Nutzers ein: `"boostyBlog": "my_blog"`).

Einschränkungen: Die Prüfung benötigt Netzwerk; der Boosty-Token des Spielers ist sein Geheimnis, der Launcher speichert
ihn nur lokal (`licenses.json`) und sendet ihn an nichts außer `api.boosty.to`.

## Datenspeicherung
```
~/.local/share/MonoLauncher/
├── packs/<pack_id>/                # Daten eines einzelnen Packs
│   ├── versions/<versionId>/       # Spielprofil einer bestimmten Version
│   │   ├── .mono-installed.json     # Installationsmarker (versionId, name, sourceTag)
│   │   ├── .mono-index.json         # Kopie von modrinth.index.json
│   │   ├── .mono-custom.json        # eigene Mods (nicht vertrauenswürdige Quellen / overrides)
│   │   └── mods/ config/ overrides # Pack-Inhalt
│   ├── active.json                 # aktive Version
│   └── mrpack-cache/               # heruntergeladenes .mrpack
├── libraries/                      # Minecraft-/Mod-Loader-Bibliotheken
└── runtime/                        # mitgelieferte Java (optional)
```
Eigene Packs werden ebenfalls in `packs.json` im Launcher-Root registriert.

## Tauri-Befehle (Hauptbefehle)
| Befehl | Beschreibung |
|--------|--------------|
| `list_packs` / `add_pack_command` / `remove_pack_command` | Pack auflisten / hinzufügen / entfernen |
| `check_for_updates(packId?)` | Nach neuer `.mrpack`-Version auf GitHub suchen |
| `install_mrpack(packId?, tag?)` | Version herunterladen, entpacken, Mods + `overrides` installieren, Hashes prüfen, Fortschritt via `download-progress` |
| `list_versions(packId?)` | GitHub-Releases (Tag, Datum, Changelog) + installierte Versionen + aktive |
| `switch_version(packId?, versionId)` | Aktive Version wechseln |
| `get_status(packId?)` | Zustand der aktiven Version, Session, RAM, eigene Mods |
| `get_news` | News: Pack-Releases/-Diskussionen + Launcher-Releases |
| `pack_repo_content(packId?)` | Pack-Sterne, Server (`servers.json`), soziale Netzwerke (`socials.json`) |
| `fetch_catalog` | Pack-Katalog aus `catalog.json` („Katalog“-Registerkarte) |
| `list_screenshots(packId?)` / `list_servers(packId?)` | Screenshots aus dem `screenshots`-Ordner und Server aus `servers.dat` der installierten Version |
| `ping_server(address, port?)` | Minecraft-Serverstatus (online/Spieler/Version/Ping) |
| `list_game_files_*` / `toggle_game_file` | Versionsdateien: Liste, Symbol, Aktivieren-Deaktivieren |
| `login_offline_command` / `ms_device_code` / `ms_poll` | Offline-Login / Microsoft OAuth2 |
| `launch_game_command(packId?, ram, session)` | Java mit angegebenem RAM starten |
| `list_java` / `ensure_java` | Gefundene Java auflisten / JRE installieren |
| `set_boosty(packId, token)` / `license_status(packId)` / `clear_license(packId)` | Boosty verknüpfen / Abo-Status / lösen |
| `system_info` | System-/verfügbarer Speicher (für empfohlenen RAM) |
| `open_external` / `open_game_folder` / `get_skin` | URL/Ordner öffnen, Mojang-Skin |

## So funktioniert die `.mrpack`-Installation
1. `modpack.mrpack` wird von GitHub Releases heruntergeladen (der ausgewählte Tag).
2. Das Archiv wird in einen temporären Ordner entpackt.
3. `modrinth.index.json` wird gelesen: Minecraft-Version, Mod-Loader, das `files`-Array.
4. Alle Dateien aus `files` werden parallel heruntergeladen (`tokio` + `reqwest`, Limit von 8 Verbindungen)
   mit SHA-1/SHA-512-Prüfung; ein Nichtübereinstimmen löscht die Datei und stoppt die Installation.
5. `overrides/` wird in das Versionsprofil kopiert; Installationsmarker und Index werden geschrieben; die Version wird aktiv.

## Sicherheit
- Mods gelten nur von den Modrinth/CurseForge-CDs als vertrauenswürdig (`cdn.modrinth.com`,
  `dl.modrinth.com`, `mediafiles.forgecdn.net`).
- Andere Dateien und `.jar` aus `overrides` — „benutzerdefiniert“: sie werden in `.mono-custom.json`
  der Version geschrieben und in einem Warnbanner angezeigt. Die Installation wird nicht blockiert (die Wahl des Autors),
  aber die Quelle wird hervorgehoben.

## Spielstart
- NeoForge / Forge / Fabric / Quilt (Startprofil aus dem Mod-Loader-Installer).
- Java 21+; automatische Suche in PATH und üblichen Verzeichnissen (`jre.rs`) oder manuelle Auswahl.
- Wayland bei leerem/weißem Fenster: `WEBKIT_DISABLE_COMPOSITING_MODE=1` und
  `WEBKIT_DISABLE_DMABUF_RENDERER=1` werden automatisch gesetzt.
- Headless-UI-Debugging: `scripts/dev-headless.sh`.