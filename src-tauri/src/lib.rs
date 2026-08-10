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
use tauri::{AppHandle, State};
use sysinfo::System;

use crate::auth::{login_offline, save_session, UserSession};
use crate::config::{default_pack_id, pack_by_id, PackDef};

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
}

/// Публичное описание сборки для фронтенда.
#[derive(Debug, Clone, Serialize)]
pub struct PackDescriptor {
    pub id: String,
    pub name: String,
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

/// Информация о системе для ползунка RAM.
#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub total_ram_gb: u64,
    pub available_ram_gb: u64,
}

/// Определяет сборку по id (или берёт дефолтную из конфига).
fn resolve_pack(pack_id: Option<String>) -> Result<&'static PackDef, String> {
    let id = pack_id.unwrap_or_else(|| default_pack_id().to_string());
    pack_by_id(&id).ok_or_else(|| format!("Сборка не найдена: {id}"))
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
fn parse_github_repo(pack: &PackDef) -> Option<(String, String)> {
    let rest = pack
        .url
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/");
    let mut parts = rest.split('/');
    let owner = parts.next()?;
    let repo = parts.next()?.split('/').next()?;
    if owner == "USER" || repo == "REPO" {
        return None;
    }
    Some((owner.to_string(), repo.to_string()))
}

/// Имя `.mrpack` файла из URL сборки.
fn mrpack_file_name(pack: &PackDef) -> Option<String> {
    std::path::Path::new(pack.url)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
}

/// URL скачивания `.mrpack` для конкретного тега релиза.
fn mrpack_url_for_tag(pack: &PackDef, tag: &str) -> String {
    let file = mrpack_file_name(pack).unwrap_or_else(|| "modpack.mrpack".into());
    let Some((owner, repo)) = parse_github_repo(pack) else {
        return pack.url.to_string();
    };
    format!("https://github.com/{owner}/{repo}/releases/download/{tag}/{file}")
}

const API_HIT_TTL: Duration = Duration::from_secs(15 * 60);
const API_FAIL_RETRY: Duration = Duration::from_secs(60);

/// Кэш GitHub API: успешные ответы живут 15 минут, ошибки (в т.ч. rate limit)
/// не дают долбить API чаще раза в минуту.
struct ApiCache {
    releases: HashMap<String, (Instant, Vec<GhVersion>)>,
    discussions: HashMap<String, (Instant, Vec<NewsItem>)>,
    failures: HashMap<String, Instant>,
}

static API_CACHE: OnceLock<std::sync::Mutex<ApiCache>> = OnceLock::new();

fn api_cache() -> &'static std::sync::Mutex<ApiCache> {
    API_CACHE.get_or_init(|| {
        std::sync::Mutex::new(ApiCache {
            releases: HashMap::new(),
            discussions: HashMap::new(),
            failures: HashMap::new(),
        })
    })
}

fn repo_key(pack: &PackDef) -> Option<String> {
    parse_github_repo(pack).map(|(o, r)| format!("{o}/{r}"))
}

/// `fetch_releases` с кэшем: повторные запросы в течение TTL не ходят в сеть,
/// а при сбое (403 rate limit, нет сети) повторная попытка — не раньше чем через минуту.
async fn fetch_releases_cached(client: &reqwest::Client, pack: &PackDef) -> Vec<GhVersion> {
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

/// Релизы сборки с GitHub (тег + ченджлог + дата).
async fn fetch_releases(client: &reqwest::Client, pack: &PackDef) -> Vec<GhVersion> {
    let Some((owner, repo)) = parse_github_repo(pack) else {
        return Vec::new();
    };
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
        out.push(GhVersion {
            url: mrpack_url_for_tag(pack, &tag),
            tag,
            name,
            prerelease: rel["prerelease"].as_bool().unwrap_or(false),
            published_at: rel["published_at"].as_str().map(|s| s.to_string()),
            body: rel["body"].as_str().unwrap_or("").to_string(),
        });
    }
    out
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

/// Список поддерживаемых сборок.
#[tauri::command]
fn list_packs() -> Vec<PackDescriptor> {
    config::PACKS
        .iter()
        .map(|p| PackDescriptor {
            id: p.id.to_string(),
            name: p.name.to_string(),
        })
        .collect()
}

/// Все версии сборки: релизы GitHub + установленные + активная.
#[tauri::command]
async fn list_versions(
    state: State<'_, AppState>,
    pack_id: Option<String>,
) -> Result<VersionsInfo, String> {
    let pack = resolve_pack(pack_id)?;
    let installed = mrpack::installed_details(pack.id);
    let active = config::active_version(pack.id).ok().filter(|v| !v.is_empty());
    let github = fetch_releases_cached(&state.client, pack).await;
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
    files::list_files(pack.id, &folder).map_err(|e| e.to_string())
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
    files::toggle_file(pack.id, &folder, &name, enabled).map_err(|e| e.to_string())
}

/// Иконка файла/папки (data-URL PNG) для отображения в списках.
#[tauri::command]
fn get_game_file_icon_command(
    pack_id: Option<String>,
    folder: String,
    name: String,
) -> Result<Option<String>, String> {
    let pack = resolve_pack(pack_id)?;
    files::file_icon(pack.id, &folder, &name).map_err(|e| e.to_string())
}

/// Иконки пачки файлов одним вызовом (data-URL PNG или None).
#[tauri::command]
fn get_game_file_icons_command(
    pack_id: Option<String>,
    folder: String,
    names: Vec<String>,
) -> Result<Vec<files::GameFileIcon>, String> {
    let pack = resolve_pack(pack_id)?;
    Ok(files::file_icons(pack.id, &folder, &names))
}

/// Лента новостей: релизы (обновления) + посты всех сборок и глобальные посты
/// hub-репозитория лаунчера, свежие сверху.
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
    for pack in config::PACKS {
        for rel in fetch_releases_cached(&state.client, pack).await {
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
        if let Some((owner, repo)) = parse_github_repo(pack) {
            items.extend(
                fetch_discussions_cached(&state.client, &owner, &repo, "post", pack.id, pack.name)
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
    let resolved = mrpack::installed_details(pack.id)
        .iter()
        .find(|v| {
            v.version_id == version_id
                || v.source_tag.as_deref() == Some(version_id.as_str())
        })
        .map(|v| v.version_id.clone())
        .ok_or_else(|| format!("Версия {version_id} не установлена"))?;
    config::set_active_version(pack.id, &resolved).map_err(|e| e.to_string())
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
    let current = active_installed_tag(pack.id);
    let releases = fetch_releases_cached(&state.client, pack).await;
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
        Some(t) => (mrpack_url_for_tag(pack, t), Some(t.clone())),
        // latest в GitHub не перенаправляет на пререлизы, поэтому берём
        // самый свежий релиз из API (включая пререлизы). А tag записываем
        // в маркер установки — иначе лаунчер будет вечно «обнаруживать обновление».
        None => match fetch_releases_cached(&client, pack).await.into_iter().next() {
            Some(r) => (r.url, Some(r.tag)),
            None => (pack.url.to_string(), None),
        },
    };
    mrpack::install_mrpack(app, &client, pack.id, &url, install_tag.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// Определяет, установлена ли сборка и возвращает общий статус.
#[tauri::command]
async fn get_status(pack_id: Option<String>) -> Result<AppStatus, String> {
    let pack = resolve_pack(pack_id)?;
    let mut status = AppStatus {
        mrpack_url: pack.url.to_string(),
        active_version: config::active_version(pack.id).ok().filter(|v| !v.is_empty()),
        active_source_tag: active_installed_tag(pack.id),
        installed_versions: mrpack::installed_versions(pack.id),
        discord_rp_enabled: config::discord_rp_enabled(),
        ..Default::default()
    };

    // Читаем метаданные активной версии из её папки.
    if let Some(idx) = status
        .active_version
        .as_ref()
        .and_then(|v| mrpack::read_version_index(pack.id, v))
    {
        status.pack_name = Some(idx.name.clone());
        status.minecraft_version = idx.dependencies.get("minecraft").cloned();
        status.loader = ["fabric-loader", "forge", "neoforge", "quilt"]
            .iter()
            .find(|k| idx.dependencies.contains_key(**k))
            .map(|k| k.replace("-loader", ""));

        let game_dir = config::version_dir(pack.id, &idx.version_id)
            .map_err(|e| e.to_string())?;
        status.installed = mrpack::is_installed(&game_dir, &idx);
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
    let dir = config::active_version(pack.id)
        .ok()
        .filter(|v| !v.is_empty())
        .and_then(|v| config::version_dir(pack.id, &v).ok())
        .filter(|d| d.exists())
        .or_else(|| {
            config::versions_root(pack.id)
                .ok()
                .filter(|d| d.exists())
        })
        .or_else(|| config::pack_dir(pack.id).ok())
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
    mrpack::verify_pack(pack.id).map_err(|e| e.to_string())
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
    let dir = config::active_game_dir(pack.id)
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
        .invoke_handler(tauri::generate_handler![
            list_packs,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}