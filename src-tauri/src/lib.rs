mod auth;
mod author;
mod config;
mod crash;
mod curseforge;
mod discord_rp;
mod export;
mod files;
mod game;
mod http_cache;
mod jre;
mod license;
mod modrinth;
mod mrpack;
mod nbt;
mod ping;
mod skins;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use sysinfo::System;
use tauri::{AppHandle, Manager, State};
use tauri::{Emitter, Listener};

use crate::auth::{login_offline, save_session, UserSession};
use crate::config::{default_pack_id, PackInfo};
use crate::author::export_author_pack_command;
use crate::export::{export_list_command, export_pack_command};

/// Глобальное состояние лаунчера (HTTP-клиент).
pub struct AppState {
    pub client: reqwest::Client,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppStatus {
    pub installed: bool,
    pub minecraft_version: Option<String>,
    pub loader: Option<String>,
    pub loader_version: Option<String>,
    pub pack_name: Option<String>,
    pub session: Option<UserSession>,
    pub mrpack_url: String,
    pub active_version: Option<String>,
    pub active_source_tag: Option<String>,
    pub installed_versions: Vec<String>,
    pub discord_rp_enabled: bool,
    /// Показывать ли плашку предупреждения о кастомных модах (warn-custom-mods.txt).
    pub warn_custom_mods: bool,
    /// Суммарное время игры в этой сборке (секунды).
    pub playtime_seconds: u64,
    /// Суммарное время игры во всех сборках (секунды).
    pub total_playtime_seconds: u64,
    /// Сколько сборок когда-либо запускалось.
    pub played_packs: u64,
    /// Файлы активной версии, скачанные не с доверенных CDN (кастомные моды).
    pub custom_mods: Vec<mrpack::CustomFile>,
}

/// Публичное описание сборки для фронтенда.
#[derive(Debug, Clone, Serialize)]
pub struct PackDescriptor {
    pub id: String,
    pub name: String,
    pub url: String,
    pub builtin: bool,
    /// "remote" | "local" (своя сборка / с Modrinth).
    pub kind: String,
    /// Владелец GitHub-репозитория сборки (если это github-сборка).
    pub author: Option<String>,
    /// Ник блога на Boosty: задан → сборка платная (подписка обязательна).
    #[serde(rename = "boostyBlog")]
    pub boosty_blog: Option<String>,
    /// Минимальная оперативка (МБ), из pack.json издателя / конфига встроенной сборки.
    #[serde(rename = "minRam")]
    pub min_ram_mb: Option<u32>,
    /// Локальная иконка сборки (абсолютный путь), если есть.
    pub icon: Option<String>,
    /// Локальный баннер сборки (абсолютный путь `packs/<id>/banner.png`), если есть.
    pub banner: Option<String>,
}

/// Имя владельца репозитория из URL сборки (github.com/OWNER/...).
fn repo_author(url: &str) -> Option<String> {
    parse_github_repo_from_url(url).map(|(owner, _)| owner)
}

#[derive(Debug, Serialize)]
pub struct UpdateInfo {
    pub current_version: Option<String>,
    pub latest_version: Option<String>,
    pub has_update: bool,
}

/// Релиз сборки на GitHub.
#[derive(Debug, Clone, Serialize)]
pub struct GhVersion {
    pub tag: String,
    pub name: String,
    pub url: String,
    pub prerelease: bool,
    pub published_at: Option<String>,
    pub body: String,
    pub assets: Vec<String>,
}

/// Всё, что нужно фронтенду для выбора версии.
#[derive(Debug, Serialize)]
pub struct VersionsInfo {
    pub github: Vec<GhVersion>,
    pub installed: Vec<mrpack::InstalledVersion>,
    pub active: Option<String>,
}

/// Один элемент ленты новостей: обновление сборки (релиз) или пост (discussion).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewsItem {
    pub kind: String,
    pub pack_id: String,
    pub pack_name: String,
    pub title: String,
    pub body: String,
    pub url: String,
    pub tag: Option<String>,
    pub category: Option<String>,
    pub date: Option<String>,
}

/// Сервер сборки из `servers.json` в корне репозитория.
#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct PackServer {
    pub name: String,
    pub ip: String,
    pub port: Option<u16>,
    pub desc: Option<String>,
}

/// Соцсеть сборки из `socials.json` в корне репозитория.
/// `color` — цвет кнопки в формате `#rrggbb` (необязательно, иначе акцент темы).
#[derive(Debug, Clone, serde::Serialize)]
pub struct PackSocial {
    pub name: String,
    pub url: String,
    pub color: Option<String>,
}

/// Тема лаунчера из `theme.json` в корне репозитория сборки (все поля — hex-цвета `#rrggbb`).
#[derive(Debug, Clone, serde::Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PackTheme {
    pub bg: Option<String>,
    pub panel: Option<String>,
    pub input: Option<String>,
    pub border: Option<String>,
    pub tx: Option<String>,
    pub tx_strong: Option<String>,
    pub tx_muted: Option<String>,
    pub accent: Option<String>,
    pub accent_strong: Option<String>,
    pub accent_hover: Option<String>,
    pub accent_deep: Option<String>,
}

/// Контент репозитория сборки: звёзды GitHub + сервера + соцсети + тема.
#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct PackRepoContent {
    pub stars: Option<i64>,
    pub servers: Vec<PackServer>,
    pub socials: Vec<PackSocial>,
    pub theme: Option<PackTheme>,
    /// URL баннера сборки (banner.png в корне репозитория, raw.githubusercontent).
    pub banner: Option<String>,
}

/// Запись каталога сборок: курируемый список авторов в `catalog.json`
/// корня репозитория лаунчера (fetch по raw.githubusercontent, без API квоты).
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct CatalogEntry {
    pub name: String,
    /// URL репозитория GitHub сборки (или прямая ссылка на .mrpack).
    pub url: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    /// Ник блога Boosty: задан → сборка платная (подписка обязательна).
    #[serde(default, rename = "boostyBlog")]
    pub boosty_blog: Option<String>,
    /// Минимальная оперативка (МБ).
    #[serde(default, rename = "minRam")]
    pub min_ram_mb: Option<u32>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Источник каталога сборок (raw-файл в корне репозитория лаунчера).
const CATALOG_URL: &str = "https://raw.githubusercontent.com/n1orio/mono-launcher/main/catalog.json";

/// Информация о системе для ползунка RAM.
#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub total_ram_gb: u64,
    pub available_ram_gb: u64,
}

/// Определяет сборку по id (или берёт дефолтную из конфига).
fn resolve_pack(pack_id: Option<String>) -> Result<PackInfo, String> {
    let id = pack_id.unwrap_or_else(|| default_pack_id().to_string());
    config::find_pack(&id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Сборка не найдена: {id}"))
}

/// Тег релиза (или versionId) активной установленной версии — основа для сравнения
/// с релизами GitHub и отображения в UI.
fn active_installed_tag(pack_id: &str) -> Option<String> {
    let active = config::active_version(pack_id)
        .ok()
        .filter(|v| !v.is_empty())?;
    mrpack::installed_details(pack_id)
        .into_iter()
        .find(|v| v.version_id == active)
        .map(|v| v.source_tag.clone().unwrap_or_else(|| v.version_id.clone()))
}

/// Извлекает owner/repo из URL сборки, чтобы опрашивать GitHub API.
fn parse_github_repo(pack: &PackInfo) -> Option<(String, String)> {
    parse_github_repo_from_url(&pack.url)
}

/// Извлекает owner/repo из произвольной github-ссылки.
pub(crate) fn parse_github_repo_from_url(url: &str) -> Option<(String, String)> {
    let rest = url
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/");
    let mut parts = rest.split('/');
    let owner = parts.next()?;
    let repo = parts.next()?.split('/').next()?;
    if owner == "USER" || repo == "REPO" || owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some((owner.to_string(), repo.to_string()))
}

/// Имя `.mrpack` файла из URL сборки.
fn mrpack_file_name(pack: &PackInfo) -> Option<String> {
    std::path::Path::new(&pack.url)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
}

/// URL скачивания `.mrpack` для конкретного тега релиза.
fn mrpack_url_for_tag(pack: &PackInfo, tag: &str) -> String {
    let file = mrpack_file_name(pack).unwrap_or_else(|| "modpack.mrpack".into());
    let Some((owner, repo)) = parse_github_repo(pack) else {
        return pack.url.clone();
    };
    format!("https://github.com/{owner}/{repo}/releases/download/{tag}/{file}")
}

/// Идентификатор сборки из репозитория GitHub (например n1orio/My-Pack -> n1orio-my-pack).
fn pack_id_from_repo(owner: &str, repo: &str) -> String {
    let slug = |s: &str| {
        s.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect::<String>()
    };
    format!("{}-{}", slug(owner), slug(repo))
}

const API_HIT_TTL: Duration = Duration::from_secs(15 * 60);
const API_FAIL_RETRY: Duration = Duration::from_secs(60);

/// Кэш GitHub API: успешные ответы живут 15 минут, ошибки (в т.ч. rate limit)
/// не дают долбить API чаще раза в минуту.
struct ApiCache {
    releases: HashMap<String, (Instant, Vec<GhVersion>)>,
    discussions: HashMap<String, (Instant, Vec<NewsItem>)>,
    meta: HashMap<String, (Instant, PackRepoContent)>,
    catalog: Option<(Instant, Vec<CatalogEntry>)>,
    failures: HashMap<String, Instant>,
}

static API_CACHE: OnceLock<std::sync::Mutex<ApiCache>> = OnceLock::new();

fn api_cache() -> &'static std::sync::Mutex<ApiCache> {
    API_CACHE.get_or_init(|| {
        std::sync::Mutex::new(ApiCache {
            releases: HashMap::new(),
            discussions: HashMap::new(),
            meta: HashMap::new(),
            catalog: None,
            failures: HashMap::new(),
        })
    })
}

fn repo_key(pack: &PackInfo) -> Option<String> {
    parse_github_repo(pack).map(|(o, r)| format!("{o}/{r}"))
}

/// `fetch_releases` с кэшем: повторные запросы в течение TTL не ходят в сеть,
/// а при сбое (403 rate limit, нет сети) повторная попытка — не раньше чем через минуту.
async fn fetch_releases_cached(client: &reqwest::Client, pack: &PackInfo) -> Vec<GhVersion> {
    let Some(key) = repo_key(pack) else {
        return Vec::new();
    };
    {
        let cache = api_cache().lock().unwrap_or_else(|e| e.into_inner());
        if let Some((at, v)) = cache.releases.get(&key) {
            if at.elapsed() < API_HIT_TTL {
                return v.clone();
            }
        } else if let Some(at) = cache.failures.get(&key) {
            if at.elapsed() < API_FAIL_RETRY {
                return Vec::new();
            }
        }
    }
    let fetched = fetch_releases(client, pack).await;
    let mut cache = api_cache().lock().unwrap_or_else(|e| e.into_inner());
    if fetched.is_empty() {
        cache.failures.insert(key, Instant::now());
    } else {
        cache
            .releases
            .insert(key.clone(), (Instant::now(), fetched.clone()));
        cache.failures.remove(&key);
    }
    fetched
}

/// `fetch_discussions` с тем же кэшем, что и релизы.
async fn fetch_discussions_cached(
    client: &reqwest::Client,
    owner: &str,
    repo: &str,
    kind: &str,
    pack_id: &str,
    pack_name: &str,
) -> Vec<NewsItem> {
    let key = format!("disc/{owner}/{repo}");
    {
        let cache = api_cache().lock().unwrap_or_else(|e| e.into_inner());
        if let Some((at, v)) = cache.discussions.get(&key) {
            if at.elapsed() < API_HIT_TTL {
                return v.clone();
            }
        } else if let Some(at) = cache.failures.get(&key) {
            if at.elapsed() < API_FAIL_RETRY {
                return Vec::new();
            }
        }
    }
    let fetched = fetch_discussions(client, owner, repo, kind, pack_id, pack_name).await;
    let mut cache = api_cache().lock().unwrap_or_else(|e| e.into_inner());
    if fetched.is_empty() {
        cache.failures.insert(key, Instant::now());
    } else {
        cache
            .discussions
            .insert(key.clone(), (Instant::now(), fetched.clone()));
        cache.failures.remove(&key);
    }
    fetched
}

/// Релизы сборки с GitHub (URL = прямая ссылка на `.mrpack`).
async fn fetch_releases(client: &reqwest::Client, pack: &PackInfo) -> Vec<GhVersion> {
    let Some((owner, repo)) = parse_github_repo(pack) else {
        return Vec::new();
    };
    let mut out = fetch_repo_releases(client, &owner, &repo).await;
    for v in &mut out {
        v.url = mrpack_url_for_tag(pack, &v.tag);
    }
    out
}

/// Релизы произвольного GitHub-репозитория (URL = страница релиза).
async fn fetch_repo_releases(client: &reqwest::Client, owner: &str, repo: &str) -> Vec<GhVersion> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases");
    let Some(json) = http_cache::cached_json(client, &url).await else {
        return Vec::new();
    };
    let Some(arr) = json.as_array() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for rel in arr {
        if rel["draft"].as_bool().unwrap_or(false) {
            continue;
        }
        let tag = rel["tag_name"].as_str().unwrap_or("").to_string();
        if tag.is_empty() {
            continue;
        }
        let name = rel["name"].as_str().unwrap_or(&tag).to_string();
        let assets = rel["assets"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|a| a["name"].as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        out.push(GhVersion {
            url: format!("https://github.com/{owner}/{repo}/releases/tag/{tag}"),
            tag,
            name,
            prerelease: rel["prerelease"].as_bool().unwrap_or(false),
            published_at: rel["published_at"].as_str().map(|s| s.to_string()),
            body: rel["body"].as_str().unwrap_or("").to_string(),
            assets,
        });
    }
    out
}

/// Релизы лаунчера (новости обновлений) с кэшем, как у сборок.
async fn fetch_launcher_releases_cached(client: &reqwest::Client) -> Vec<GhVersion> {
    let key = "rel/launcher".to_string();
    {
        let cache = api_cache().lock().unwrap_or_else(|e| e.into_inner());
        if let Some((at, v)) = cache.releases.get(&key) {
            if at.elapsed() < API_HIT_TTL {
                return v.clone();
            }
        } else if let Some(at) = cache.failures.get(&key) {
            if at.elapsed() < API_FAIL_RETRY {
                return Vec::new();
            }
        }
    }
    let fetched = fetch_repo_releases(client, NEWS_REPO.0, NEWS_REPO.1).await;
    let mut cache = api_cache().lock().unwrap_or_else(|e| e.into_inner());
    if fetched.is_empty() {
        cache.failures.insert(key, Instant::now());
    } else {
        cache
            .releases
            .insert(key.clone(), (Instant::now(), fetched.clone()));
        cache.failures.remove(&key);
    }
    fetched
}

/// Hub-репозиторий с глобальными постами/новостями (GitHub Discussions).
/// Посты также считываются из репозиториев каждой сборки.
const NEWS_REPO: (&str, &str) = ("n1orio", "mono-launcher");

/// Манифест обновления лаунчера (latest.json). Используется, помимо апдейтера,
/// как источник локализованных нот новостей (`notes_localized[locale]`).
const LAUNCHER_MANIFEST_URL: &str =
    "https://github.com/n1orio/mono-launcher/releases/latest/download/latest.json";

/// Посты (обсуждения) из репозитория. Discussions должны быть включены,
/// иначе репозиторий просто не даёт постов — не ошибка.
async fn fetch_discussions(
    client: &reqwest::Client,
    owner: &str,
    repo: &str,
    kind: &str,
    pack_id: &str,
    pack_name: &str,
) -> Vec<NewsItem> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/discussions?per_page=100");
    let Some(json) = http_cache::cached_json(client, &url).await else {
        return Vec::new();
    };
    let Some(arr) = json.as_array() else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|d| {
            let title = d["title"].as_str().unwrap_or("").trim().to_string();
            if title.is_empty() {
                return None;
            }
            Some(NewsItem {
                kind: kind.into(),
                pack_id: pack_id.to_string(),
                pack_name: pack_name.to_string(),
                title,
                body: d["body"].as_str().unwrap_or("").to_string(),
                url: d["html_url"].as_str().unwrap_or("").to_string(),
                tag: None,
                category: d["category"]["name"].as_str().map(|s| s.to_string()),
                date: d["created_at"].as_str().map(|s| s.to_string()),
            })
        })
        .collect()
}

/// Список сборок: встроенные + добавленные пользователем.
#[tauri::command]
fn list_packs() -> Result<Vec<PackDescriptor>, String> {
    config::all_packs()
        .map(|packs| {
            packs
                .into_iter()
                .map(|p| {
                    let author = repo_author(&p.url);
                    PackDescriptor {
                        id: p.id,
                        name: p.name,
                        url: p.url,
                        builtin: p.builtin,
                        kind: p.kind.clone(),
                        author,
                        boosty_blog: p.boosty_blog.clone(),
                        min_ram_mb: p.min_ram_mb,
                        icon: p.icon,
                        banner: p.banner,
                    }
                })
                .collect()
        })
        .map_err(|e| e.to_string())
}

/// Добавляет сборку по URL репозитория GitHub (или прямой ссылке на `.mrpack`).
/// Проверяет, что репозиторий существует и в его релизах есть `.mrpack` и `pack.json`.
/// `blog` (из deep link) — ник блога на Boosty; если не задан, читается из pack.json.
/// `minRam` читается из pack.json (МБ).
async fn add_pack_impl(
    client: &reqwest::Client,
    url: &str,
    name: Option<&str>,
    blog: Option<&str>,
) -> Result<PackDescriptor, String> {
    let url = url.trim().to_string();
    if url.is_empty() || !url.contains("github.com/") {
        return Err(
            "URL должен быть ссылкой на GitHub (например https://github.com/USER/REPO).".into(),
        );
    }
    let (owner, repo) = parse_github_repo_from_url(&url)
        .ok_or("Не удалось разобрать владельца/репозиторий из URL")?;

    // Запрещаем дубликаты по тому же репозиторию (встроенные и пользовательские).
    for existing in config::all_packs().map_err(|e| e.to_string())? {
        if let Some((o, r)) = parse_github_repo(&existing) {
            if o == owner && r == repo {
                return Err(format!("Сборка «{}» уже добавлена", existing.name));
            }
        }
    }

    // Проверяем формат: в релизах должен быть файл .mrpack и рядом с ним JSON
    // с метаданными сборки (pack.json). Это контракт «сборки этого лаунчера».
    let probe = PackInfo {
        id: pack_id_from_repo(&owner, &repo),
        name: repo.clone(),
        url: format!("https://github.com/{owner}/{repo}/releases/latest/download/modpack.mrpack"),
        builtin: false,
        kind: "remote".into(),
        boosty_blog: blog.map(String::from),
        min_ram_mb: None,
        icon: None,
        banner: None,
    };
    let releases = fetch_releases(client, &probe).await;
    let mrpack_release = releases.iter().find(|r| {
        r.assets
            .iter()
            .any(|a| a.to_ascii_lowercase().ends_with(".mrpack"))
    });
    let Some(release) = mrpack_release else {
        return Err(
            "В релизах репозитория нет файла .mrpack — сборка не соответствует формату. \
             Загрузите .mrpack в GitHub Releases (создайте релиз с этим файлом) и повторите."
                .into(),
        );
    };
    let json_asset = release
        .assets
        .iter()
        .find(|a| a.to_ascii_lowercase().ends_with(".json"))
        .cloned();
    let Some(json_asset) = json_asset else {
        return Err(
            "В релизе с .mrpack нет файла pack.json с описанием сборки. \
             Загрузите его в тот же релиз (см. пример в разделе «Разработчикам»)."
                .into(),
        );
    };

    // Ссылку на .mrpack берём из URL пользователя, если она ведёт на файл,
    // иначе строим её от актуального релиза с .mrpack.
    let mrpack_url = if url.to_ascii_lowercase().ends_with(".mrpack") {
        url.clone()
    } else {
        let asset = release
            .assets
            .iter()
            .find(|a| a.to_ascii_lowercase().ends_with(".mrpack"))
            .unwrap();
        format!(
            "https://github.com/{owner}/{repo}/releases/download/{}/{asset}",
            release.tag
        )
    };

    // Метаданные из pack.json: имя сборки, ник блога на Boosty (платность)
    // и минимальная оперативка (minRam, в МБ).
    let user_name = name.map(str::trim).filter(|n| !n.is_empty());
    let mut json_name: Option<String> = None;
    let mut json_blog: Option<String> = None;
    let mut json_min_ram: Option<u32> = None;
    {
        let json_url = format!(
            "https://github.com/{owner}/{repo}/releases/download/{}/{}",
            release.tag, json_asset
        );
        if let Ok(resp) = client
            .get(&json_url)
            .header("User-Agent", "mono-launcher")
            .send()
            .await
        {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if user_name.is_none() {
                    if let Some(n) = json["name"].as_str() {
                        json_name = Some(n.trim().to_string());
                    }
                }
                if let Some(b) = json["boostyBlog"]
                    .as_str()
                    .or_else(|| json["boosty_blog"].as_str())
                {
                    json_blog = Some(b.trim().to_string());
                }
                if let Some(ram) = json["minRam"].as_u64().or_else(|| json["min_ram"].as_u64()) {
                    json_min_ram = Some(ram.clamp(256, 65536) as u32);
                }
            }
        }
    }

    let id = pack_id_from_repo(&owner, &repo);
    let pack_name = user_name
        .map(String::from)
        .or(json_name)
        .unwrap_or(repo)
        .trim()
        .to_string();
    // Блог: параметр deep link → pack.json издателя. Минимальная оперативка — pack.json.
    let boosty_blog = blog
        .map(str::trim)
        .filter(|b| !b.is_empty())
        .map(String::from)
        .or(json_blog);
    config::add_user_pack(
        &id,
        &pack_name,
        &mrpack_url,
        "remote",
        boosty_blog.as_deref(),
        json_min_ram,
    )
    .map_err(|e| e.to_string())?;
    // GitHub-сборки добавляются уже заблокированными (управляемыми): пользователь
    // может их отвязать, чтобы самому менять файлы.
    let _ = config::set_pack_locked(&id, true);
    Ok(PackDescriptor {
        id,
        name: pack_name,
        url: mrpack_url,
        builtin: false,
        kind: "remote".into(),
        author: Some(owner),
        boosty_blog,
        min_ram_mb: json_min_ram,
        icon: None,
        banner: None,
    })
}

#[tauri::command]
async fn add_pack_command(
    state: State<'_, AppState>,
    url: String,
    name: Option<String>,
    blog: Option<String>,
) -> Result<PackDescriptor, String> {
    // Сериализуем с deep-link добавлением, иначе щустрый клик по UI и ссылка
    // могут прочитать/записать packs.json одновременно.
    let _guard = add_pack_lock().lock().await;
    add_pack_impl(&state.client, &url, name.as_deref(), blog.as_deref()).await
}

/// Удаляет пользовательскую сборку (вместе с локальными данными).
#[tauri::command]
fn remove_pack_command(pack_id: String) -> Result<(), String> {
    let pack = config::find_pack(&pack_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Сборка не найдена: {pack_id}"))?;
    if pack.builtin {
        return Err("Встроенную сборку нельзя удалить".into());
    }
    config::remove_user_pack(&pack_id).map_err(|e| e.to_string())?;
    Ok(())
}

// ==== Modrinth: свои сборки, моды, обновления ====

/// Поиск модов/модпаков на Modrinth.
/// `kind` — "mod" | "modpack"; пустой query — топ по загрузкам.
/// Фильтры: categories (категории+загрузчики), versions (версии игры),
/// environment ("client"/"server"), index — сортировка.
#[tauri::command]
async fn modrinth_search_command(
    state: State<'_, AppState>,
    query: String,
    kind: String,
    limit: Option<u32>,
    offset: Option<u32>,
    filters: Option<modrinth::SearchFilters>,
) -> Result<Vec<modrinth::ModrinthProject>, String> {
    modrinth::search_projects(
        &state.client,
        &query,
        &kind,
        limit.unwrap_or(20),
        offset.unwrap_or(0),
        &filters.unwrap_or_default(),
    )
    .await
    .map_err(|e| e.to_string())
}

/// Теги Modrinth для фильтров поиска: загрузчики, категории, версии игры.
/// `kind` — тип проекта ("mod" | "modpack" | "resourcepack" | "shaderpack" | "datapack").
#[tauri::command]
async fn modrinth_tags_command(
    state: State<'_, AppState>,
    kind: String,
) -> Result<modrinth::ModrinthTags, String> {
    modrinth::tags(&state.client, &[&kind])
        .await
        .map_err(|e| e.to_string())
}

/// Версии проекта Modrinth (для выбора конкретной версии мода).
#[tauri::command]
async fn modrinth_project_versions_command(
    state: State<'_, AppState>,
    project_id: String,
    game_version: Option<String>,
    loader: Option<String>,
) -> Result<Vec<modrinth::ModrinthVersion>, String> {
    modrinth::project_versions(
        &state.client,
        &project_id,
        game_version.as_deref(),
        loader.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())
}

/// Версия Modrinth по id.
#[tauri::command]
async fn modrinth_version_command(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<modrinth::ModrinthVersion, String> {
    modrinth::version_by_id(&state.client, &version_id)
        .await
        .map_err(|e| e.to_string())
}

/// Полный проект Modrinth (описание, галерея скриншотов) — страница сборки.
#[tauri::command]
async fn modrinth_project_command(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<modrinth::ModrinthProject, String> {
    modrinth::project_by_id(&state.client, &project_id)
        .await
        .map_err(|e| e.to_string())
}

/// Первичный файл версии (primary, иначе первый).
fn primary_file(version: &modrinth::ModrinthVersion) -> Result<&modrinth::ModrinthFile, String> {
    version
        .files
        .iter()
        .find(|f| f.primary == Some(true))
        .or_else(|| version.files.first())
        .ok_or_else(|| format!("У версии {} нет файлов", version.name))
}

/// Устанавливает файл из Modrinth в папку активной версии сборки
/// (mods/ · resourcepacks/ · shaderpacks/ · datapacks/ для датапаков — в
/// saves/<world>/datapacks/), с проверкой sha1 и трекингом для обновлений.
#[tauri::command]
async fn modrinth_install_mod_command(
    app: AppHandle,
    state: State<'_, AppState>,
    pack_id: String,
    version_id: String,
    folder: String,
    world: Option<String>,
) -> Result<(modrinth::TrackedMod, u32), String> {
    let pack = resolve_pack(Some(pack_id.clone()))?;
    ensure_unlocked(&pack.id)?;
    let version = modrinth::version_by_id(&state.client, &version_id)
        .await
        .map_err(|e| e.to_string())?;
    let file = primary_file(&version)?;
    let game_dir = config::active_game_dir(&pack.id).map_err(|e| e.to_string())?;
    let target_dir = match folder.as_str() {
        "mods" => {
            if !file.filename.to_ascii_lowercase().ends_with(".jar") {
                return Err("Моды Modrinth ставятся файлами .jar".into());
            }
            game_dir.join("mods")
        }
        "resourcepacks" | "shaderpacks" | "datapacks" => {
            if !file.filename.to_ascii_lowercase().ends_with(".zip") {
                return Err(format!(
                    "В папку {folder}/ ставятся файлы .zip (выбранный файл — .{} )",
                    file.filename.rsplit('.').next().unwrap_or("?")
                ));
            }
            if folder == "datapacks" {
                let w = world.as_deref().ok_or("Для датапака нужно выбрать мир")?;
                if w.is_empty() || w.contains('/') || w.contains('\\') || w.starts_with('.') {
                    return Err("Некорректное имя мира".into());
                }
                game_dir.join("saves").join(w).join("datapacks")
            } else {
                game_dir.join(&folder)
            }
        }
        other => return Err(format!("Неизвестная папка: {other}")),
    };
    // При установке версии уже установленного проекта (та же сборка) — заменяем,
    // удаляя старый файл этого проекта, а не добавляем рядом дубль.
    let tracked = modrinth::tracked_mods(&pack.id);
    for t in tracked {
        if t.project_id == version.project_id && t.folder == folder {
            for p in [target_dir.join(&t.file_name), target_dir.join(format!("{}.disabled", t.file_name))] {
                let _ = std::fs::remove_file(&p);
            }
        }
    }
    // Поверх существующего файла не перезаписываем: сначала удалите старый
    // или используйте обновление (modrinth_update_mod_command).
    let (file_name, _) = modrinth::download_file(&state.client, file, &target_dir)
        .await
        .map_err(|e| e.to_string())?;
    let tracked = modrinth::TrackedMod {
        file_name,
        folder: folder.clone(),
        world: if folder == "datapacks" { world.clone() } else { None },
        version_id: version.id.clone(),
        project_id: version.project_id.clone(),
        sha1: file
            .hashes
            .get("sha1")
            .cloned()
            .unwrap_or_default(),
        game_version: version.game_versions.first().cloned().unwrap_or_default(),
        loader: version.loaders.first().cloned().unwrap_or_default(),
    };
    modrinth::upsert_tracked_mod(&pack.id, &tracked).map_err(|e| e.to_string())?;
    // Автодокачка required-зависимостей (до 3 уровней), если их нет.
    let deps_installed = install_modrinth_dependencies(
        &state.client,
        &pack.id,
        &folder,
        world.as_deref(),
        &target_dir,
        &version,
    )
    .await?;
    // Показываем файл в списке (событие для обновления UI).
    let _ = app.emit("mods-changed", ());
    Ok((tracked, deps_installed))
}

/// Сколько уровней required-зависимостей Modrinth доустанавливаем автоматически.
const MODRINTH_DEP_MAX_DEPTH: u32 = 3;

/// Автодокачка required-зависимостей Modrinth-версии: для каждой зависимости
/// (по project_id, при версии — через version_id) выбирается версия под версию
/// Minecraft/лоадер сборки, скачивается в ту же папку и трекается.
/// Уже отслеживаемые проекты и циклы (по project_id) пропускаются.
/// Возвращает число установленных файлов.
async fn install_modrinth_dependencies(
    client: &reqwest::Client,
    pack_id: &str,
    folder: &str,
    world: Option<&str>,
    target_dir: &std::path::Path,
    root: &modrinth::ModrinthVersion,
) -> Result<u32, String> {
    let mut installed = 0u32;
    let tracked = modrinth::tracked_mods(pack_id);
    let mut visited = HashSet::new();
    visited.insert(root.project_id.clone());
    let mut stack: Vec<(modrinth::ModrinthVersion, u32)> = vec![(root.clone(), 0)];
    while let Some((version, depth)) = stack.pop() {
        if depth >= MODRINTH_DEP_MAX_DEPTH {
            continue;
        }
        for dep in &version.dependencies {
            if dep.dependency_type != "required" {
                continue;
            }
            let Some(project_id) = dep.project_id.clone() else {
                continue;
            };
            if !visited.insert(project_id.clone()) {
                continue;
            }
            if tracked.iter().any(|t| t.project_id == project_id) {
                continue;
            }
            let dep_version = if let Some(version_id) = &dep.version_id {
                modrinth::version_by_id(client, version_id)
                    .await
                    .map_err(|e| format!("Не удалось получить зависимость {project_id} Modrinth: {e}"))?
            } else {
                let mut candidates = modrinth::project_versions(
                    client,
                    &project_id,
                    version.game_versions.first().map(String::as_str),
                    version.loaders.first().map(String::as_str),
                )
                .await
                .map_err(|e| format!("Не удалось получить версии зависимости {project_id}: {e}"))?;
                if candidates.is_empty() {
                    candidates = modrinth::project_versions(client, &project_id, None, None)
                        .await
                        .map_err(|e| format!("Не удалось получить версии зависимости {project_id}: {e}"))?;
                }
                let Some(first) = candidates.into_iter().next() else {
                    continue;
                };
                first
            };
            let Some(dep_file) = dep_version
                .files
                .iter()
                .find(|f| f.primary == Some(true))
                .or_else(|| dep_version.files.first())
            else {
                continue;
            };
            let (file_name, _) = modrinth::download_file(client, dep_file, target_dir)
                .await
                .map_err(|e| format!("Зависимость {project_id}: {e}"))?;
            let tracked_entry = modrinth::TrackedMod {
                file_name,
                folder: folder.to_string(),
                world: if folder == "datapacks" { world.map(str::to_string) } else { None },
                version_id: dep_version.id.clone(),
                project_id: project_id.clone(),
                sha1: dep_file.hashes.get("sha1").cloned().unwrap_or_default(),
                game_version: dep_version.game_versions.first().cloned().unwrap_or_default(),
                loader: dep_version.loaders.first().cloned().unwrap_or_default(),
            };
            modrinth::upsert_tracked_mod(pack_id, &tracked_entry).map_err(|e| e.to_string())?;
            installed += 1;
            stack.push((dep_version, depth + 1));
        }
    }
    Ok(installed)
}

/// Поиск на CurseForge по классу (моды/ресурспаки/шейдеры/сборки).
#[tauri::command]
async fn curseforge_search_command(
    state: State<'_, AppState>,
    query: String,
    class_id: u32,
    category_id: Option<u32>,
    game_version: Option<String>,
    sort: Option<String>,
) -> Result<Vec<curseforge::CurseSearchHit>, String> {
    curseforge::search(
        &state.client,
        &query,
        class_id,
        category_id,
        game_version.as_deref(),
        sort.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())
}

/// Категории класса проектов CurseForge (для фильтра поиска).
#[tauri::command]
async fn curseforge_categories_command(
    state: State<'_, AppState>,
    class_id: u32,
) -> Result<Vec<curseforge::CurseCategory>, String> {
    curseforge::categories(&state.client, class_id)
        .await
        .map_err(|e| e.to_string())
}

/// Файлы проекта CurseForge (для выбора версии сборки).
#[tauri::command]
async fn curseforge_modpack_files_command(
    state: State<'_, AppState>,
    project_id: u32,
) -> Result<Vec<curseforge::CursePackFile>, String> {
    curseforge::pack_files(&state.client, project_id)
        .await
        .map_err(|e| e.to_string())
}

/// Полное описание проекта CurseForge (деталка сборки: описание/скриншоты).
#[tauri::command]
async fn curseforge_project_detail_command(
    state: State<'_, AppState>,
    project_id: u32,
) -> Result<curseforge::CurseProjectDetail, String> {
    curseforge::project_detail(&state.client, project_id)
        .await
        .map_err(|e| e.to_string())
}

/// Подходящий файл проекта CurseForge (последний под версию Minecraft сборки,
/// либо просто последний) — готов к установке.
#[tauri::command]
async fn curseforge_latest_file_command(
    state: State<'_, AppState>,
    pack_id: String,
    project_id: u32,
) -> Result<curseforge::CurseFile, String> {
    let pack = resolve_pack(Some(pack_id))?;
    let mc = config::active_version(&pack.id)
        .ok()
        .and_then(|v| mrpack::read_version_index(&pack.id, &v))
        .and_then(|idx| idx.dependencies.get("minecraft").cloned());
    curseforge::latest_file(&state.client, project_id, mc.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// Скачивает файл CurseForge в папку сборки (mods/resourcepacks/shaderpacks)
/// с проверкой sha1 и автодокачкой required-зависимостей.
#[tauri::command]
async fn curseforge_install_command(
    app: AppHandle,
    state: State<'_, AppState>,
    pack_id: String,
    file: curseforge::CurseFile,
    folder: String,
    title: Option<String>,
    icon: Option<String>,
) -> Result<curseforge::InstallResult, String> {
    let pack = resolve_pack(Some(pack_id))?;
    let game_dir = config::active_game_dir(&pack.id).map_err(|e| e.to_string())?;
    let target_dir = match folder.as_str() {
        "mods" | "resourcepacks" | "shaderpacks" => game_dir.join(&folder),
        other => return Err(format!("Неизвестная папка: {other}")),
    };
    let name = curseforge::download_to(&state.client, &file, &target_dir)
        .await
        .map_err(|e| e.to_string())?;
    // Трекинг до установки зависимостей: файл уже на диске, и даже если
    // автодокачка required-зависимостей упадёт, в списке файлов будет мета/иконка
    // проекта CurseForge (иначе плашка и иконка теряются).
    curseforge::upsert_tracked(
        &pack.id,
        &curseforge::CurseTracked {
            file_name: name.clone(),
            folder: folder.clone(),
            project_id: file.project_id,
            title: title.unwrap_or_default(),
            icon: icon.unwrap_or_default(),
        },
    );
    // Автодокачка required-зависимостей (до FileDependency::MAX_DEPTH уровней, без циклов).
    let deps_installed = curseforge::install_dependencies(
        &state.client,
        file.project_id,
        file.file_id,
        if file.game_version.is_empty() {
            None
        } else {
            Some(&file.game_version)
        },
        &target_dir,
    )
    .await
    .map_err(|e| format!("Зависимости CurseForge: {e}"))?;
    let _ = app.emit("mods-changed", ());
    Ok(curseforge::InstallResult {
        name,
        deps_installed,
    })
}

/// Задан ли API-ключ CurseForge (для подсказки в UI, сам ключ не возвращаем).
#[tauri::command]
fn curseforge_key_configured_command() -> bool {
    curseforge::api_key_from_cfg().is_some()
}

/// Скачивает и устанавливает сборку CurseForge как отдельную сборку
/// (id = `cf-<projectId>`). Повторный вызов с той же версией — обновление.
#[tauri::command]
async fn curseforge_install_pack_command(
    app: AppHandle,
    state: State<'_, AppState>,
    project_id: u32,
    file_id: u32,
) -> Result<PackDescriptor, String> {
    let project = curseforge::project(&state.client, project_id)
        .await
        .map_err(|e| e.to_string())?;
    let pack_id = format!("cf-{project_id}");
    let existing = config::find_pack(&pack_id).map_err(|e| e.to_string())?;
    if existing.is_none() {
        config::add_user_pack(
            &pack_id,
            &project.name,
            &format!("https://www.curseforge.com/minecraft/modpacks/{project_id}"),
            "local",
            None,
            None,
        )
        .map_err(|e| e.to_string())?;
    }
    curseforge::install_modpack(&app, &state.client, &pack_id, project_id, file_id)
        .await
        .map_err(|e| e.to_string())?;
    // Сборка с CurseForge — управляемая по умолчанию (правки заблокированы,
    // пока пользователь не «отвяжет» её).
    let _ = config::set_pack_locked(&pack_id, true);
    if let Some(logo_url) = &project.logo_url {
        let icon_path = config::pack_dir(&pack_id)
            .map_err(|e| e.to_string())?
            .join("icon.png");
        let _ = modrinth::download_icon(&state.client, logo_url, &icon_path).await;
    }
    let icon = config::pack_icon_path(&pack_id);
    let (name, url) = match existing {
        Some(p) => (p.name, p.url),
        None => (project.name, format!("https://www.curseforge.com/minecraft/modpacks/{project_id}")),
    };
    Ok(PackDescriptor {
        id: pack_id.clone(),
        name,
        url,
        builtin: false,
        kind: "local".into(),
        author: None,
        boosty_blog: None,
        min_ram_mb: None,
        icon,
        banner: config::pack_banner_path(&pack_id),
    })
}

/// Доступное обновление установленного из Modrinth файла.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModUpdate {
    /// Имя файла (в mods/ / resourcepacks/ / shaderpacks/ / datapacks/).
    pub file_name: String,
    /// Папка игры, где лежит файл.
    pub folder: String,
    /// Новая версия Modrinth.
    pub new_version: modrinth::ModrinthVersion,
}

/// Проверяет обновления установленных из Modrinth модов активной версии.
///
/// Проверяются НЕ только отслеживаемые моды (`tracked_mods`), а ВСЕ файлы в
/// папках mods/resourcepacks/shaderpacks (в т.ч. пришедшие со сборкой из .mrpack).
/// Так у ресурсов, скачанных из пака, тоже появляется плашка обновления.
#[tauri::command]
async fn modrinth_check_updates_command(
    state: State<'_, AppState>,
    pack_id: String,
) -> Result<Vec<ModUpdate>, String> {
    let pack = resolve_pack(Some(pack_id.clone()))?;
    let game_dir = config::active_game_dir(&pack.id).map_err(|e| e.to_string())?;
    // Чистим осиротевшие записи трекинга (файла на диске уже нет) — иначе они
    // считаются «устаревшими» и ошибочно показываются как доступные обновления,
    // а при обновлении падают с «файл не найден».
    let _ = modrinth::prune_missing_tracked(&pack.id, &game_dir);
    let tracked = modrinth::tracked_mods(&pack.id);

    // name -> (folder, local sha1, id версии обновления на текущий момент).
    let mut files: HashMap<String, (String, String, Option<String>)> = HashMap::new();
    for t in &tracked {
        if !t.sha1.is_empty() {
            files
                .entry(t.file_name.clone())
                .or_insert_with(|| {
                    (t.folder.clone(), t.sha1.clone(), Some(t.version_id.clone()))
                });
        }
    }
    // Остальные файлы сборки (без записи отслеживания) — хэшируем на лету.
    for folder in ["mods", "resourcepacks", "shaderpacks"] {
        let dir = game_dir.join(folder);
        if !dir.exists() {
            continue;
        }
        let Ok(rd) = std::fs::read_dir(&dir) else { continue };
        for e in rd.flatten() {
            let Ok(meta) = e.metadata() else { continue };
            if meta.is_dir() {
                continue;
            }
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            // На диске мод может лежать с суффиксом ".disabled" (отключённый).
            // Проверяем расширение по базовому имени, но ключом храним реальное
            // имя файла, чтобы обновление находило файл на диске.
            let base = name.strip_suffix(".disabled").unwrap_or(&name);
            if !(base.ends_with(".jar") || base.ends_with(".zip")) {
                continue;
            }
            if files.contains_key(&name) {
                continue;
            }
            if let Ok(sha) = mrpack::compute_sha1(&e.path()) {
                files.insert(name, (folder.to_string(), sha, None));
            }
        }
    }
    if files.is_empty() {
        return Ok(Vec::new());
    }
    // sha1 -> имя файла (для привязки ответа Modrinth по хэшу).
    let mut by_sha: HashMap<String, String> = HashMap::new();
    let mut hashes: Vec<String> = Vec::with_capacity(files.len());
    for (n, (_, sha, _)) in &files {
        by_sha.insert(sha.clone(), n.clone());
        hashes.push(sha.clone());
    }
    // Пустые game_versions/loaders → каждый хэш резолвится в свою последнюю версию.
    // Фильтруем по версии MC и загрузчику сборки, чтобы не предлагать обновление
    // под более свежую MC (например 26.x при сборке на 1.21.x).
    let (mc, loader) = active_mc_loader(&pack.id);
    let gv: Vec<String> = mc.into_iter().collect();
    let ld: Vec<String> = loader.into_iter().collect();
    let updates = modrinth::check_updates(&state.client, &hashes, &gv, &ld)
        .await
        .map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for (sha, new) in updates {
        let Some(name) = by_sha.get(&sha) else {
            continue;
        };
        let Some((folder, local_sha, tracked_vid)) = files.get(name) else {
            continue;
        };
        // Уже актуальная версия — обновление не показываем.
        let is_latest = new.files.iter().any(|f| {
            f.hashes
                .get("sha1")
                .is_some_and(|s| s.eq_ignore_ascii_case(local_sha))
        });
        let is_same_as_tracked = tracked_vid.as_ref().is_some_and(|vid| vid == &new.id);
        if is_latest || is_same_as_tracked {
            continue;
        }
        out.push(ModUpdate {
            file_name: name.clone(),
            folder: folder.clone(),
            new_version: new,
        });
    }
    Ok(out)
}

/// sha1 установленного файла мода по project_id (для отметки «установлена» в списке версий).
/// Ищет файл по отслеживанию; если на диске файл переименован в `.disabled` — берёт его.
#[tauri::command]
fn installed_mod_sha1_command(
    pack_id: String,
    project_id: String,
) -> Result<Option<String>, String> {
    let pack = resolve_pack(Some(pack_id))?;
    let game_dir = config::active_game_dir(&pack.id).map_err(|e| e.to_string())?;
    let tracked = modrinth::tracked_mods(&pack.id);
    for t in tracked.iter().filter(|t| t.project_id == project_id) {
        let dir = if t.folder == "datapacks" {
            game_dir
                .join("saves")
                .join(t.world.as_deref().unwrap_or(""))
                .join("datapacks")
        } else {
            game_dir.join(&t.folder)
        };
        let mut path = dir.join(&t.file_name);
        if !path.exists() {
            path = dir.join(format!("{}.disabled", t.file_name));
        }
        if path.exists() {
            if let Ok(sha) = mrpack::compute_sha1(&path) {
                return Ok(Some(sha.to_lowercase()));
            }
        }
    }
    Ok(None)
}

/// Версия Minecraft и загрузчик активной версии сборки (для фильтрации обновлений
/// по совместимости — чтобы не обновлять мод под более новую MC, чем в сборке).
fn active_mc_loader(pack_id: &str) -> (Option<String>, Option<String>) {
    let idx = config::active_version(pack_id)
        .ok()
        .and_then(|v| mrpack::read_version_index(pack_id, &v));
    let Some(idx) = idx else {
        return (None, None);
    };
    let mc = idx.dependencies.get("minecraft").cloned();
    let loader = ["fabric-loader", "forge", "neoforge", "quilt"]
        .iter()
        .find(|k| idx.dependencies.contains_key(**k))
        .map(|k| k.replace("-loader", ""));
    (mc, loader)
}

/// Обновляет один файл из Modrinth до последней подходящей версии
/// (файл в своей папке перезаписывается). Работает и для файлов, пришедших
/// со сборкой из .mrpack — их ищем по имени среди установленных.
#[tauri::command]
async fn modrinth_update_mod_command(
    state: State<'_, AppState>,
    pack_id: String,
    file_name: String,
) -> Result<modrinth::TrackedMod, String> {
    let pack = resolve_pack(Some(pack_id.clone()))?;
    ensure_unlocked(&pack.id)?;
    let game_dir = config::active_game_dir(&pack.id).map_err(|e| e.to_string())?;
    let _ = modrinth::prune_missing_tracked(&pack.id, &game_dir);
    let tracked = modrinth::tracked_mods(&pack.id);
    let tracked_entry = tracked.iter().find(|t| t.file_name == file_name);

    // Запись отслеживания (мир/папка) либо обыскиваем папки.
    let (folder, world, gap_loader): (String, Option<String>, Option<String>) =
        match tracked_entry {
            Some(t) => (t.folder.clone(), t.world.clone(), None),
            None => {
                let found = ["mods", "resourcepacks", "shaderpacks"]
                    .iter()
                    .find(|f| game_dir.join(f).join(&file_name).exists());
                match found {
                    Some(f) => ((*f).to_string(), None, None),
                    None => {
                        return Err(format!(
                            "Файл {file_name} не найден в папках игры (не из Modrinth)"
                        ))
                    }
                }
            }
        };

    let dir = if folder == "datapacks" {
        game_dir
            .join("saves")
            .join(world.as_deref().unwrap_or(""))
            .join("datapacks")
    } else {
        game_dir.join(&folder)
    };
    let path = dir.join(&file_name);
    if !path.exists() {
        return Err(format!("Файл {file_name} не найден в папке {folder}/"));
    }
    let local_sha = mrpack::compute_sha1(&path).map_err(|e| e.to_string())?;
    let hashes: Vec<String> = vec![local_sha.clone()];
    let (mc, loader) = active_mc_loader(&pack.id);
    let gv: Vec<String> = mc.into_iter().collect();
    let ld: Vec<String> = loader.into_iter().collect();
    let updates = modrinth::check_updates(&state.client, &hashes, &gv, &ld)
        .await
        .map_err(|e| e.to_string())?;
    let Some(new) = updates.get(&local_sha) else {
        return Err("Для этого файла нет обновлений".into());
    };
    let file = primary_file(new)?;
    modrinth::update_file_to(&state.client, file, &path)
        .await
        .map_err(|e| e.to_string())?;
    let updated = modrinth::TrackedMod {
        file_name: file_name.clone(),
        folder: folder.clone(),
        world: world.clone(),
        version_id: new.id.clone(),
        project_id: new.project_id.clone(),
        sha1: file.hashes.get("sha1").cloned().unwrap_or_default(),
        game_version: new
            .game_versions
            .first()
            .cloned()
            .or_else(|| gap_loader.clone())
            .unwrap_or_default(),
        loader: new
            .loaders
            .first()
            .cloned()
            .unwrap_or_else(|| gap_loader.unwrap_or_default()),
    };
    modrinth::upsert_tracked_mod(&pack.id, &updated).map_err(|e| e.to_string())?;
    Ok(updated)
}

/// Удаляет установленный из Modrinth файл (из своей папки) и его трекинг.
#[tauri::command]
fn modrinth_remove_mod_command(pack_id: String, file_name: String) -> Result<(), String> {
    let pack = resolve_pack(Some(pack_id.clone()))?;
    ensure_unlocked(&pack.id)?;
    let game_dir = config::active_game_dir(&pack.id).map_err(|e| e.to_string())?;
    let tracked = modrinth::tracked_mods(&pack.id);
    let entry = tracked.iter().find(|t| t.file_name == file_name);
    let folder = entry.map(|t| t.folder.clone()).unwrap_or_else(|| "mods".to_string());
    let dir = if folder == "datapacks" {
        game_dir
            .join("saves")
            .join(entry.and_then(|t| t.world.as_deref()).unwrap_or(""))
            .join("datapacks")
    } else {
        game_dir.join(&folder)
    };
    // Разрешаем удалять и выключенный (.disabled).
    let candidates = [dir.join(&file_name), dir.join(format!("{file_name}.disabled"))];
    let mut removed = false;
    for c in candidates {
        if c.exists() {
            std::fs::remove_file(&c).map_err(|e| e.to_string())?;
            removed = true;
        }
    }
    if !removed {
        return Err(format!("Файл {file_name} не найден в папке {folder}/"));
    }
    modrinth::remove_tracked_mod(
        &pack.id,
        &file_name,
        &folder,
        entry.and_then(|t| t.world.as_deref()),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Устанавливает иконку сборки (копирует выбранный PNG в `packs/<id>/icon.png`).
#[tauri::command]
fn set_pack_icon_command(pack_id: String, path: String) -> Result<(), String> {
    let src = std::path::PathBuf::from(&path);
    if !src.is_file() {
        return Err(format!("Файл не найден: {path}"));
    }
    let dest = config::pack_dir(&pack_id)
        .map_err(|e| e.to_string())?
        .join("icon.png");
    std::fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::copy(&src, &dest).map_err(|e| e.to_string())?;
    Ok(())
}

/// Устанавливает баннер сборки (копирует выбранный PNG в `packs/<id>/banner.png`).
#[tauri::command]
fn set_pack_banner_command(pack_id: String, path: String) -> Result<(), String> {
    let src = std::path::PathBuf::from(&path);
    if !src.is_file() {
        return Err(format!("Файл не найден: {path}"));
    }
    let dest = config::pack_dir(&pack_id)
        .map_err(|e| e.to_string())?
        .join("banner.png");
    std::fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::copy(&src, &dest).map_err(|e| e.to_string())?;
    Ok(())
}

/// Меняет название сборки (встроенной — переопределение, пользовательской — packs.json).
#[tauri::command]
fn set_pack_name_command(pack_id: String, name: String) -> Result<(), String> {
    config::set_pack_name(&pack_id, &name).map_err(|e| e.to_string())
}

/// Id сборок в порядке убывания времени последнего запуска.
#[tauri::command]
fn recent_packs_command() -> Vec<String> {
    config::recent_pack_ids()
}

/// Скачивает иконку сборки в `packs/<id>/icon.png` (если её ещё нет):
/// — авторские сборки (GitHub): `icon.png` из корня репозитория автора;
/// — сборки с Modrinth (`mrn-<id>`): иконка проекта;
/// — сборки с CurseForge (`cf-<id>`): логотип проекта.
/// Возвращает, нашлась ли иконка.
#[tauri::command]
async fn fetch_pack_icon_command(
    state: State<'_, AppState>,
    pack_id: String,
) -> Result<bool, String> {
    let pack = config::find_pack(&pack_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Сборка не найдена".to_string())?;
    let dest = config::pack_dir(&pack_id)
        .map_err(|e| e.to_string())?
        .join("icon.png");
    if dest.exists() {
        return Ok(true);
    }
    if let Some((owner, repo)) = parse_github_repo(&pack) {
        let url = format!("https://raw.githubusercontent.com/{owner}/{repo}/HEAD/icon.png");
        if let Ok(resp) = state.client.get(&url).send().await {
            if resp.status().is_success() {
                let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                std::fs::write(&dest, &bytes).map_err(|e| e.to_string())?;
                return Ok(true);
            }
        }
        return Ok(false);
    }
    let icon_url = if let Some(pid) = pack_id.strip_prefix("mrn-") {
        modrinth::project_by_id(&state.client, pid)
            .await
            .map_err(|e| e.to_string())?
            .icon_url
    } else if let Some(pid) = pack_id.strip_prefix("cf-") {
        let id: u32 = pid.parse::<u32>().map_err(|e| e.to_string())?;
        curseforge::project(&state.client, id)
            .await
            .map_err(|e| e.to_string())?
            .logo_url
    } else {
        None
    };
    let Some(icon_url) = icon_url else {
        return Ok(false);
    };
    modrinth::download_icon(&state.client, &icon_url, &dest)
        .await
        .map_err(|e| e.to_string())?;
    Ok(true)
}

/// Скачивает и устанавливает модпак с Modrinth как отдельную сборку
/// (id = `mrn-<projectId>`). Повторный вызов с той же версией — обновление.
#[tauri::command]
async fn modrinth_install_pack_command(
    app: AppHandle,
    state: State<'_, AppState>,
    version_id: String,
) -> Result<PackDescriptor, String> {
    let version = modrinth::version_by_id(&state.client, &version_id)
        .await
        .map_err(|e| e.to_string())?;
    let project = modrinth::project_by_id(&state.client, &version.project_id)
        .await
        .map_err(|e| e.to_string())?;
    let mrpack = version
        .files
        .iter()
        .find(|f| f.filename.to_ascii_lowercase().ends_with(".mrpack"))
        .ok_or_else(|| "У версии модпака нет файла .mrpack".to_string())?;
    let pack_id = format!("mrn-{}", version.project_id);
    if let Some(existing) = config::find_pack(&pack_id).map_err(|e| e.to_string())? {
        if existing.boosty_blog.is_some() {
            return Err("Сборка с этим Modrinth-проектом уже добавлена".into());
        }
        // Уже добавлена — просто переустанавливаем/обновляем.
        mrpack::install_mrpack(app, &state.client, &pack_id, &mrpack.url, Some(&version.version_number))
            .await
            .map_err(|e| e.to_string())?;
        let _ = config::set_pack_locked(&pack_id, true);
        if let Some(icon_url) = &project.icon_url {
            let icon_path = config::pack_dir(&pack_id)
                .map_err(|e| e.to_string())?
                .join("icon.png");
            let _ = modrinth::download_icon(&state.client, icon_url, &icon_path).await;
        }
        let icon = config::pack_icon_path(&pack_id);
        let banner = config::pack_banner_path(&pack_id);
        return Ok(PackDescriptor {
            id: pack_id,
            name: existing.name,
            url: existing.url,
            builtin: false,
            kind: "local".into(),
            author: None,
            boosty_blog: None,
            min_ram_mb: None,
            icon,
            banner,
        })
    }
    config::add_user_pack(
        &pack_id,
        &project.title,
        &mrpack.url,
        "local",
        None,
        None,
    )
    .map_err(|e| e.to_string())?;
    mrpack::install_mrpack(
        app,
        &state.client,
        &pack_id,
        &mrpack.url,
        Some(&version.version_number),
    )
    .await
    .map_err(|e| e.to_string())?;
    // Сборка с Modrinth — управляемая по умолчанию (правки заблокированы,
    // пока пользователь не «отвяжет» её).
    let _ = config::set_pack_locked(&pack_id, true);
    if let Some(icon_url) = &project.icon_url {
        let icon_path = config::pack_dir(&pack_id)
            .map_err(|e| e.to_string())?
            .join("icon.png");
        let _ = modrinth::download_icon(&state.client, icon_url, &icon_path).await;
    }
    let icon = config::pack_icon_path(&pack_id);
    let banner = config::pack_banner_path(&pack_id);
    Ok(PackDescriptor {
        id: pack_id,
        name: project.title.clone(),
        url: mrpack.url.clone(),
        builtin: false,
        kind: "local".into(),
        author: None,
        boosty_blog: None,
        min_ram_mb: None,
        icon,
        banner,
    })
}

/// Поддерживаемые загрузчики для создания своей сборки.
const LOCAL_LOADERS: &[&str] = &["vanilla", "fabric", "quilt", "forge", "neoforge"];

/// Префикс версий NeoForge для заданной версии Minecraft (1.21.4 → "21.4.").
fn neoforge_prefix(mc: &str) -> Option<String> {
    let rest = mc.strip_prefix("1.")?;
    let mut parts = rest.split('.');
    let a = parts.next()?;
    let b = parts.next()?;
    Some(format!("{a}.{b}."))
}

/// Числовое сравнение версий ("21.4.9" < "21.4.10"; суффиксы -beta/-alpha игнорируются).
fn version_numeric(v: &str) -> Vec<i64> {
    let base = v.split('-').next().unwrap_or(v);
    base.split('.')
        .filter_map(|s| s.parse::<i64>().ok())
        .collect()
}

/// Копирует выбранный пользователем файл (иконку/баннер) в папку сборки.
fn copy_pack_asset(src: Option<&str>, dest: &std::path::Path) -> Result<(), String> {
    let Some(src) = src else { return Ok(()); };
    let src_path = std::path::PathBuf::from(src);
    if !src_path.is_file() {
        return Err(format!("Файл не найден: {src}"));
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::copy(&src_path, dest).map_err(|e| e.to_string())?;
    Ok(())
}

/// Определяет версию загрузчика под версию Minecraft. Возвращает (ключ зависимости
/// в .mono-index.json, версию загрузчика) или None для ванили.
async fn meta_json(client: &reqwest::Client, url: &str) -> Result<serde_json::Value, String> {
    client
        .get(url)
        .header("User-Agent", "mono-launcher")
        .send()
        .await
        .map_err(|e| format!("Не удалось получить метаданные загрузчика: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Сервис загрузчика вернул ошибку: {e}"))?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Не удалось прочитать метаданные загрузчика: {e}"))
}

/// Версии NeoForge под версию Minecraft (убыванию), из maven-метаданных.
async fn neoforge_versions(client: &reqwest::Client, mc: &str) -> Result<Vec<String>, String> {
    let url = "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";
    let prefix = neoforge_prefix(mc)
        .ok_or_else(|| format!("Не удалось определить версию NeoForge для Minecraft {mc}"))?;
    let text = client
        .get(url)
        .header("User-Agent", "mono-launcher")
        .send()
        .await
        .map_err(|e| format!("Не удалось получить версии NeoForge: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Maven NeoForge вернул ошибку: {e}"))?
        .text()
        .await
        .map_err(|e| format!("Не удалось прочитать версии NeoForge: {e}"))?;
    let mut versions: Vec<String> = Vec::new();
    for part in text.split("<version>").skip(1) {
        let Some(end) = part.find("</version>") else {
            continue;
        };
        let v = &part[..end];
        if v.starts_with(&prefix) {
            versions.push(v.to_string());
        }
    }
    versions.sort_by(|a, b| {
        version_numeric(b)
            .cmp(&version_numeric(a))
            .then_with(|| b.cmp(a))
    });
    versions.dedup();
    Ok(versions)
}

/// Версии мода вдлоадера (fabric/quilt — список из их meta-API).
async fn loader_versions_from_meta(
    client: &reqwest::Client,
    url: &str,
    mc: &str,
    loader: &str,
) -> Result<Vec<String>, String> {
    let resp = meta_json(client, url).await?;
    let arr = resp
        .as_array()
        .ok_or_else(|| format!("Здесь версии загрузчика {loader} недоступны для Minecraft {mc}"))?;
    // Стабильные сборки идут первыми, затем прочие.
    let mut stable: Vec<String> = Vec::new();
    let mut others: Vec<String> = Vec::new();
    for v in arr {
        let Some(ver) = v["loader"]["version"].as_str() else {
            continue;
        };
        if v["loader"]["stable"].as_bool().unwrap_or(false) {
            if !stable.contains(&ver.to_string()) {
                stable.push(ver.to_string());
            }
        } else if !others.contains(&ver.to_string()) {
            others.push(ver.to_string());
        }
    }
    stable.extend(others);
    Ok(stable)
}

/// Определяет версию загрузчика под версию Minecraft. `requested` — выбранная
/// пользователем версия (пустая строка → последняя). Возвращает (ключ зависимости
/// в .mono-index.json, версию загрузчика) или None для ванили.
async fn resolve_loader_version(
    client: &reqwest::Client,
    loader: &str,
    mc: &str,
    requested: Option<&str>,
) -> Result<Option<(String, String)>, String> {
    match loader {
        "vanilla" => Ok(None),
        "fabric" => {
            let url = format!("https://meta.fabricmc.net/v2/versions/loader/{mc}");
            let list = loader_versions_from_meta(client, &url, mc, "fabric").await?;
            let version = if let Some(r) = requested {
                list.iter()
                    .find(|v| v == &r)
                    .cloned()
                    .ok_or_else(|| format!("Версия fabric {r} не найдена для Minecraft {mc}"))?
            } else {
                list.first().cloned().ok_or_else(|| {
                    format!("Загрузчик fabric не поддерживает Minecraft {mc}")
                })?
            };
            Ok(Some(("fabric-loader".into(), version)))
        }
        "quilt" => {
            let url = format!("https://meta.quiltmc.org/v3/versions/loader/{mc}");
            let list = loader_versions_from_meta(client, &url, mc, "quilt").await?;
            let version = if let Some(r) = requested {
                list.iter()
                    .find(|v| v == &r)
                    .cloned()
                    .ok_or_else(|| format!("Версия quilt {r} не найдена для Minecraft {mc}"))?
            } else {
                list.first().cloned().ok_or_else(|| {
                    format!("Загрузчик quilt не поддерживает Minecraft {mc}")
                })?
            };
            Ok(Some(("quilt-loader".into(), version)))
        }
        "forge" => {
            // У пользователя — только номер сборки; итоговая версия = "{mc}-{build}".
            if let Some(r) = requested.map(str::trim).filter(|r| !r.is_empty()) {
                return Ok(Some(("forge".into(), format!("{mc}-{r}"))));
            }
            let url = "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";
            let resp = meta_json(client, url).await?;
            let promos = resp
                .get("promos")
                .and_then(|p| p.as_object())
                .ok_or_else(|| format!("Здесь промо-версии forge недоступны для Minecraft {mc}"))?;
            // Приоритет: recommended → latest → голый ключ версии.
            let mut picked = None;
            for key in [
                format!("{mc}-recommended"),
                format!("{mc}-latest"),
                mc.to_string(),
            ] {
                if let Some(v) = promos.get(&key) {
                    if let Some(s) = v.as_str() {
                        if !s.is_empty() {
                            picked = Some(s.to_string());
                            break;
                        }
                    }
                }
            }
            let build = picked
                .ok_or_else(|| format!("Загрузчик forge не поддерживает Minecraft {mc}"))?;
            Ok(Some(("forge".into(), format!("{mc}-{build}"))))
        }
        "neoforge" => {
            let list = neoforge_versions(client, mc).await?;
            let version = if let Some(r) = requested.map(str::trim).filter(|r| !r.is_empty()) {
                list.iter()
                    .find(|v| v == &r)
                    .cloned()
                    .ok_or_else(|| format!("Версия neoforge {r} не найдена для Minecraft {mc}"))?
            } else {
                list.first().cloned().ok_or_else(|| {
                    format!("Загрузчик neoforge не поддерживает Minecraft {mc}")
                })?
            };
            Ok(Some(("neoforge".into(), version)))
        }
        _ => Err(format!("Загрузчик «{loader}» не поддерживается")),
    }
}

/// Доступные версии модлоадера под версию Minecraft (для выбора при создании
/// своей сборки). Пустой список — загрузчик не применим (vanilla).
#[tauri::command]
async fn local_loader_versions_command(
    state: State<'_, AppState>,
    loader: String,
    minecraft_version: String,
) -> Result<Vec<String>, String> {
    if loader == "vanilla" || minecraft_version.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mc = minecraft_version.trim().to_string();
    match loader.as_str() {
        "fabric" => {
            let url = format!("https://meta.fabricmc.net/v2/versions/loader/{mc}");
            loader_versions_from_meta(&state.client, &url, &mc, "fabric").await
        }
        "quilt" => {
            let url = format!("https://meta.quiltmc.org/v3/versions/loader/{mc}");
            loader_versions_from_meta(&state.client, &url, &mc, "quilt").await
        }
        "forge" => {
            // Промо-версии Forge: для версии есть recommended и/или latest.
            let url = "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";
            let resp = meta_json(&state.client, url).await?;
            let promos = resp
                .get("promos")
                .and_then(|p| p.as_object())
                .ok_or_else(|| format!("Здесь промо-версии forge недоступны для Minecraft {mc}"))?;
            let mut out: Vec<String> = Vec::new();
            for key in [format!("{mc}-recommended"), format!("{mc}-latest")] {
                if let Some(v) = promos.get(&key).and_then(|x| x.as_str()) {
                    if !v.is_empty() && !out.contains(&v.to_string()) {
                        out.push(v.to_string());
                    }
                }
            }
            Ok(out)
        }
        "neoforge" => neoforge_versions(&state.client, &mc).await,
        _ => Ok(Vec::new()),
    }
}

/// Создаёт свою (локальную) сборку: база Minecraft + опциональный загрузчик.
/// Сразу ставит базу (файлы игры скачаются при первом запуске).
#[tauri::command]
async fn create_local_pack_command(
    state: State<'_, AppState>,
    name: String,
    minecraft_version: String,
    loader: Option<String>,
    loader_version: Option<String>,
    icon: Option<String>,
    banner: Option<String>,
) -> Result<PackDescriptor, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Укажите название сборки".into());
    }
    let mc = minecraft_version.trim().to_string();
    if mc.is_empty() {
        return Err("Укажите версию Minecraft".into());
    }
    let loader = loader.unwrap_or_else(|| "vanilla".into());
    if !LOCAL_LOADERS.contains(&loader.as_str()) {
        return Err(format!(
            "Загрузчик «{loader}» не поддерживается для своих сборок (доступно: vanilla, fabric, quilt, forge, neoforge)"
        ));
    }
    let slug: String = name
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let slug = slug.trim_matches('-');
    let pack_id = if slug.is_empty() {
        "local-pack".into()
    } else {
        format!("local-{slug}")
    };
    if config::find_pack(&pack_id).map_err(|e| e.to_string())?.is_some() {
        return Err(format!("Сборка «{name}» уже существует (id {pack_id})"));
    }
    // Версия загрузчика: указанная пользователем или последняя подходящая.
    let requested = loader_version.as_deref().filter(|v| !v.trim().is_empty());
    let resolver = resolve_loader_version(&state.client, &loader, &mc, requested).await?;
    let mut index_deps = std::collections::HashMap::new();
    index_deps.insert("minecraft".to_string(), mc.clone());
    let version_id = match &resolver {
        Some((dep_key, lv)) => {
            index_deps.insert(dep_key.clone(), lv.clone());
            format!("{mc}-{loader}-{lv}")
        }
        None => mc.clone(),
    };
    let index = mrpack::ModrinthIndex {
        format_version: 1,
        game: "minecraft".into(),
        version_id: version_id.clone(),
        name: name.clone(),
        summary: None,
        files: Vec::new(),
        dependencies: index_deps,
    };
    let game_dir = config::version_dir(&pack_id, &version_id).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&game_dir).map_err(|e| e.to_string())?;
    std::fs::write(
        game_dir.join(".mono-index.json"),
        serde_json::to_vec_pretty(&index).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    mrpack::write_install_marker(&game_dir, &index, None).map_err(|e| e.to_string())?;
    config::set_active_version(&pack_id, &version_id).map_err(|e| e.to_string())?;
    let url = format!("local://{version_id}");
    // Иконка и баннер сборки (необязательно) — копируются в папку сборки.
    let pack_dir = config::pack_dir(&pack_id).map_err(|e| e.to_string())?;
    if icon.is_some() {
        copy_pack_asset(icon.as_deref(), &pack_dir.join("icon.png"))?;
    }
    if banner.is_some() {
        copy_pack_asset(banner.as_deref(), &pack_dir.join("banner.png"))?;
    }
    config::add_user_pack(&pack_id, &name, &url, "local", None, None).map_err(|e| e.to_string())?;
    let icon_path = config::pack_icon_path(&pack_id);
    let banner_path = config::pack_banner_path(&pack_id);
    Ok(PackDescriptor {
        id: pack_id,
        name,
        url,
        builtin: false,
        kind: "local".into(),
        author: None,
        boosty_blog: None,
        min_ram_mb: None,
        icon: icon_path,
        banner: banner_path,
    })
}

/// Меняет версию Minecraft / загрузчик / версию загрузчика у активной версии
/// уже установленной сборки. Доступно только для своих (локальных) сборок:
/// у чужих сборок из .mrpack набор версий определяется релизами и менять его
/// нельзя. Загрузчик версии (или «последняя подходящая») резолвится так же,
/// как при создании сборки. Заменяет зависимости в индексе активной версии и
/// переписывает маркер установки.
#[tauri::command]
async fn edit_pack_version_command(
    state: State<'_, AppState>,
    pack_id: String,
    minecraft_version: String,
    loader: String,
    loader_version: String,
) -> Result<(), String> {
    let pack = resolve_pack(Some(pack_id.clone()))?;
    if pack.kind != "local" {
        return Err("Версии Minecraft/загрузчика можно менять только у своих сборок".into());
    }
    let mc = minecraft_version.trim().to_string();
    if mc.is_empty() {
        return Err("Укажите версию Minecraft".into());
    }
    let loader = loader.trim().to_string();
    if !LOCAL_LOADERS.contains(&loader.as_str()) {
        return Err(format!(
            "Загрузчик «{loader}» не поддерживается для своих сборок (доступно: vanilla, fabric, quilt, forge, neoforge)"
        ));
    }
    let active = config::active_version(&pack.id).map_err(|e| e.to_string())?;
    let mut index = mrpack::read_version_index(&pack.id, &active)
        .ok_or_else(|| "Не найден индекс активной версии сборки".to_string())?;

    let requested = if loader_version.trim().is_empty() {
        None
    } else {
        Some(loader_version.trim())
    };
    let resolver = resolve_loader_version(&state.client, &loader, &mc, requested).await?;

    // Убираем прежний загрузчик и выставляем новый + версию Minecraft.
    for key in ["neoforge", "forge", "fabric-loader", "quilt-loader"] {
        index.dependencies.remove(key);
    }
    index.dependencies.insert("minecraft".into(), mc.clone());
    if let Some((dep_key, lv)) = &resolver {
        index.dependencies.insert(dep_key.clone(), lv.clone());
    }

    let game_dir = config::version_dir(&pack.id, &active).map_err(|e| e.to_string())?;
    std::fs::write(
        game_dir.join(".mono-index.json"),
        serde_json::to_vec_pretty(&index).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    mrpack::write_install_marker(&game_dir, &index, None).map_err(|e| e.to_string())?;
    Ok(())
}

/// Версия Minecraft для выбора при создании своей сборки.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McVersionInfo {
    pub id: String,
    /// "release" | "snapshot"
    pub kind: String,
}

/// Список релизных и снапшот-версий Minecraft (для выбора при создании своей
/// сборки) из официального манифеста Mojang. Сначала релизы (по убыванию),
/// затем снапшоты (по дате).
#[tauri::command]
async fn minecraft_versions_command(state: State<'_, AppState>) -> Result<Vec<McVersionInfo>, String> {
    let url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
    let resp: serde_json::Value = state
        .client
        .get(url)
        .header("User-Agent", "mono-launcher")
        .send()
        .await
        .map_err(|e| format!("Не удалось получить список версий Minecraft: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Manifest Mojang вернул ошибку: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Не удалось прочитать список версий Minecraft: {e}"))?;
    let entries = resp["versions"]
        .as_array()
        .ok_or("Пустой список версий Minecraft")?;
    // Сортируем по времени релиза (releaseTime) по убыванию, затем стабильно
    // разделяем на релизы и снапшоты, чтобы релизы шли первыми.
    let mut sorted: Vec<(&serde_json::Value, serde_json::Value)> = entries
        .iter()
        .filter(|v| matches!(v["type"].as_str(), Some("release") | Some("snapshot")))
        .map(|v| {
            let time = v["releaseTime"].as_str().unwrap_or("").to_string();
            (v, serde_json::Value::String(time))
        })
        .collect();
    sorted.sort_by(|a, b| b.1.as_str().cmp(&a.1.as_str()));
    let mut out = Vec::with_capacity(sorted.len());
    for (v, _) in sorted.iter().filter(|(v, _)| v["type"].as_str() == Some("release")) {
        if let Some(id) = v["id"].as_str() {
            out.push(McVersionInfo { id: id.to_string(), kind: "release".into() });
        }
    }
    for (v, _) in sorted.iter().filter(|(v, _)| v["type"].as_str() == Some("snapshot")) {
        if let Some(id) = v["id"].as_str() {
            out.push(McVersionInfo { id: id.to_string(), kind: "snapshot".into() });
        }
    }
    Ok(out)
}

const DEEP_LINK_SCHEME: &str = "mono";
const DEEP_LINK_PREFIX: &str = "mono://";

/// Мьютекс, чтобы параллельные deep link не добавляли одну сборку дважды.
static ADD_PACK_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

fn add_pack_lock() -> &'static tokio::sync::Mutex<()> {
    ADD_PACK_LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
}

/// Недавно обработанные deep link (защита от двойного срабатывания: одни и те же
/// аргументы приходят и в single-instance callback, и в событие плагина).
static HANDLED_LINKS: OnceLock<std::sync::Mutex<Vec<(String, Instant)>>> = OnceLock::new();

/// Разбирает deep link вида `mono://add-pack?url=<github-url>&name=<имя>&blog=<boosty-ник>`.
/// Параметры percent-encoded; возвращает (url сборки, имя, ник блога Boosty).
fn parse_deep_link(url: &str) -> Option<(String, Option<String>, Option<String>)> {
    let rest = url.strip_prefix(DEEP_LINK_PREFIX)?;
    let (path, query) = rest.split_once('?')?;
    if path != "add-pack" {
        return None;
    }
    let mut pack_url: Option<String> = None;
    let mut name: Option<String> = None;
    let mut blog: Option<String> = None;
    for pair in query.split('&') {
        let Some((key, value)) = pair.split_once('=') else {
            // Кусок без '=' (например "mono://add-pack?url=X&name") — игнорируем,
            // а не роняем разбор всей ссылки.
            continue;
        };
        match key {
            "url" => pack_url = Some(pct_decode(value)),
            "name" => name = Some(pct_decode(value)),
            "blog" => blog = Some(pct_decode(value)),
            _ => {}
        }
    }
    pack_url.map(|u| (u, name, blog))
}

/// Декодирует percent-encoding (%XX → байт).
fn pct_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                out.push(h * 16 + l);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Добавляет сборку из deep link или подтверждает уже добавленную.
/// Возвращает (описание сборки, была ли она уже добавлена).
async fn ensure_pack_from_link(
    client: &reqwest::Client,
    pack_url: &str,
    name: Option<&str>,
    blog: Option<&str>,
) -> Result<(PackDescriptor, bool), String> {
    let (owner, repo) = parse_github_repo_from_url(pack_url)
        .ok_or("Не удалось разобрать владельца/репозиторий из URL")?;
    let id = pack_id_from_repo(&owner, &repo);
    if let Some(existing) = config::find_pack(&id).map_err(|e| e.to_string())? {
        let author = repo_author(&existing.url);
        return Ok((
            PackDescriptor {
                id: existing.id,
                name: existing.name,
                url: existing.url,
                builtin: existing.builtin,
                kind: existing.kind,
                author,
                boosty_blog: existing.boosty_blog.clone(),
                min_ram_mb: existing.min_ram_mb,
                icon: existing.icon,
                banner: existing.banner,
            },
            true,
        ));
    }
    add_pack_impl(client, pack_url, name, blog)
        .await
        .map(|p| (p, false))
}

/// Последний результат добавления — на случай, если фронтенд ещё не подписался
/// на событие (первый запуск лаунчера по ссылке). Забирается командой
/// `take_pending_pack_add` и протухает через 30 секунд.
static PENDING_PACK_ADD: OnceLock<std::sync::Mutex<Option<(Instant, serde_json::Value)>>> =
    OnceLock::new();

#[tauri::command]
fn take_pending_pack_add() -> Option<serde_json::Value> {
    let mut slot = PENDING_PACK_ADD
        .get_or_init(|| std::sync::Mutex::new(None))
        .lock()
        .unwrap();
    let value = slot
        .as_ref()
        .filter(|(t, _)| t.elapsed() < Duration::from_secs(30))
        .map(|(_, v)| v.clone());
    if slot
        .as_ref()
        .is_some_and(|(t, _)| t.elapsed() >= Duration::from_secs(30))
    {
        *slot = None;
    }
    value
}

/// Обрабатывает deep link: добавляет сборку (или подтверждает существующую)
/// и сообщает фронтенду через событие `pack-added`.
fn handle_deep_link(app: &AppHandle, url: &str) {
    let Some((pack_url, name, blog)) = parse_deep_link(url) else {
        return;
    };
    {
        let mut links = HANDLED_LINKS
            .get_or_init(|| std::sync::Mutex::new(Vec::new()))
            .lock()
            .unwrap();
        let now = Instant::now();
        links.retain(|(_, t)| now.duration_since(*t) < Duration::from_secs(10));
        if links.iter().any(|(u, _)| u == &pack_url) {
            return;
        }
        links.push((pack_url.clone(), now));
    }

    let client = app.state::<AppState>().client.clone();
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let _guard = add_pack_lock().lock().await;
        let result =
            ensure_pack_from_link(&client, &pack_url, name.as_deref(), blog.as_deref()).await;
        let payload = match &result {
            Ok((p, already)) => serde_json::json!({
                "ok": true,
                "already": already,
                "id": p.id,
                "name": p.name,
                "boostyBlog": p.boosty_blog,
                "minRam": p.min_ram_mb,
            }),
            Err(e) => serde_json::json!({
                "ok": false,
                "already": false,
                "id": "",
                "name": "",
                "error": e,
            }),
        };
        *PENDING_PACK_ADD
            .get_or_init(|| std::sync::Mutex::new(None))
            .lock()
            .unwrap() = Some((Instant::now(), payload.clone()));
        let _ = handle.emit("pack-added", payload);
    });
}

/// Регистрирует обработку deep link на всех платформах: single-instance ловит
/// ссылку в уже запущенном процессе, `get_current`/обработка событий — при
/// холодном запуске по ссылке (macOS), аргументы CLI — при холодном запуске на
/// Linux/Windows (ссылка приходит аргументом процесса).
pub fn register_deep_link_handlers(app: &tauri::AppHandle) {
    use tauri_plugin_deep_link::DeepLinkExt;

    let _ = app.deep_link().register(DEEP_LINK_SCHEME);

    let handle = app.clone();
    app.listen("deep-link://new-url", move |event| {
        let urls: Vec<tauri::Url> = serde_json::from_str(event.payload()).unwrap_or_default();
        for url in urls
            .iter()
            .filter(|u| u.as_str().starts_with(DEEP_LINK_PREFIX))
        {
            handle_deep_link(&handle, url.as_str());
        }
    });

    if let Ok(Some(urls)) = app.deep_link().get_current() {
        for url in urls
            .iter()
            .filter(|u| u.as_str().starts_with(DEEP_LINK_PREFIX))
        {
            handle_deep_link(app, url.as_str());
        }
    }

    for url in std::env::args()
        .skip(1)
        .filter(|a| a.starts_with(DEEP_LINK_PREFIX))
    {
        handle_deep_link(app, &url);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_deep_link_with_encoded_url() {
        let (url, name, blog) = parse_deep_link(&format!(
            "mono://add-pack?url={}&name=My%20Pack&blog=My-Blog",
            pct_decode("https%3A%2F%2Fgithub.com%2Fn1orio%2Fmono-pack-example")
        ))
        .unwrap();
        assert_eq!(url, "https://github.com/n1orio/mono-pack-example");
        assert_eq!(name.as_deref(), Some("My Pack"));
        assert_eq!(blog.as_deref(), Some("My-Blog"));
    }

    #[test]
    fn parses_deep_link_without_name() {
        let (url, name, blog) = parse_deep_link(
            "mono://add-pack?url=https%3A%2F%2Fgithub.com%2Fn1orio%2Fmono-pack-example",
        )
        .unwrap();
        assert_eq!(url, "https://github.com/n1orio/mono-pack-example");
        assert_eq!(name, None);
        assert_eq!(blog, None);
    }

    #[test]
    fn parses_deep_link_without_blog() {
        let (_, _, blog) = parse_deep_link(
            "mono://add-pack?url=https%3A%2F%2Fgithub.com%2Fn1orio%2Fmono-pack-example&name=Pack",
        )
        .unwrap();
        assert_eq!(blog, None);
    }

    #[test]
    fn rejects_other_paths_and_schemes() {
        assert!(parse_deep_link("mono://install?url=x").is_none());
        assert!(parse_deep_link("https://github.com/n1orio/mono-pack-example").is_none());
        assert!(parse_deep_link("mono://add-pack").is_none());
    }

    #[test]
    fn pct_decodes_reserved_chars() {
        assert_eq!(pct_decode("a%20b%2Fc"), "a b/c");
        assert_eq!(pct_decode("plain"), "plain");
        assert_eq!(pct_decode("100%"), "100%");
    }
}

/// Все версии сборки: релизы GitHub + установленные + активная.
#[tauri::command]
async fn list_versions(
    state: State<'_, AppState>,
    pack_id: Option<String>,
) -> Result<VersionsInfo, String> {
    let pack = resolve_pack(pack_id)?;
    let installed = mrpack::installed_details(&pack.id);
    let active = config::active_version(&pack.id)
        .ok()
        .filter(|v| !v.is_empty());
    let github = fetch_releases_cached(&state.client, &pack).await;
    Ok(VersionsInfo {
        github,
        installed,
        active,
    })
}

/// Список файлов папки игры (моды/ресурспаки/шейдеры/миры) с состоянием.
#[tauri::command]
fn list_game_files_command(
    pack_id: Option<String>,
    folder: String,
) -> Result<Vec<files::GameFileEntry>, String> {
    let pack = resolve_pack(pack_id)?;
    files::list_files(&pack.id, &folder).map_err(|e| e.to_string())
}

/// Включает/выключает файл (мод/ресурспак/шейдер) переименованием в *.disabled.
#[tauri::command]
fn toggle_game_file_command(
    pack_id: Option<String>,
    folder: String,
    name: String,
    enabled: bool,
) -> Result<(), String> {
    let pack = resolve_pack(pack_id)?;
    ensure_unlocked(&pack.id)?;
    files::toggle_file(&pack.id, &folder, &name, enabled).map_err(|e| e.to_string())
}

/// Заблокировано ли изменение файлов сборки (подписка/managed-сборка).
#[tauri::command]
fn pack_locked_command(pack_id: Option<String>) -> Result<bool, String> {
    let pack = resolve_pack(pack_id)?;
    Ok(config::pack_locked(&pack.id))
}

/// Включает/снимает блокировку правок сборки («отвязка» — пользователь сам
/// вносит изменения). Возвращает новое состояние.
#[tauri::command]
fn set_pack_locked_command(pack_id: String, locked: bool) -> Result<bool, String> {
    let pack = resolve_pack(Some(pack_id))?;
    config::set_pack_locked(&pack.id, locked).map_err(|e| e.to_string())?;
    Ok(config::pack_locked(&pack.id))
}

/// Не даёт менять файлы сборки, пока она заблокирована.
fn ensure_unlocked(pack_id: &str) -> Result<(), String> {
    if config::pack_locked(pack_id) {
        Err(
            "Сборка заблокирована: чтобы вносить изменения, отвяжите её в настройках сборки."
                .into(),
        )
    } else {
        Ok(())
    }
}

/// Иконка файла/папки (data-URL PNG) для отображения в списках.
#[tauri::command]
fn get_game_file_icon_command(
    pack_id: Option<String>,
    folder: String,
    name: String,
) -> Result<Option<String>, String> {
    let pack = resolve_pack(pack_id)?;
    files::file_icon(&pack.id, &folder, &name).map_err(|e| e.to_string())
}

/// Иконки пачки файлов одним вызовом (data-URL PNG или None).
#[tauri::command]
fn get_game_file_icons_command(
    pack_id: Option<String>,
    folder: String,
    names: Vec<String>,
) -> Result<Vec<files::GameFileIcon>, String> {
    let pack = resolve_pack(pack_id)?;
    Ok(files::file_icons(&pack.id, &folder, &names))
}

/// Лента новостей: релизы и обновления лаунчера, релизы (обновления) + посты
/// всех сборок, свежие сверху.
#[tauri::command]
async fn get_news_command(
    app: AppHandle,
    state: State<'_, AppState>,
    locale: String,
) -> Result<Vec<NewsItem>, String> {
    // Локализованные ноты берём из latest.json (манифест обновления): ключ
    // `notes_localized[locale]`, фолбэк — английский, затем тело релиза.
    let mut notes_localized: HashMap<String, String> = HashMap::new();
    let mut latest_launcher_ver: Option<String> = None;
    if let Some(m) = http_cache::cached_json(&state.client, LAUNCHER_MANIFEST_URL).await {
        latest_launcher_ver = m
            .get("version")
            .and_then(|v| v.as_str())
            .map(String::from);
        if let Some(map) = m.get("notes_localized").and_then(|x| x.as_object()) {
            for (k, v) in map {
                if let Some(s) = v.as_str() {
                    if !s.is_empty() {
                        notes_localized.insert(k.clone(), s.to_string());
                    }
                }
            }
        }
    }

    let mut items: Vec<NewsItem> = Vec::new();
    // Обновления лаунчера (релизы из GitHub Releases) — грузим первыми,
    // обычно это самые свежие новости.
    for rel in fetch_launcher_releases_cached(&state.client).await {
        // Локализуем только самый свежий релиз (у остальных нот в манифесте нет).
        let is_latest = latest_launcher_ver
            .as_deref()
            .map(|v| rel.tag == format!("launcher-v{v}"))
            .unwrap_or(false);
        let body = if is_latest {
            notes_localized
                .get(&locale)
                .or_else(|| notes_localized.get("en"))
                .cloned()
                .unwrap_or_else(|| rel.body.clone())
        } else {
            rel.body.clone()
        };
        items.push(NewsItem {
            kind: "update".into(),
            pack_id: "launcher".into(),
            pack_name: "Mono Launcher".into(),
            title: rel.name,
            body,
            url: rel.url,
            tag: Some(rel.tag),
            category: None,
            date: rel.published_at,
        });
    }
    emit_news_chunk(&app, &items);
    // Глобальные посты лаунчера.
    items.extend(
        fetch_discussions_cached(
            &state.client,
            NEWS_REPO.0,
            NEWS_REPO.1,
            "post",
            "launcher",
            "Mono Launcher",
        )
        .await,
    );
    emit_news_chunk(&app, &items);
    for pack in config::all_packs().map_err(|e| e.to_string())? {
        for rel in fetch_releases_cached(&state.client, &pack).await {
            items.push(NewsItem {
                kind: "update".into(),
                pack_id: pack.id.to_string(),
                pack_name: pack.name.to_string(),
                title: rel.name,
                body: rel.body,
                url: rel.url,
                tag: Some(rel.tag),
                category: None,
                date: rel.published_at,
            });
        }
        // Посты из репозитория сборки (если там включены Discussions).
        if let Some((owner, repo)) = parse_github_repo(&pack) {
            items.extend(
                fetch_discussions_cached(
                    &state.client,
                    &owner,
                    &repo,
                    "post",
                    &pack.id,
                    &pack.name,
                )
                .await,
            );
        }
        emit_news_chunk(&app, &items);
    }
    sort_news(&mut items);
    Ok(items)
}

/// Свежие сверху (без даты — вниз).
fn sort_news(items: &mut [NewsItem]) {
    items.sort_by(|a, b| match (&b.date, &a.date) {
        (Some(x), Some(y)) => x.cmp(y),
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (None, None) => std::cmp::Ordering::Equal,
    });
}

/// Отправляет фронтенду текущий накопленный список новостей (потоковая загрузка):
/// UI показывает новости по мере их подгрузки, свежие сверху.
fn emit_news_chunk(app: &AppHandle, items: &[NewsItem]) {
    let mut sorted = items.to_vec();
    sort_news(&mut sorted);
    let _ = app.emit("news-chunk", sorted);
}

/// Контент репозитория сборки: звёзды GitHub + скриншоты (screenshots.json)
/// + сервера (servers.json). Raw-файлы не тратят лимиты GitHub API.
async fn fetch_pack_repo_content(
    client: &reqwest::Client,
    owner: &str,
    repo: &str,
) -> PackRepoContent {
    let mut out = PackRepoContent::default();

    // Звёзды: один API-вызов на репозиторий (кэшируется + ETag).
    let meta_url = format!("https://api.github.com/repos/{owner}/{repo}");
    if let Some(v) = http_cache::cached_json(client, &meta_url).await {
        out.stars = v["stargazers_count"].as_i64();
    }

    // Сервера: манифест списка серверов.
    let srv_url = format!("https://raw.githubusercontent.com/{owner}/{repo}/HEAD/servers.json");
    if let Some(v) = http_cache::cached_json(client, &srv_url).await {
        if let Some(arr) = v.as_array() {
            for item in arr {
                let name = item
                    .get("name")
                    .and_then(|x| x.as_str())
                    .map(String::from)
                    .unwrap_or_default();
                let ip = item
                    .get("ip")
                    .and_then(|x| x.as_str())
                    .map(String::from)
                    .unwrap_or_default();
                if name.is_empty() || ip.is_empty() {
                    continue;
                }
                out.servers.push(PackServer {
                    name,
                    ip,
                    port: item.get("port").and_then(|x| x.as_u64()).map(|p| p as u16),
                    desc: item.get("desc").and_then(|x| x.as_str()).map(String::from),
                });
            }
        }
    }

    // Соцсети: объект `{ "name": "url" }` или массив `["url", {"name": "url", "color": "#rrggbb"}]`.
    let soc_url = format!("https://raw.githubusercontent.com/{owner}/{repo}/HEAD/socials.json");
    if let Some(v) = http_cache::cached_json(client, &soc_url).await {
        let mut push = |name: String, url: String, color: Option<String>| {
            let name = normalize_social_name(&name, &url);
            let color = color.filter(|c| is_hex_color(c));
            if !name.is_empty() && url.starts_with("https://") && out.socials.len() < 8 {
                out.socials.push(PackSocial { name, url, color });
            }
        };
        match v {
            serde_json::Value::Object(map) => {
                for (name, u) in map {
                    if let Some(url) = u.as_str() {
                        push(name, url.to_string(), None);
                    }
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    match item {
                        serde_json::Value::String(url) => push(String::new(), url, None),
                        serde_json::Value::Object(o) => {
                            let name = o
                                .get("name")
                                .and_then(|x| x.as_str())
                                .unwrap_or("")
                                .to_string();
                            let url = o.get("url").and_then(|x| x.as_str()).map(String::from);
                            let color = o.get("color").and_then(|x| x.as_str()).map(String::from);
                            if let Some(url) = url {
                                push(name, url, color);
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    // Тема лаунчера: необязательный theme.json с hex-цветами.
    let theme_url = format!("https://raw.githubusercontent.com/{owner}/{repo}/HEAD/theme.json");
    if let Some(v) = http_cache::cached_json(client, &theme_url).await {
        let mut theme = PackTheme::default();
        let mut has_any = false;
        let fields: [(&str, &mut Option<String>); 11] = [
                    ("bg", &mut theme.bg),
                    ("panel", &mut theme.panel),
                    ("input", &mut theme.input),
                    ("border", &mut theme.border),
                    ("tx", &mut theme.tx),
                    ("txStrong", &mut theme.tx_strong),
                    ("txMuted", &mut theme.tx_muted),
                    ("accent", &mut theme.accent),
                    ("accentStrong", &mut theme.accent_strong),
                    ("accentHover", &mut theme.accent_hover),
                    ("accentDeep", &mut theme.accent_deep),
                ];
                for (key, slot) in fields {
                    let Some(raw) = v.get(key).and_then(|x| x.as_str()) else {
                        continue;
                    };
                    if raw.len() == 7
                    && raw.starts_with('#')
                    && raw[1..].chars().all(|c| c.is_ascii_hexdigit())
                {
                    *slot = Some(raw.to_string());
                    has_any = true;
                }
            }
            if has_any {
                out.theme = Some(theme);
            }
        }

    // Баннер сборки: необязательный banner.png в корне репозитория.
    let banner_url = format!("https://raw.githubusercontent.com/{owner}/{repo}/HEAD/banner.png");
    if http_cache::cached_get(client, &banner_url, &[]).await.is_some() {
        out.banner = Some(banner_url);
    }
    out
}

/// Валидный hex-цвет `#rrggbb` (как в theme.json).
fn is_hex_color(raw: &str) -> bool {
    raw.len() == 7
        && raw.starts_with('#')
        && raw[1..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Выводит имя соцсети из её имени или домена ссылки.
fn normalize_social_name(name: &str, url: &str) -> String {
    let trimmed = name.trim().to_lowercase();
    if !trimmed.is_empty() {
        return trimmed;
    }
    let host = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("");
    let host = host.to_lowercase();
    let known = [
        ("discord.gg", "discord"),
        ("discord.com", "discord"),
        ("t.me", "telegram"),
        ("telegram.me", "telegram"),
        ("vk.com", "vk"),
        ("youtube.com", "youtube"),
        ("youtu.be", "youtube"),
        ("twitch.tv", "twitch"),
        ("x.com", "x"),
        ("twitter.com", "x"),
        ("github.com", "github"),
        ("boosty.to", "boosty"),
        ("patreon.com", "patreon"),
        ("tiktok.com", "tiktok"),
    ];
    for (host_pat, social) in known {
        if host == host_pat || host.ends_with(&format!(".{host_pat}")) {
            return social.to_string();
        }
    }
    "link".to_string()
}

/// `fetch_pack_repo_content` с кэшем (15 минут).
async fn fetch_pack_repo_content_cached(
    client: &reqwest::Client,
    owner: &str,
    repo: &str,
) -> PackRepoContent {
    let key = format!("meta/{owner}/{repo}");
    {
        let cache = api_cache().lock().unwrap_or_else(|e| e.into_inner());
        if let Some((at, v)) = cache.meta.get(&key) {
            if at.elapsed() < API_HIT_TTL {
                return v.clone();
            }
        }
    }
    let fetched = fetch_pack_repo_content(client, owner, repo).await;
    let mut cache = api_cache().lock().unwrap_or_else(|e| e.into_inner());
    cache.meta.insert(key, (Instant::now(), fetched.clone()));
    fetched
}

/// Каталог сборок из `catalog.json` репозитория лаунчера, с кэшем 15 минут.
async fn fetch_catalog_cached(client: &reqwest::Client) -> Vec<CatalogEntry> {
    {
        let cache = api_cache().lock().unwrap_or_else(|e| e.into_inner());
        if let Some((at, v)) = &cache.catalog {
            if at.elapsed() < API_HIT_TTL {
                return v.clone();
            }
        } else if let Some(at) = cache.failures.get("catalog") {
            if at.elapsed() < API_FAIL_RETRY {
                return Vec::new();
            }
        }
    }
    let mut entries: Vec<CatalogEntry> = Vec::new();
    if let Some(bytes) = http_cache::cached_get(client, CATALOG_URL, &[]).await {
        if let Ok(list) = serde_json::from_slice::<Vec<CatalogEntry>>(&bytes) {
            entries = list
                .into_iter()
                .filter(|e| !e.name.trim().is_empty() && !e.url.trim().is_empty())
                .map(|mut e| {
                    e.min_ram_mb = e.min_ram_mb.map(|r| r.clamp(256, 65536));
                    e
                })
                .collect();
        }
    }
    let mut cache = api_cache().lock().unwrap_or_else(|e| e.into_inner());
    if entries.is_empty() {
        cache.failures.insert("catalog".to_string(), Instant::now());
    } else {
        cache.catalog = Some((Instant::now(), entries.clone()));
        cache.failures.remove("catalog");
    }
    entries
}

/// Каталог сборок для вкладки «Каталог» (список от авторов, курируется в этом репозитории).
#[tauri::command]
async fn fetch_catalog_command(state: State<'_, AppState>) -> Result<Vec<CatalogEntry>, String> {
    Ok(fetch_catalog_cached(&state.client).await)
}

/// Скриншоты и сервера текущей сборки (из её GitHub-репозитория).
#[tauri::command]
async fn pack_repo_content_command(
    state: State<'_, AppState>,
    pack_id: Option<String>,
) -> Result<PackRepoContent, String> {
    let pack = resolve_pack(pack_id)?;
    let Some((owner, repo)) = parse_github_repo(&pack) else {
        return Ok(PackRepoContent::default());
    };
    Ok(fetch_pack_repo_content_cached(&state.client, &owner, &repo).await)
}

/// Переключает активную версию сборки (по тегу GitHub или versionId).
#[tauri::command]
async fn switch_version(pack_id: Option<String>, version_id: String) -> Result<(), String> {
    let pack = resolve_pack(pack_id)?;
    if version_id.is_empty() {
        return Err("Пустая версия".into());
    }
    // Разрешаем передавать как versionId, так и тег релиза.
    let resolved = mrpack::installed_details(&pack.id)
        .iter()
        .find(|v| {
            v.version_id == version_id || v.source_tag.as_deref() == Some(version_id.as_str())
        })
        .map(|v| v.version_id.clone())
        .ok_or_else(|| format!("Версия {version_id} не установлена"))?;
    config::set_active_version(&pack.id, &resolved).map_err(|e| e.to_string())
}

/// Возвращает информацию о памяти системы.
#[tauri::command]
fn system_info() -> Result<SystemInfo, String> {
    let mut sys = System::new();
    sys.refresh_memory();
    // sysinfo возвращает байты -> ГБ (GiB). Общее округляем вверх (физические 31,3 -> 32),
    // доступное — вниз, чтобы не обманывать пользователя.
    let gib = 1024f64 * 1024.0 * 1024.0;
    let total = (sys.total_memory() as f64 / gib).ceil() as u64;
    let available = (sys.available_memory() as f64 / gib).floor() as u64;
    Ok(SystemInfo {
        total_ram_gb: total,
        available_ram_gb: available,
    })
}

/// Проверяет наличие новой версии `.mrpack` на GitHub Releases.
#[tauri::command]
async fn check_for_updates(
    state: State<'_, AppState>,
    pack_id: Option<String>,
) -> Result<UpdateInfo, String> {
    let pack = resolve_pack(pack_id)?;
    let current = active_installed_tag(&pack.id);
    let releases = fetch_releases_cached(&state.client, &pack).await;
    let latest = releases.first().map(|r| r.tag.clone());
    Ok(UpdateInfo {
        current_version: current.clone(),
        has_update: match (&current, &latest) {
            (Some(c), Some(l)) => l != c,
            _ => false,
        },
        latest_version: latest,
    })
}

/// Полное скачивание и установка сборки.
/// Если tag указан — ставится конкретный релиз, иначе latest.
#[tauri::command]
async fn install_mrpack(
    app: AppHandle,
    state: State<'_, AppState>,
    pack_id: Option<String>,
    tag: Option<String>,
) -> Result<mrpack::PackInfo, String> {
    let pack = resolve_pack(pack_id)?;
    // Гейт лицензии: платные сборки требуют активную подписку Boosty.
    license::ensure_license(&state.client, &pack.id)
        .await
        .map_err(|e| e.to_string())?;
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(600))
        .build()
        .map_err(|e| e.to_string())?;
    let (url, install_tag) = match &tag {
        Some(t) => (mrpack_url_for_tag(&pack, t), Some(t.clone())),
        // latest в GitHub не перенаправляет на пререлизы, поэтому берём
        // самый свежий релиз из API (включая пререлизы). А tag записываем
        // в маркер установки — иначе лаунчер будет вечно «обнаруживать обновление».
        None => match fetch_releases_cached(&client, &pack)
            .await
            .into_iter()
            .next()
        {
            Some(r) => (r.url, Some(r.tag)),
            None => (pack.url.to_string(), None),
        },
    };
    let installed = mrpack::install_mrpack(app, &client, &pack.id, &url, install_tag.as_deref())
        .await
        .map_err(|e| e.to_string())?;
    // Сборки, скачиваемые из удалённого источника (GitHub/Modrinth/CurseForge) —
    // управляемые по умолчанию: правки файлов заблокированы, пока пользователь
    // не «отвяжет» сборку. Локальные сборки не трогаем.
    if pack.kind == "remote" {
        let _ = config::set_pack_locked(&pack.id, true);
    }
    Ok(installed)
}

/// Определяет, установлена ли сборка и возвращает общий статус.
#[tauri::command]
async fn get_status(pack_id: Option<String>) -> Result<AppStatus, String> {
    let pack = resolve_pack(pack_id)?;
    let mut status = AppStatus {
        mrpack_url: pack.url.to_string(),
        active_version: config::active_version(&pack.id)
            .ok()
            .filter(|v| !v.is_empty()),
        active_source_tag: active_installed_tag(&pack.id),
        installed_versions: mrpack::installed_versions(&pack.id),
        discord_rp_enabled: config::discord_rp_enabled(),
        warn_custom_mods: config::warn_custom_mods_enabled(),
        playtime_seconds: mrpack::pack_playtime_seconds(&pack.id),
        ..Default::default()
    };

    // Общая статистика лаунчера: часы во всех сборках + сколько сборок игралось.
    if let Ok(packs) = config::all_packs() {
        let mut total = 0u64;
        let mut played = 0u64;
        for p in &packs {
            let t = mrpack::pack_playtime_seconds(&p.id);
            if t > 0 {
                total += t;
                played += 1;
            }
        }
        status.total_playtime_seconds = total;
        status.played_packs = played;
    }

    // Читаем метаданные активной версии из её папки.
    if let Some(idx) = status
        .active_version
        .as_ref()
        .and_then(|v| mrpack::read_version_index(&pack.id, v))
    {
        status.pack_name = Some(idx.name.clone());
        status.minecraft_version = idx.dependencies.get("minecraft").cloned();
        status.loader = ["fabric-loader", "forge", "neoforge", "quilt"]
            .iter()
            .find(|k| idx.dependencies.contains_key(**k))
            .map(|k| k.replace("-loader", ""));
        status.loader_version = ["fabric-loader", "forge", "neoforge", "quilt"]
            .iter()
            .find_map(|k| idx.dependencies.get(*k).cloned());

        let game_dir = config::version_dir(&pack.id, &idx.version_id).map_err(|e| e.to_string())?;
        status.installed = mrpack::is_installed(&game_dir, &idx);
    }

    if let Some(v) = status.active_version.as_deref() {
        status.custom_mods = mrpack::read_custom_mods(&pack.id, v);
    }

    status.session = auth::load_session().ok().flatten();
    Ok(status)
}

/// Папка, которую открывает «Папка сборки»: активная версия сборки, если
/// установлена; иначе папка данных сборки (создаётся при надобности).
fn resolve_pack_open_dir(pack_id: Option<String>) -> Result<std::path::PathBuf, String> {
    let pack = resolve_pack(pack_id)?;
    Ok(config::active_version(&pack.id)
        .ok()
        .filter(|v| !v.is_empty())
        .and_then(|v| config::version_dir(&pack.id, &v).ok())
        .filter(|d| d.exists())
        .or_else(|| config::versions_root(&pack.id).ok().filter(|d| d.exists()))
        .or_else(|| config::pack_dir(&pack.id).ok())
        .unwrap_or_else(|| config::launcher_root().unwrap_or_else(|_| std::env::temp_dir())))
}

/// Открывает в системном проводнике папку активной версии сборки.
/// Если сборка ещё не установлена — открывает (создавая) папку данных сборки.
#[tauri::command]
fn open_pack_dir(app: AppHandle, pack_id: Option<String>) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let dir = resolve_pack_open_dir(pack_id)?;
    let _ = std::fs::create_dir_all(&dir);
    app.opener()
        .open_path(dir.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| e.to_string())
}

/// Путь к папке сборки (для UI: подсказки, копирования пути в браузере и т.п.).
#[tauri::command]
fn get_pack_dir_command(pack_id: Option<String>) -> Result<String, String> {
    resolve_pack_open_dir(pack_id).map(|d| d.to_string_lossy().into_owned())
}

/// Оффлайн-логин.
#[tauri::command]
async fn login_offline_command(username: String) -> Result<UserSession, String> {
    let session = login_offline(&username).map_err(|e| e.to_string())?;
    save_session(&session).map_err(|e| e.to_string())?;
    // Любой вход попадает в список аккаунтов и становится активным.
    let _ = auth::upsert_account(&session);
    Ok(session)
}

/// Microsoft OAuth2, фаза 1: запрашиваем device code для показа в UI.
#[tauri::command]
async fn ms_device_code_command(
    state: State<'_, AppState>,
) -> Result<auth::DeviceCodeInfo, String> {
    auth::ms_device_code(&state.client)
        .await
        .map_err(|e| e.to_string())
}

/// Microsoft OAuth2, фаза 2: поллим подтверждение и проходим
/// цепочку Xbox Live → XSTS → Minecraft, возвращаем игровую сессию.
#[tauri::command]
async fn ms_poll_command(
    state: State<'_, AppState>,
    device_code: String,
    interval: u64,
    expires_in: u64,
) -> Result<UserSession, String> {
    let session = auth::ms_poll(&state.client, &device_code, interval, expires_in)
        .await
        .map_err(|e| e.to_string())?;
    save_session(&session).map_err(|e| e.to_string())?;
    // Любой вход попадает в список аккаунтов и становится активным.
    let _ = auth::upsert_account(&session);
    Ok(session)
}

/// Ely.by OAuth2, фаза 1: запрашиваем device code для показа в UI.
#[tauri::command]
async fn ely_device_code_command(
    state: State<'_, AppState>,
) -> Result<auth::DeviceCodeInfo, String> {
    auth::ely_device_code(&state.client)
        .await
        .map_err(|e| e.to_string())
}

/// Ely.by OAuth2, фаза 2: поллим подтверждение и возвращаем игровую сессию
/// (токен Ely.by с правами minecraft_server_session передаётся напрямую игре).
#[tauri::command]
async fn ely_poll_command(
    state: State<'_, AppState>,
    device_code: String,
    interval: u64,
    expires_in: u64,
) -> Result<UserSession, String> {
    let session = auth::ely_poll(&state.client, &device_code, interval, expires_in)
        .await
        .map_err(|e| e.to_string())?;
    save_session(&session).map_err(|e| e.to_string())?;
    let _ = auth::upsert_account(&session);
    Ok(session)
}

/// Список сохранённых аккаунтов и активный (accounts.json).
#[tauri::command]
fn list_accounts_command() -> Result<auth::Accounts, String> {
    Ok(auth::load_accounts())
}

/// Переключает активный аккаунт и возвращает его сессию.
#[tauri::command]
fn switch_account_command(id: String) -> Result<UserSession, String> {
    auth::switch_account(&id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Аккаунт не найден".to_string())
}

/// Удаляет аккаунт; активным становится первый оставшийся (или выход).
#[tauri::command]
fn remove_account_command(id: String) -> Result<Option<UserSession>, String> {
    auth::remove_account(&id).map_err(|e| e.to_string())
}

/// Запуск игры.
#[tauri::command]
#[allow(clippy::too_many_arguments)] // Tauri-сигнатура фиксированная.
async fn launch_game_command(
    app: AppHandle,
    state: State<'_, AppState>,
    pack_id: Option<String>,
    ram_gb: u32,
    session: UserSession,
    width: u32,
    height: u32,
    server_address: Option<String>,
) -> Result<(), String> {
    let pack_id = pack_id.unwrap_or_else(|| default_pack_id().to_string());
    // Гейт лицензии: платные сборки требуют активную подписку Boosty.
    license::ensure_license(&state.client, &pack_id)
        .await
        .map_err(|e| e.to_string())?;
    // Авто-коннект ("host" или "host:port") — пустая строка игнорируется.
    let server = server_address
        .as_deref()
        .and_then(game::parse_server_address);
    game::launch_game(
        &pack_id,
        ram_gb,
        session,
        app,
        width.max(320),
        height.max(240),
        server,
    )
    .await
    .map_err(|e| e.to_string())
}

/// Пинг Minecraft-сервера (1.7+ статус с фолбэком на legacy 0xFE).
#[tauri::command]
async fn ping_server_command(
    address: String,
    port: Option<u16>,
) -> Result<ping::ServerStatus, String> {
    ping::ping_server(&address, port.unwrap_or(25565)).await
}

/// Текущий скин (локальный файл + модель).
#[tauri::command]
fn get_local_skin_command() -> Result<skins::SkinInfo, String> {
    skins::get_skin().map_err(|e| e.to_string())
}

/// Устанавливает скин из выбранного PNG и грузит его в публичный скин-API.
#[tauri::command]
async fn set_local_skin_command(
    state: tauri::State<'_, AppState>,
    path: String,
    model: String,
    nick: String,
) -> Result<skins::SkinInfo, String> {
    let info = skins::set_skin_local(&path, &model).map_err(|e| e.to_string())?;
    let bytes =
        std::fs::read(skins::skin_path().map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let model = if model == "slim" { "slim" } else { "classic" };
    skins::upload_skin(&state.client, &nick, &bytes, model)
        .await
        .map_err(|e| e.to_string())?;
    Ok(info)
}

/// Удаляет скин (локально и из API).
#[tauri::command]
async fn clear_local_skin_command(
    state: tauri::State<'_, AppState>,
    nick: String,
) -> Result<(), String> {
    let _ = skins::delete_remote_skin(&state.client, &nick).await;
    skins::clear_skin_local().map_err(|e| e.to_string())
}

/// Базовый URL скин-API (для инструкции разработчикам серверов).
#[tauri::command]
fn skin_api_url_command() -> String {
    skins::SKINS_API_URL.to_string()
}

/// Принимает токен Boosty от игрока: сохраняет и проверяет подписку на блог сборки.
#[tauri::command]
async fn set_boosty_command(
    state: State<'_, AppState>,
    pack_id: String,
    token: String,
) -> Result<license::LicenseInfo, String> {
    license::set_license(&state.client, &pack_id, &token)
        .await
        .map_err(|e| e.to_string())
}

/// Статус лицензии сборки (подписка Boosty) для панели в UI.
#[tauri::command]
async fn license_status_command(
    state: State<'_, AppState>,
    pack_id: String,
) -> Result<license::LicenseInfo, String> {
    license::license_status(&state.client, &pack_id)
        .await
        .map_err(|e| e.to_string())
}

/// Удаляет сохранённый токен Boosty сборки.
#[tauri::command]
fn clear_license_command(pack_id: String) -> Result<(), String> {
    license::clear_license(&pack_id).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ScreenshotList {
    installed: bool,
    screenshots: Vec<String>,
}

/// Скриншоты активной установленной версии: папка `screenshots` игрового
/// каталога (полные пути — фронт отдаёт их через asset-протокол).
#[tauri::command]
fn list_screenshots_command(pack_id: Option<String>) -> Result<ScreenshotList, String> {
    let pack_id = pack_id.unwrap_or_else(|| default_pack_id().to_string());
    let installed = config::active_version_file(&pack_id)
        .map(|f| f.exists())
        .unwrap_or(false);
    let mut screenshots = Vec::new();
    if installed {
        if let Ok(dir) = config::active_game_dir(&pack_id) {
            let shots_dir = dir.join("screenshots");
            if shots_dir.is_dir() {
                if let Ok(rd) = std::fs::read_dir(&shots_dir) {
                    let mut files: Vec<String> = rd
                        .flatten()
                        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
                        .filter(|e| {
                            e.path()
                                .extension()
                                .and_then(|x| x.to_str())
                                .map(|x| {
                                    ["png", "jpg", "jpeg", "webp", "bmp", "gif"]
                                        .contains(&x.to_lowercase().as_str())
                                })
                                .unwrap_or(false)
                        })
                        .map(|e| e.path().to_string_lossy().to_string())
                        .collect();
                    files.sort();
                    screenshots = files;
                }
            }
        }
    }
    Ok(ScreenshotList {
        installed,
        screenshots,
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SavedServersList {
    installed: bool,
    servers: Vec<nbt::SavedServer>,
}

/// Сервера игрока из servers.dat активной установленной версии.
#[tauri::command]
fn list_servers_command(pack_id: Option<String>) -> Result<SavedServersList, String> {
    let pack_id = pack_id.unwrap_or_else(|| default_pack_id().to_string());
    let installed = config::active_version_file(&pack_id)
        .map(|f| f.exists())
        .unwrap_or(false);
    let mut servers = Vec::new();
    if installed {
        if let Ok(dir) = config::active_game_dir(&pack_id) {
            let file = dir.join("servers.dat");
            if file.exists() {
                let data = std::fs::read(&file).map_err(|e| format!("servers.dat: чтение: {e}"))?;
                nbt::parse_servers_dat(&data, &mut servers)
                    .map_err(|e| format!("servers.dat: разбор: {e}"))?;
            }
        }
    }
    Ok(SavedServersList { installed, servers })
}

/// Последние строки лога запуска (для показа в UI при старте).
#[tauri::command]
fn get_launch_log() -> Result<String, String> {
    let path = config::launch_log_file().map_err(|e| e.to_string())?;
    let raw = std::fs::read_to_string(&path).unwrap_or_default();
    // Только хвост — не заваливаем UI огромным логом.
    let tail: Vec<&str> = raw.lines().rev().take(1500).collect();
    let mut out: Vec<&str> = tail.into_iter().collect();
    out.reverse();
    Ok(out.join("\n"))
}

/// Очищает сохранённый лог запуска.
#[tauri::command]
fn clear_launch_log() -> Result<(), String> {
    let path = config::launch_log_file().map_err(|e| e.to_string())?;
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Анализирует свежие краш-артефакты сборки и классифицирует причину.
#[tauri::command]
fn analyze_crash_command(pack_id: Option<String>) -> crash::CrashAnalysis {
    let pack_id = pack_id.unwrap_or_else(|| default_pack_id().to_string());
    crash::analyze_pack(&pack_id)
}

/// Версия лаунчера (из Cargo.toml) — для отчётов об ошибках.
#[tauri::command]
fn launcher_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Открывает URL во внешнем браузере.
#[tauri::command]
fn open_url(app: AppHandle, url: String) -> Result<(), String> {
    // Разрешаем только web/mailto — строки приходят из сторонних metadata
    // (socials.json и т.п.), нельзя допускать file:// и произвольных схем.
    let lower = url.to_ascii_lowercase();
    if !(lower.starts_with("https://")
        || lower.starts_with("http://")
        || lower.starts_with("mailto:"))
    {
        return Err("Недопустимая ссылка (разрешены только http/https/mailto)".into());
    }
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(&url, None::<&str>)
        .map_err(|e| e.to_string())
}

/// Список найденных Java (встроенная + системные + PATH) с флагом выбранной.
#[tauri::command]
fn list_java_command() -> Vec<jre::JavaInfo> {
    jre::list_javas()
}

/// Сохраняет выбранную Java (путь к бинарю) в конфиг. None — авто.
#[tauri::command]
fn set_java_path_command(path: Option<String>) -> Result<(), String> {
    config::set_java_selection(path.as_deref()).map_err(|e| e.to_string())
}

/// Версия Java, заданная для сборки (мажорный номер; None — авто).
#[tauri::command]
fn get_pack_java_command(pack_id: String) -> Option<u32> {
    config::pack_java(&pack_id)
}

/// Задаёт версию Java для сборки (None — авто по версии Minecraft).
#[tauri::command]
fn set_pack_java_command(pack_id: String, version: Option<u32>) -> Result<(), String> {
    config::set_pack_java(&pack_id, version).map_err(|e| e.to_string())
}

/// Скачивает и распаковывает Java нужного мажора (по умолчанию 21), если её ещё нет.
#[tauri::command]
async fn ensure_java_command(
    app: AppHandle,
    state: State<'_, AppState>,
    major: Option<u32>,
) -> Result<String, String> {
    jre::ensure_java(&app, &state.client, major.unwrap_or(21))
        .await
        .map_err(|e| e.to_string())
}

/// Проверка целостности файлов активной версии сборки.
#[tauri::command]
fn verify_game_command(pack_id: Option<String>) -> Result<mrpack::VerifyResult, String> {
    let pack = resolve_pack(pack_id)?;
    mrpack::verify_pack(&pack.id).map_err(|e| e.to_string())
}

/// Удаляет файлы/папки игры по именам (моды/ресурспаки/шейдеры/миры).
/// Имена берутся только из списка UI (базовые имена, без путей).
/// Снятые с учёта моды Modrinth вычищаются из трекинга обновлений.
#[tauri::command]
fn delete_game_files_command(
    app: AppHandle,
    pack_id: Option<String>,
    folder: String,
    names: Vec<String>,
) -> Result<usize, String> {
    let pack = resolve_pack(pack_id)?;
    ensure_unlocked(&pack.id)?;
    if !matches!(folder.as_str(), "mods" | "resourcepacks" | "shaderpacks" | "saves") {
        return Err("Неизвестная папка".into());
    }
    let removed = files::delete_files(&pack.id, &folder, &names).map_err(|e| e.to_string())?;
    if folder == "mods" {
        for name in &names {
            let _ = modrinth::remove_tracked_mod(&pack.id, name, "mods", None);
        }
    }
    let _ = app.emit("mods-changed", ());
    Ok(removed)
}

/// Открывает служебную папку игры: mods / screenshots / resourcepacks / shaderpacks / saves / logs.
#[tauri::command]
fn open_game_folder_command(
    app: AppHandle,
    pack_id: Option<String>,
    folder: String,
) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let pack = resolve_pack(pack_id)?;
    let sub = match folder.as_str() {
        "mods" | "screenshots" | "resourcepacks" | "shaderpacks" | "saves" | "logs" => {
            folder.clone()
        }
        _ => return Err("Неизвестная папка".into()),
    };
    let dir = config::active_game_dir(&pack.id)
        .map_err(|e| e.to_string())?
        .join(&sub);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    app.opener()
        .open_path(dir.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| e.to_string())
}

/// Включает/выключает Discord Rich Presence (конфиг discord-rp.txt).
#[tauri::command]
fn set_discord_rp_command(enabled: bool) -> Result<(), String> {
    config::set_discord_rp_enabled(enabled).map_err(|e| e.to_string())
}

/// Включает/выключает плашку предупреждения о кастомных модах (warn-custom-mods.txt).
#[tauri::command]
fn set_warn_custom_mods_command(enabled: bool) -> Result<(), String> {
    config::set_warn_custom_mods_enabled(enabled).map_err(|e| e.to_string())
}

/// Запоминает язык интерфейса (ru/en) для строк, формируемых на стороне Rust
/// (например, активность Discord).
#[tauri::command]
fn set_locale_command(locale: String) {
    crate::discord_rp::set_locale(locale);
}

/// URL текстуры скина Mojang-профиля по uuid (без даш). None — нет скина/профиля.
#[tauri::command]
async fn get_skin_command(
    state: State<'_, AppState>,
    uuid: String,
) -> Result<Option<String>, String> {
    let url = format!("https://sessionserver.mojang.com/session/minecraft/profile/{uuid}");
    let resp = state
        .client
        .get(&url)
        .header("User-Agent", "mono-launcher")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let skin_url = json
        .get("properties")
        .and_then(|p| p.as_array())
        .and_then(|props| props.iter().find(|pr| pr["name"] == "textures"))
        .and_then(|pr| {
            use base64::Engine;
            pr["value"]
                .as_str()
                .and_then(|b64| base64::engine::general_purpose::STANDARD.decode(b64).ok())
                .and_then(|raw| serde_json::from_slice::<serde_json::Value>(&raw).ok())
        })
        .and_then(|t| t["textures"]["SKIN"]["url"].as_str().map(|s| s.to_string()));
    Ok(skin_url)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState {
        client: reqwest::Client::new(),
    };

    tauri::Builder::default()
        .manage(state)
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        // Single-instance должен быть первым: ловит deep link аргументы
        // запущенного экземпляра на Linux/Windows.
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if let Some(url) = argv.iter().find(|a| a.starts_with(DEEP_LINK_PREFIX)) {
                handle_deep_link(app, url);
            }
        }))
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            #[cfg(desktop)]
            register_deep_link_handlers(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_packs,
            add_pack_command,
            remove_pack_command,
            take_pending_pack_add,
            check_for_updates,
            list_versions,
            switch_version,
            system_info,
            install_mrpack,
            get_status,
            login_offline_command,
            ms_device_code_command,
            ms_poll_command,
            ely_device_code_command,
            ely_poll_command,
            curseforge_search_command,
            curseforge_categories_command,
            curseforge_latest_file_command,
            curseforge_install_command,
            curseforge_modpack_files_command,
            curseforge_project_detail_command,
            curseforge_install_pack_command,
            curseforge_key_configured_command,
            list_accounts_command,
            switch_account_command,
            remove_account_command,
            launch_game_command,
            game::stop_game_command,
            ping_server_command,
            get_local_skin_command,
            set_local_skin_command,
            clear_local_skin_command,
            skin_api_url_command,
            list_screenshots_command,
            list_servers_command,
            get_launch_log,
            clear_launch_log,
            analyze_crash_command,
            open_pack_dir,
            get_pack_dir_command,
            launcher_version,
            open_url,
            list_java_command,
            set_java_path_command,
            get_pack_java_command,
            set_pack_java_command,
            ensure_java_command,
            verify_game_command,
            open_game_folder_command,
            get_skin_command,
            set_discord_rp_command,
            set_warn_custom_mods_command,
            set_locale_command,
            get_news_command,
            list_game_files_command,
            toggle_game_file_command,
            pack_locked_command,
            set_pack_locked_command,
            delete_game_files_command,
            get_game_file_icon_command,
            get_game_file_icons_command,
            pack_repo_content_command,
            fetch_catalog_command,
            set_boosty_command,
            license_status_command,
            clear_license_command,
            modrinth_search_command,
            modrinth_tags_command,
            modrinth_project_versions_command,
            modrinth_project_command,
            set_pack_icon_command,
            set_pack_banner_command,
            set_pack_name_command,
            recent_packs_command,
            fetch_pack_icon_command,
            modrinth_version_command,
            modrinth_install_mod_command,
            modrinth_check_updates_command,
            modrinth_update_mod_command,
            installed_mod_sha1_command,
            modrinth_remove_mod_command,
            modrinth_install_pack_command,
            create_local_pack_command,
            edit_pack_version_command,
            local_loader_versions_command,
            minecraft_versions_command,
            export_pack_command,
            export_list_command,
            export_author_pack_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
