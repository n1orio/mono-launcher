mod auth;
mod config;
mod discord_rp;
mod files;
mod game;
mod jre;
mod mrpack;

use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use tauri::{Emitter, Listener};
use sysinfo::System;

use crate::auth::{login_offline, save_session, UserSession};
use crate::config::{default_pack_id, PackInfo};

/// Глобальное состояние лаунчера (HTTP-клиент).
pub struct AppState {
    pub client: reqwest::Client,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppStatus {
    pub installed: bool,
    pub minecraft_version: Option<String>,
    pub loader: Option<String>,
    pub pack_name: Option<String>,
    pub session: Option<UserSession>,
    pub mrpack_url: String,
    pub active_version: Option<String>,
    pub active_source_tag: Option<String>,
    pub installed_versions: Vec<String>,
    pub discord_rp_enabled: bool,
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
    /// Владелец GitHub-репозитория сборки (если это github-сборка).
    pub author: Option<String>,
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

/// Контент репозитория сборки: звёзды GitHub + скриншоты + сервера.
#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct PackRepoContent {
    pub stars: Option<i64>,
    pub screenshots: Vec<String>,
    pub servers: Vec<PackServer>,
}

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
    let active = config::active_version(pack_id).ok().filter(|v| !v.is_empty())?;
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
fn parse_github_repo_from_url(url: &str) -> Option<(String, String)> {
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
    failures: HashMap<String, Instant>,
}

static API_CACHE: OnceLock<std::sync::Mutex<ApiCache>> = OnceLock::new();

fn api_cache() -> &'static std::sync::Mutex<ApiCache> {
    API_CACHE.get_or_init(|| {
        std::sync::Mutex::new(ApiCache {
            releases: HashMap::new(),
            discussions: HashMap::new(),
            meta: HashMap::new(),
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
        cache.releases.insert(key.clone(), (Instant::now(), fetched.clone()));
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
    let fetched =
        fetch_discussions(client, owner, repo, kind, pack_id, pack_name).await;
    let mut cache = api_cache().lock().unwrap_or_else(|e| e.into_inner());
    if fetched.is_empty() {
        cache.failures.insert(key, Instant::now());
    } else {
        cache.discussions.insert(key.clone(), (Instant::now(), fetched.clone()));
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
async fn fetch_repo_releases(
    client: &reqwest::Client,
    owner: &str,
    repo: &str,
) -> Vec<GhVersion> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases");
    let Ok(resp) = client
        .get(&url)
        .header("User-Agent", "nio-launcher")
        .send()
        .await
    else {
        return Vec::new();
    };
    if !resp.status().is_success() {
        return Vec::new();
    }
    let Ok(json) = resp.json::<serde_json::Value>().await else {
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
        cache.releases.insert(key.clone(), (Instant::now(), fetched.clone()));
        cache.failures.remove(&key);
    }
    fetched
}

/// Hub-репозиторий с глобальными постами/новостями (GitHub Discussions).
/// Посты также считываются из репозиториев каждой сборки.
const NEWS_REPO: (&str, &str) = ("n1orio", "nio-launcher");

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
    let Ok(resp) = client
        .get(&url)
        .header("User-Agent", "nio-launcher")
        .send()
        .await
    else {
        return Vec::new();
    };
    if !resp.status().is_success() {
        return Vec::new();
    }
    let Ok(json) = resp.json::<serde_json::Value>().await else {
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
                author,
            }
        })
                .collect()
        })
        .map_err(|e| e.to_string())
}

/// Добавляет сборку по URL репозитория GitHub (или прямой ссылке на `.mrpack`).
/// Проверяет, что репозиторий существует и в его релизах есть `.mrpack` и `pack.json`.
async fn add_pack_impl(
    client: &reqwest::Client,
    url: &str,
    name: Option<&str>,
) -> Result<PackDescriptor, String> {
    let url = url.trim().to_string();
    if url.is_empty() || !url.contains("github.com/") {
        return Err("URL должен быть ссылкой на GitHub (например https://github.com/USER/REPO).".into());
    }
    let (owner, repo) =
        parse_github_repo_from_url(&url).ok_or("Не удалось разобрать владельца/репозиторий из URL")?;

    // Запрещаем дубликаты по тому же репозиторию (встроенные и пользовательские).
    for existing in config::all_packs().map_err(|e| e.to_string())? {
        if let Some((o, r)) = parse_github_repo(&existing) {
            if o == owner && r == repo {
                return Err(format!(
                    "Сборка «{}» уже добавлена",
                    existing.name
                ));
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
        .map(|a| a.clone());
    if json_asset.is_none() {
        return Err(
            "В релизе с .mrpack нет файла pack.json с описанием сборки. \
             Загрузите его в тот же релиз (см. пример в разделе «Разработчикам»)."
                .into(),
        );
    }

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
        format!("https://github.com/{owner}/{repo}/releases/download/{}/{asset}", release.tag)
    };

    // Имя из pack.json (если пользователь не задал своё).
    let mut json_name: Option<String> = None;
    if name.is_none() || name.map(str::trim).unwrap_or("").is_empty() {
        if let Some(asset) = &json_asset {
            let json_url = format!(
                "https://github.com/{owner}/{repo}/releases/download/{}/{}",
                release.tag, asset
            );
            if let Ok(resp) = client
                .get(&json_url)
                .header("User-Agent", "nio-launcher")
                .send()
                .await
            {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(n) = json["name"].as_str() {
                        json_name = Some(n.trim().to_string());
                    }
                }
            }
        }
    }

    let id = pack_id_from_repo(&owner, &repo);
    let pack_name = name
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .map(String::from)
        .or(json_name)
        .unwrap_or(repo)
        .trim()
        .to_string();
    config::add_user_pack(&id, &pack_name, &mrpack_url).map_err(|e| e.to_string())?;
    Ok(PackDescriptor {
        id,
        name: pack_name,
        url: mrpack_url,
        builtin: false,
        author: Some(owner),
    })
}

#[tauri::command]
async fn add_pack_command(
    state: State<'_, AppState>,
    url: String,
    name: Option<String>,
) -> Result<PackDescriptor, String> {
    add_pack_impl(&state.client, &url, name.as_deref()).await
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

const DEEP_LINK_SCHEME: &str = "niol";
const DEEP_LINK_PREFIX: &str = "niol://";

/// Мьютекс, чтобы параллельные deep link не добавляли одну сборку дважды.
static ADD_PACK_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

fn add_pack_lock() -> &'static tokio::sync::Mutex<()> {
    ADD_PACK_LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
}

/// Недавно обработанные deep link (защита от двойного срабатывания: одни и те же
/// аргументы приходят и в single-instance callback, и в событие плагина).
static HANDLED_LINKS: OnceLock<std::sync::Mutex<Vec<(String, Instant)>>> = OnceLock::new();

/// Разбирает deep link вида `niol://add-pack?url=<github-url>&name=<имя>`.
/// Параметры percent-encoded; возвращает (url сборки, имя).
fn parse_deep_link(url: &str) -> Option<(String, Option<String>)> {
    let rest = url.strip_prefix(DEEP_LINK_PREFIX)?;
    let (path, query) = rest.split_once('?')?;
    if path != "add-pack" {
        return None;
    }
    let mut pack_url: Option<String> = None;
    let mut name: Option<String> = None;
    for pair in query.split('&') {
        let (key, value) = pair.split_once('=')?;
        match key {
            "url" => pack_url = Some(pct_decode(value)),
            "name" => name = Some(pct_decode(value)),
            _ => {}
        }
    }
    pack_url.map(|u| (u, name))
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
                author,
            },
            true,
        ));
    }
    add_pack_impl(client, pack_url, name)
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
    if slot.as_ref().is_some_and(|(t, _)| t.elapsed() >= Duration::from_secs(30)) {
        *slot = None;
    }
    value
}

/// Обрабатывает deep link: добавляет сборку (или подтверждает существующую)
/// и сообщает фронтенду через событие `pack-added`.
fn handle_deep_link(app: &AppHandle, url: &str) {
    let Some((pack_url, name)) = parse_deep_link(url) else {
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
        let result = ensure_pack_from_link(&client, &pack_url, name.as_deref()).await;
        let payload = match &result {
            Ok((p, already)) => serde_json::json!({
                "ok": true,
                "already": already,
                "id": p.id,
                "name": p.name,
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
        let urls: Vec<tauri::Url> =
            serde_json::from_str(event.payload()).unwrap_or_default();
        for url in urls.iter().filter(|u| u.as_str().starts_with(DEEP_LINK_PREFIX)) {
            handle_deep_link(&handle, url.as_str());
        }
    });

    if let Ok(Some(urls)) = app.deep_link().get_current() {
        for url in urls.iter().filter(|u| u.as_str().starts_with(DEEP_LINK_PREFIX)) {
            handle_deep_link(app, url.as_str());
        }
    }

    for url in std::env::args().skip(1).filter(|a| a.starts_with(DEEP_LINK_PREFIX)) {
        handle_deep_link(app, &url);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_deep_link_with_encoded_url() {
        let (url, name) = parse_deep_link(&format!(
            "niol://add-pack?url={}&name=My%20Pack",
            pct_decode("https%3A%2F%2Fgithub.com%2Fn1orio%2Fnio-pack-example")
        ))
        .unwrap();
        assert_eq!(url, "https://github.com/n1orio/nio-pack-example");
        assert_eq!(name.as_deref(), Some("My Pack"));
    }

    #[test]
    fn parses_deep_link_without_name() {
        let (url, name) = parse_deep_link(
            "niol://add-pack?url=https%3A%2F%2Fgithub.com%2Fn1orio%2Fnio-pack-example",
        )
        .unwrap();
        assert_eq!(url, "https://github.com/n1orio/nio-pack-example");
        assert_eq!(name, None);
    }

    #[test]
    fn rejects_other_paths_and_schemes() {
        assert!(parse_deep_link("niol://install?url=x").is_none());
        assert!(parse_deep_link("https://github.com/n1orio/nio-pack-example").is_none());
        assert!(parse_deep_link("niol://add-pack").is_none());
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
    let active = config::active_version(&pack.id).ok().filter(|v| !v.is_empty());
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
    files::toggle_file(&pack.id, &folder, &name, enabled).map_err(|e| e.to_string())
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
async fn get_news_command(state: State<'_, AppState>) -> Result<Vec<NewsItem>, String> {
    let mut items: Vec<NewsItem> = Vec::new();
    // Глобальные посты лаунчера.
    items.extend(
        fetch_discussions_cached(
            &state.client,
            NEWS_REPO.0,
            NEWS_REPO.1,
            "post",
            "launcher",
            "NIO Launcher",
        )
        .await,
    );
    // Обновления лаунчера (релизы из GitHub Releases).
    for rel in fetch_launcher_releases_cached(&state.client).await {
        items.push(NewsItem {
            kind: "update".into(),
            pack_id: "launcher".into(),
            pack_name: "NIO Launcher".into(),
            title: rel.name,
            body: rel.body,
            url: rel.url,
            tag: Some(rel.tag),
            category: None,
            date: rel.published_at,
        });
    }
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
                fetch_discussions_cached(&state.client, &owner, &repo, "post", &pack.id, &pack.name)
                    .await,
            );
        }
    }
    // Свежие сверху (без даты — вниз).
    items.sort_by(|a, b| match (&b.date, &a.date) {
        (Some(x), Some(y)) => x.cmp(y),
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (None, None) => std::cmp::Ordering::Equal,
    });
    Ok(items)
}

/// Контент репозитория сборки: звёзды GitHub + скриншоты (screenshots.json)
/// + сервера (servers.json). Raw-файлы не тратят лимиты GitHub API.
async fn fetch_pack_repo_content(
    client: &reqwest::Client,
    owner: &str,
    repo: &str,
) -> PackRepoContent {
    let mut out = PackRepoContent::default();

    // Звёзды: один API-вызов на репозиторий (кэшируется).
    let meta_url = format!("https://api.github.com/repos/{owner}/{repo}");
    if let Ok(resp) = client.get(&meta_url).header("User-Agent", "nio-launcher").send().await {
        if resp.status().is_success() {
            if let Ok(v) = resp.json::<serde_json::Value>().await {
                out.stars = v["stargazers_count"].as_i64();
            }
        }
    }

    // Скриншоты: манифест с путями к картинкам в репозитории.
    let shot_url =
        format!("https://raw.githubusercontent.com/{owner}/{repo}/HEAD/screenshots.json");
    if let Ok(resp) = client.get(&shot_url).send().await {
        if resp.status().is_success() {
            if let Ok(v) = resp.json::<serde_json::Value>().await {
                let paths: Vec<String> = v
                    .as_array()
                    .or_else(|| v.get("files").and_then(|f| f.as_array()))
                    .map(|a| {
                        a.iter()
                            .filter_map(|x| x.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();
                const IMG_EXT: [&str; 5] = ["png", "jpg", "jpeg", "webp", "gif"];
                for p in paths {
                    let clean = p.trim_start_matches('/');
                    let is_img = std::path::Path::new(clean)
                        .extension()
                        .map(|e| e.to_string_lossy().to_ascii_lowercase())
                        .is_some_and(|e| IMG_EXT.contains(&e.as_str()));
                    if clean.is_empty() || clean.starts_with("..") || !is_img {
                        continue;
                    }
                    out.screenshots
                        .push(format!("https://raw.githubusercontent.com/{owner}/{repo}/HEAD/{clean}"));
                }
            }
        }
    }

    // Сервера: манифест списка серверов.
    let srv_url = format!("https://raw.githubusercontent.com/{owner}/{repo}/HEAD/servers.json");
    if let Ok(resp) = client.get(&srv_url).send().await {
        if resp.status().is_success() {
            if let Ok(v) = resp.json::<serde_json::Value>().await {
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
                            port: item
                                .get("port")
                                .and_then(|x| x.as_u64())
                                .map(|p| p as u16),
                            desc: item.get("desc").and_then(|x| x.as_str()).map(String::from),
                        });
                    }
                }
            }
        }
    }
    out
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
    cache
        .meta
        .insert(key, (Instant::now(), fetched.clone()));
    fetched
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
async fn switch_version(
    pack_id: Option<String>,
    version_id: String,
) -> Result<(), String> {
    let pack = resolve_pack(pack_id)?;
    if version_id.is_empty() {
        return Err("Пустая версия".into());
    }
    // Разрешаем передавать как versionId, так и тег релиза.
    let resolved = mrpack::installed_details(&pack.id)
        .iter()
        .find(|v| {
            v.version_id == version_id
                || v.source_tag.as_deref() == Some(version_id.as_str())
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
    pack_id: Option<String>,
    tag: Option<String>,
) -> Result<mrpack::PackInfo, String> {
    let pack = resolve_pack(pack_id)?;
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
        None => match fetch_releases_cached(&client, &pack).await.into_iter().next() {
            Some(r) => (r.url, Some(r.tag)),
            None => (pack.url.to_string(), None),
        },
    };
    mrpack::install_mrpack(app, &client, &pack.id, &url, install_tag.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// Определяет, установлена ли сборка и возвращает общий статус.
#[tauri::command]
async fn get_status(pack_id: Option<String>) -> Result<AppStatus, String> {
    let pack = resolve_pack(pack_id)?;
    let mut status = AppStatus {
        mrpack_url: pack.url.to_string(),
        active_version: config::active_version(&pack.id).ok().filter(|v| !v.is_empty()),
        active_source_tag: active_installed_tag(&pack.id),
        installed_versions: mrpack::installed_versions(&pack.id),
        discord_rp_enabled: config::discord_rp_enabled(),
        ..Default::default()
    };

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

        let game_dir = config::version_dir(&pack.id, &idx.version_id)
            .map_err(|e| e.to_string())?;
        status.installed = mrpack::is_installed(&game_dir, &idx);
    }

    if let Some(v) = status.active_version.as_deref() {
        status.custom_mods = mrpack::read_custom_mods(&pack.id, v);
    }

    status.session = auth::load_session().ok().flatten();
    Ok(status)
}

/// Открывает в системном проводнике папку активной версии сборки.
/// Если сборка ещё не установлена — открывает (создавая) папку данных сборки.
#[tauri::command]
fn open_pack_dir(app: AppHandle, pack_id: Option<String>) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let pack = resolve_pack(pack_id)?;
    let dir = config::active_version(&pack.id)
        .ok()
        .filter(|v| !v.is_empty())
        .and_then(|v| config::version_dir(&pack.id, &v).ok())
        .filter(|d| d.exists())
        .or_else(|| {
            config::versions_root(&pack.id)
                .ok()
                .filter(|d| d.exists())
        })
        .or_else(|| config::pack_dir(&pack.id).ok())
        .unwrap_or_else(|| config::launcher_root().unwrap_or_else(|_| std::env::temp_dir()));
    // Если папки нет (сборка не установлена) — создаём, чтобы проводник открыл её.
    let _ = std::fs::create_dir_all(&dir);
    app.opener()
        .open_path(dir.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| e.to_string())
}

/// Оффлайн-логин.
#[tauri::command]
async fn login_offline_command(username: String) -> Result<UserSession, String> {
    let session = login_offline(&username).map_err(|e| e.to_string())?;
    save_session(&session).map_err(|e| e.to_string())?;
    Ok(session)
}

/// Microsoft OAuth2, фаза 1: запрашиваем device code для показа в UI.
#[tauri::command]
async fn ms_device_code_command(
    state: State<'_, AppState>,
) -> Result<auth::MsDeviceCodeInfo, String> {
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
    Ok(session)
}

/// Запуск игры.
#[tauri::command]
async fn launch_game_command(
    app: AppHandle,
    pack_id: Option<String>,
    ram_gb: u32,
    session: UserSession,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let pack_id = pack_id.unwrap_or_else(|| default_pack_id().to_string());
    game::launch_game(&pack_id, ram_gb, session, app, width.max(320), height.max(240))
        .await
        .map_err(|e| e.to_string())
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

/// Версия лаунчера (из Cargo.toml) — для отчётов об ошибках.
#[tauri::command]
fn launcher_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Открывает URL во внешнем браузере.
#[tauri::command]
fn open_url(app: AppHandle, url: String) -> Result<(), String> {
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

/// Скачивает и распаковывает встроенную JRE 21 (если её ещё нет).
#[tauri::command]
async fn ensure_java_command(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    jre::ensure_bundled_java(&app, &state.client)
        .await
        .map_err(|e| e.to_string())
}

/// Проверка целостности файлов активной версии сборки.
#[tauri::command]
fn verify_game_command(pack_id: Option<String>) -> Result<mrpack::VerifyResult, String> {
    let pack = resolve_pack(pack_id)?;
    mrpack::verify_pack(&pack.id).map_err(|e| e.to_string())
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

/// Запоминает язык интерфейса (ru/en) для строк, формируемых на стороне Rust
/// (например, активность Discord).
#[tauri::command]
fn set_locale_command(locale: String) {
    crate::discord_rp::set_locale(locale);
}

/// URL текстуры скина Mojang-профиля по uuid (без даш). None — нет скина/профиля.
#[tauri::command]
async fn get_skin_command(state: State<'_, AppState>, uuid: String) -> Result<Option<String>, String> {
    let url = format!("https://sessionserver.mojang.com/session/minecraft/profile/{uuid}");
    let resp = state
        .client
        .get(&url)
        .header("User-Agent", "nio-launcher")
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
                .and_then(|b64| {
                    base64::engine::general_purpose::STANDARD
                        .decode(b64)
                        .ok()
                })
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
            launch_game_command,
            get_launch_log,
            clear_launch_log,
            open_pack_dir,
            launcher_version,
            open_url,
            list_java_command,
            set_java_path_command,
            ensure_java_command,
            verify_game_command,
            open_game_folder_command,
            get_skin_command,
            set_discord_rp_command,
            set_locale_command,
            get_news_command,
            list_game_files_command,
            toggle_game_file_command,
            get_game_file_icon_command,
            get_game_file_icons_command,
            pack_repo_content_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}