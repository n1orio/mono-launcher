//! CurseForge: поиск файлов (моды/ресурспаки/шейдеры) и установка в сборку.
//!
//! API v1 (api.curseforge.com) требует ключ: `x-api-key`. Общий ключ встроен
//! в лаунчер (константа CURSEFORGE_API_KEY); переопределяется переменной
//! окружения MONO_CURSEFORGE_KEY (напр. в CI).
//!
//! Получить ключ: console.curseforge.com → API keys (нужен аккаунт Twitch/CurseForge).
//! Файлы скачиваются с CDN forgecdn.net — отдельный доступ не нужен.

use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::config;

const API_BASE: &str = "https://api.curseforge.com/v1";

/// Маркер «проект/файл удалён на CurseForge» — такие записи манифеста
/// пропускаем, в отличие от сетевых/серверных ошибок.
const CF_NOT_FOUND: &str = "CF_NOT_FOUND";
const GAME_MINECRAFT: u32 = 432;

/// Классы проектов Minecraft на CurseForge.
pub const CLASS_MODS: u32 = 6;
/// Используются для фильтрации и показа расширений файлов (см. тесты).
#[allow(dead_code)]
pub const CLASS_RESOURCEPACKS: u32 = 12;
#[allow(dead_code)]
pub const CLASS_SHADERPACKS: u32 = 6552;
#[allow(dead_code)]
pub const CLASS_MODPACKS: u32 = 4471;

/// Запись об установленном вручную файле CurseForge (для показа меты в списке).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseTracked {
    pub file_name: String,
    pub folder: String,
    pub project_id: u32,
    /// Название проекта (из поискового хита) — чтобы не ходить за метой на API.
    #[serde(default)]
    pub title: String,
    /// URL логотипа проекта (миниатюра), из поискового хита.
    #[serde(default)]
    pub icon: String,
}

/// Путь к файлу трекинга установленных вручную CurseForge-файлов.
fn curse_track_file(pack_id: &str) -> Result<std::path::PathBuf> {
    Ok(config::active_game_dir(pack_id)?.join(".mono-curseforge.json"))
}

/// Текущий список отслеживаемых CurseForge-файлов активной версии.
fn tracked(pack_id: &str) -> Vec<CurseTracked> {
    let Ok(path) = curse_track_file(pack_id) else {
        return Vec::new();
    };
    if !path.exists() {
        return Vec::new();
    }
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

fn save_tracked(pack_id: &str, mods: &[CurseTracked]) {
    let Ok(path) = curse_track_file(pack_id) else {
        return;
    };
    if let Some(parent) = path.parent() {
        if std::fs::create_dir_all(parent).is_err() {
            return;
        }
    }
    let _ = std::fs::write(&path, serde_json::to_string(mods).unwrap_or_default());
}

/// Добавляет/обновляет запись (имя файла + папка → project_id).
pub fn upsert_tracked(pack_id: &str, t: &CurseTracked) {
    let mut mods = tracked(pack_id);
    if let Some(existing) = mods
        .iter_mut()
        .find(|m| m.file_name == t.file_name && m.folder == t.folder)
    {
        *existing = t.clone();
    } else {
        mods.push(t.clone());
    }
    save_tracked(pack_id, &mods);
}

/// Мета отслеживаемого файла папки игры: «имя файла (включая .disabled)» → (project_id, title, icon).
pub fn tracked_meta(
    pack_id: &str,
    folder: &str,
) -> std::collections::HashMap<String, (u32, String, String)> {
    tracked(pack_id)
        .into_iter()
        .filter(|m| m.folder == folder)
        .map(|m| {
            (
                m.file_name.clone(),
                (m.project_id, m.title.clone(), m.icon.clone()),
            )
        })
        .collect()
}

/// Общий API-ключ CurseForge, встроенный в лаунчер (чтобы всем пользователям
/// не нужно было вводить свой). Заполните своим значением вместо `CHANGE_ME`.
/// Можно переопределить переменной окружения MONO_CURSEFORGE_KEY (напр. в CI).
const CURSEFORGE_API_KEY: &str = "$2a$10$xSHIQILV.MP7ms3Rld9qn.IGY.UrQW996e9T2vWKgH6q.j6DXISlK";

/// API-ключ CurseForge. Приоритет (сверху вниз):
/// 1) переменная окружения MONO_CURSEFORGE_KEY,
/// 2) локальный файл `curseforge-key.txt` в корне репозитория (не коммитится),
/// 3) встроенная константа CURSEFORGE_API_KEY.
pub fn api_key_from_cfg() -> Option<String> {
    if let Ok(env) = std::env::var("MONO_CURSEFORGE_KEY") {
        let t = env.trim().to_string();
        if !t.is_empty() && t != "CHANGE_ME" {
            return Some(t);
        }
    }
    // Файл в корне репозитория (для локальной разработки без шитья секрета
    // в исходники). Файл добавлен в .gitignore.
    if let Ok(raw) = std::fs::read_to_string("curseforge-key.txt") {
        let t = raw.trim().to_string();
        if !t.is_empty() && t != "CHANGE_ME" {
            return Some(t);
        }
    }
    let t = CURSEFORGE_API_KEY.trim();
    if !t.is_empty() && t != "CHANGE_ME" {
        Some(t.to_string())
    } else {
        None
    }
}

fn require_api_key() -> Result<String> {
    api_key_from_cfg().ok_or_else(|| {
        anyhow!(
            "CurseForge требует API-ключ.\n\
             Получите его на console.curseforge.com → API keys (бесплатно, нужен аккаунт CurseForge),\n\
             затем впишите его в константу CURSEFORGE_API_KEY в src-tauri/src/curseforge.rs."
        )
    })
}

fn ua() -> String {
    format!("mono-launcher/{}", env!("CARGO_PKG_VERSION"))
}

/// Расширение бандла для класса проектов.
fn file_ext_for_class(class_id: u32) -> &'static str {
    match class_id {
        CLASS_MODS => "jar",
        _ => "zip",
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseSearchHit {
    pub project_id: u32,
    pub name: String,
    pub summary: String,
    pub author: String,
    pub download_count: u64,
    /// Расширение бандла: jar для модов, zip для ресурспаков/шейдеров.
    pub file_ext: String,
    /// URL логотипа проекта (миниатюра 256x256).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SearchResp {
    data: Vec<SearchItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchItem {
    id: u32,
    name: String,
    summary: String,
    download_count: u64,
    authors: Vec<Author>,
    #[serde(default)]
    #[allow(dead_code)]
    latest_files_indexes: Vec<LatestFileIndex>,
    #[serde(default)]
    logo: Option<SearchLogo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchLogo {
    thumbnail_url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Author {
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LatestFileIndex {
    #[allow(dead_code)]
    game_version: String,
    #[allow(dead_code)]
    file_id: u32,
}

/// Поиск проектов по классу (моды/ресурспаки/шейдеры/сборки).
///
/// `sort_field` — числовой `sortField` CurseForge (по умолчанию "2" — популярность);
/// `game_version` — версия Minecraft (например "1.20.1") для фильтра `gameVersion`.
/// Всегда передаётся `sortOrder=desc`, иначе CurseForge не применяет сортировку.
pub async fn search(
    client: &reqwest::Client,
    query: &str,
    class_id: u32,
    category_id: Option<u32>,
    game_version: Option<&str>,
    sort_field: Option<&str>,
) -> Result<Vec<CurseSearchHit>> {
    let key = require_api_key()?;
    let mut req = client
        .get(format!("{API_BASE}/mods/search"))
        .header("x-api-key", &key)
        .header("User-Agent", ua())
        .query(&[
            ("gameId", GAME_MINECRAFT.to_string()),
            ("classId", class_id.to_string()),
            ("searchFilter", query.trim().to_string()),
            ("pageSize", "20".into()),
            ("sortField", sort_field.unwrap_or("2").to_string()),
            ("sortOrder", "desc".to_string()),
        ]);
    if let Some(cat) = category_id {
        req = req.query(&[("categoryId", cat.to_string())]);
    }
    if let Some(gv) = game_version.filter(|v| !v.trim().is_empty()) {
        req = req.query(&[("gameVersion", gv.trim())]);
    }
    let resp = req
        .send()
        .await
        .context("Не удалось связаться с CurseForge")?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        let tip = match status.as_u16() {
            401 | 403 => {
                "ключ не принят, либо CurseForge временно ограничивает поиск \
                 (лимит запросов или сбой на их стороне — поиск может \
                 «отвалиться» даже при валидном ключе; проверьте ключ и повторите позже)"
            }
            429 => "CurseForge ограничил частоту запросов — повторите чуть позже",
            _ => "повторите позже",
        };
        return Err(anyhow!(
            "Поиск CurseForge ответил HTTP {status} ({tip}). Ответ сервера: {body}"
        ));
    }
    let resp: SearchResp = resp.json().await?;
    let file_ext = file_ext_for_class(class_id);
    Ok(resp
        .data
        .into_iter()
        .map(|i| CurseSearchHit {
            project_id: i.id,
            name: i.name,
            summary: i.summary,
            author: i.authors.first().map(|a| a.name.clone()).unwrap_or_default(),
            download_count: i.download_count,
            file_ext: file_ext.into(),
            icon_url: i
                .logo
                .and_then(|l| (!l.thumbnail_url.is_empty()).then_some(l.thumbnail_url)),
        })
        .collect())
}

/// Категория проекта CurseForge (для фильтра поиска).
#[derive(Debug, Clone, Serialize)]
pub struct CurseCategory {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct CategoriesResp {
    data: Vec<CategoryItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CategoryItem {
    id: u32,
    name: String,
    #[serde(default)]
    class_id: Option<u32>,
}

/// Категории класса проектов (для фильтра поиска на фронтенде).
pub async fn categories(
    client: &reqwest::Client,
    class_id: u32,
) -> Result<Vec<CurseCategory>> {
    let key = require_api_key()?;
    let resp: CategoriesResp = client
        .get(format!("{API_BASE}/categories"))
        .header("x-api-key", &key)
        .header("User-Agent", ua())
        .query(&[("gameId", GAME_MINECRAFT.to_string())])
        .send()
        .await
        .context("Не удалось получить категории CurseForge")?
        .error_for_status()
        .context("CurseForge отклонил запрос (проверьте API-ключ)")?
        .json()
        .await?;
    let mut out: Vec<CurseCategory> = resp
        .data
        .into_iter()
        .filter(|c| c.class_id == Some(class_id))
        .map(|c| CurseCategory { id: c.id, name: c.name })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// Файл сборки CurseForge (для выбора версии).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CursePackFile {
    pub file_id: u32,
    pub file_name: String,
    pub display_name: String,
    pub game_version: String,
    pub file_date: String,
}

/// Файлы проекта (последние 50), отсортированные по дате (новые сверху).
#[derive(Debug, Deserialize)]
struct FilesResp {
    data: Vec<FileItem>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileItem {
    id: u32,
    display_name: String,
    file_name: String,
    is_available: bool,
    download_url: Option<String>,
    #[serde(default)]
    game_versions: Vec<String>,
    file_date: String,
    #[serde(default)]
    hashes: Vec<FileHash>,
    #[serde(default)]
    dependencies: Vec<FileDependency>,
}

/// Зависимость файла CurseForge (для автодокачки required-зависимостей).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // поля уходят во фронтенд по IPC
pub struct FileDependency {
    pub mod_id: Option<u32>,
    /// 1=embedded library, 2=optional, 3=required, 4=tool, 5=incompatible, 6=include.
    pub relation_type: u32,
}

impl FileDependency {
    /// Сколько уровней зависимостей доустанавливаем автоматически.
    pub const MAX_DEPTH: u32 = 3;
    pub const REQUIRED: u32 = 3;

    pub fn is_required(&self) -> bool {
        self.relation_type == Self::REQUIRED && self.mod_id.is_some()
    }
}

#[derive(Debug, Clone, Deserialize)]
struct FileHash {
    algo: u32, // 1 = sha1, 2 = sha512
    value: String,
}

/// Файл, готовый к установке.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // поля уходят во фронтенд по IPC
pub struct CurseFile {
    pub file_id: u32,
    pub project_id: u32,
    pub file_name: String,
    pub download_url: String,
    pub sha1: String,
    pub game_version: String,
    /// Required-зависимости этого файла (для автодокачки).
    #[serde(default)]
    pub dependencies: Vec<FileDependency>,
}

/// Результат установки файла CurseForge: имя файла + число автодокачанных зависимостей.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallResult {
    pub name: String,
    pub deps_installed: u32,
}

/// Файлы проекта для выбора версии сборки (последние 50, новые сверху).
pub async fn pack_files(
    client: &reqwest::Client,
    project_id: u32,
) -> Result<Vec<CursePackFile>> {
    let key = require_api_key()?;
    let resp: FilesResp = client
        .get(format!("{API_BASE}/mods/{project_id}/files"))
        .header("x-api-key", &key)
        .header("User-Agent", ua())
        .query(&[("pageSize", "50")])
        .send()
        .await
        .context("Не удалось получить файлы проекта CurseForge")?
        .error_for_status()
        .context("CurseForge отклонил запрос (проверьте API-ключ)")?
        .json()
        .await?;
    let mut out: Vec<CursePackFile> = resp
        .data
        .into_iter()
        .filter(|f| f.is_available)
        .map(|f| CursePackFile {
            file_id: f.id,
            file_name: f.file_name,
            display_name: f.display_name,
            game_version: f.game_versions.first().cloned().unwrap_or_default(),
            file_date: f.file_date,
        })
        .collect();
    out.sort_by(|a, b| b.file_date.cmp(&a.file_date));
    Ok(out)
}

fn curse_file_from_item(project_id: u32, f: FileItem) -> CurseFile {
    let sha1 = f
        .hashes
        .iter()
        .find(|h| h.algo == 1)
        .map(|h| h.value.clone())
        .unwrap_or_default();
    CurseFile {
        file_id: f.id,
        project_id,
        file_name: f.file_name,
        download_url: f.download_url.unwrap_or_else(|| {
            format!("https://www.curseforge.com/minecraft/mc-mods/{project_id}/download/{}/file", f.id)
        }),
        sha1,
        game_version: f.game_versions.first().cloned().unwrap_or_default(),
        dependencies: f.dependencies,
    }
}

/// Устанавливает required-зависимости файла CurseForge (рекурсивно, до
/// `FileDependency::MAX_DEPTH` уровней). Возвращает количество установленных.
/// Уже стоящие файлы (по имени в папке) пропускаются; циклы отсекаются по project_id.
pub async fn install_dependencies(
    client: &reqwest::Client,
    project_id: u32,
    file_id: u32,
    mc_version: Option<&str>,
    dest_dir: &Path,
) -> Result<u32, String> {
    let mut installed = 0u32;
    let mut visited = std::collections::HashSet::new();
    let mut stack: Vec<(u32, u32, u32)> = vec![(project_id, file_id, 0)];
    while let Some((pid, fid, depth)) = stack.pop() {
        if depth >= FileDependency::MAX_DEPTH {
            continue;
        }
        let file = file_by_id(client, pid, fid)
            .await
            .map_err(|e| e.to_string())?;
        for dep in file.dependencies {
            if !dep.is_required() {
                continue;
            }
            let Some(mod_id) = dep.mod_id else { continue };
            if !visited.insert(mod_id) {
                continue;
            }
            let dep_file = latest_file(client, mod_id, mc_version).await.map_err(|e| {
                format!("Не удалось найти зависимость проекта {mod_id} CurseForge: {e}")
            })?;
            let name = dep_file
                .file_name
                .rsplit('/')
                .next()
                .unwrap_or(&dep_file.file_name)
                .to_string();
            if dest_dir.join(&name).exists() {
                continue;
            }
            download_to(client, &dep_file, dest_dir).await.map_err(|e| {
                format!("Не удалось скачать зависимость «{name}» CurseForge: {e}")
            })?;
            installed += 1;
            stack.push((dep_file.project_id, dep_file.file_id, depth + 1));
        }
    }
    Ok(installed)
}

/// Файл проекта по id (для установки конкретной версии сборки).
pub async fn file_by_id(
    client: &reqwest::Client,
    project_id: u32,
    file_id: u32,
) -> Result<CurseFile> {
    let key = require_api_key()?;
    let resp = client
        .get(format!("{API_BASE}/mods/{project_id}/files/{file_id}"))
        .header("x-api-key", &key)
        .header("User-Agent", ua())
        .send()
        .await
        .context("Не удалось получить файл проекта CurseForge")?;
    let status = resp.status();
    if status == reqwest::StatusCode::NOT_FOUND || status == reqwest::StatusCode::GONE {
        // Удалённый на платформе проект/файл — известное отсутствие (не сеть и не рейт-лимит).
        return Err(anyhow!("{CF_NOT_FOUND}"));
    }
    let resp: SingleFileResp = resp
        .error_for_status()
        .context("CurseForge отклонил запрос (проверьте API-ключ)")?
        .json()
        .await?;
    Ok(curse_file_from_item(project_id, resp.data))
}

#[derive(Debug, Deserialize)]
struct SingleFileResp {
    data: FileItem,
}

/// Проект CurseForge (имя + логотип — для установки сборки).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CfProject {
    pub id: u32,
    pub name: String,
    pub logo_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ProjectResp {
    data: ProjectItem,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectItem {
    id: u32,
    name: String,
    slug: String,
    summary: String,
    description: String,
    logo: Option<LogoItem>,
    #[serde(default)]
    screenshots: Vec<LogoItem>,
    #[serde(default)]
    categories: Vec<CategoryItem>,
    #[serde(default)]
    authors: Vec<Author>,
    download_count: u64,
    links: ProjectLinks,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LogoItem {
    url: String,
    #[serde(default)]
    thumbnail_url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectLinks {
    website_url: String,
}

/// Полное описание проекта CurseForge (деталка сборки: описание/скриншоты).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseProjectDetail {
    pub project_id: u32,
    pub name: String,
    pub slug: String,
    pub summary: String,
    pub description: String,
    pub author: String,
    pub download_count: u64,
    pub icon_url: Option<String>,
    /// URL-ы скриншотов (полные).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub screenshots: Vec<String>,
    /// Категории (имена).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,
    pub website_url: String,
}

pub async fn project(client: &reqwest::Client, project_id: u32) -> Result<CfProject> {
    let key = require_api_key()?;
    let resp: ProjectResp = client
        .get(format!("{API_BASE}/mods/{project_id}"))
        .header("x-api-key", &key)
        .header("User-Agent", ua())
        .send()
        .await
        .context("Не удалось получить проект CurseForge")?
        .error_for_status()
        .context("CurseForge отклонил запрос (проверьте API-ключ)")?
        .json()
        .await?;
    Ok(CfProject {
        id: resp.data.id,
        name: resp.data.name,
        logo_url: resp.data.logo.map(|l| {
            if !l.thumbnail_url.is_empty() {
                l.thumbnail_url
            } else {
                l.url
            }
        }),
    })
}

pub async fn project_detail(
    client: &reqwest::Client,
    project_id: u32,
) -> Result<CurseProjectDetail> {
    let key = require_api_key()?;
    let resp: ProjectResp = client
        .get(format!("{API_BASE}/mods/{project_id}"))
        .header("x-api-key", &key)
        .header("User-Agent", ua())
        .send()
        .await
        .context("Не удалось получить проект CurseForge")?
        .error_for_status()
        .context("CurseForge отклонил запрос (проверьте API-ключ)")?
        .json()
        .await?;
    let d = resp.data;
    Ok(CurseProjectDetail {
        project_id: d.id,
        name: d.name,
        slug: d.slug,
        summary: d.summary,
        description: d.description,
        author: d.authors.first().map(|a| a.name.clone()).unwrap_or_default(),
        download_count: d.download_count,
        icon_url: d
            .logo
            .as_ref()
            .map(|l| {
                if !l.thumbnail_url.is_empty() {
                    l.thumbnail_url.clone()
                } else {
                    l.url.clone()
                }
            }),
        screenshots: d.screenshots.into_iter().map(|s| s.url).collect(),
        categories: d.categories.into_iter().map(|c| c.name).collect(),
        website_url: d.links.website_url,
    })
}

/// Выбирает подходящий файл: последний по дате; при указании версии
/// Minecraft — последний файл с этой версией в списке поддерживаемых.
pub async fn latest_file(
    client: &reqwest::Client,
    project_id: u32,
    mc_version: Option<&str>,
) -> Result<CurseFile> {
    let key = require_api_key()?;
    let resp: FilesResp = client
        .get(format!("{API_BASE}/mods/{project_id}/files"))
        .header("x-api-key", &key)
        .header("User-Agent", ua())
        .query(&[("pageSize", "50")])
        .send()
        .await
        .context("Не удалось получить файлы проекта CurseForge")?
        .error_for_status()
        .context("CurseForge отклонил запрос (проверьте API-ключ)")?
        .json()
        .await?;

    let files: Vec<(String, FileItem)> = resp
        .data
        .into_iter()
        .filter(|f| f.is_available && f.download_url.is_some())
        .map(|f| (f.file_date.clone(), f))
        .collect();
    let Some(f) = pick_latest(files, mc_version) else {
        return Err(anyhow!("У проекта нет доступных файлов"));
    };

    Ok(curse_file_from_item(project_id, f))
}

/// Кандидаты (дата, файл) → выбирает последний, при `mc_version` — последний
/// с подходящей версией Minecraft (иначе последний вообще).
fn pick_latest(files: Vec<(String, FileItem)>, mc_version: Option<&str>) -> Option<FileItem> {
    let mut files = files;
    files.sort_by(|a, b| b.0.cmp(&a.0));
    match mc_version {
        Some(mc) => {
            let prefix = format!("{mc}.");
            files
                .iter()
                .find(|(_, f)| f.game_versions.iter().any(|g| g == mc || g.starts_with(&prefix)))
                .map(|(_, f)| f.clone())
                .or_else(|| files.first().map(|(_, f)| f.clone()))
        }
        None => files.first().map(|(_, f)| f.clone()),
    }
}

/// Скачивает файл с CDN CurseForge в папку сборки, проверяя sha1.
/// Возвращает имя файла.
pub async fn download_to(
    client: &reqwest::Client,
    file: &CurseFile,
    dest_dir: &Path,
) -> Result<String> {
    use sha1::Digest;
    let name = file
        .file_name
        .rsplit('/')
        .next()
        .unwrap_or(&file.file_name)
        .to_string();
    let resp = client
        .get(&file.download_url)
        .header("User-Agent", ua())
        .send()
        .await
        .context("Не удалось скачать файл с CurseForge")?
        .error_for_status()
        .context("CurseForge CDN вернул ошибку")?;
    let bytes = resp.bytes().await.context("Не удалось прочитать файл")?;
    if !file.sha1.is_empty() {
        let mut hasher = sha1::Sha1::new();
        hasher.update(&bytes);
        let actual = format!("{:x}", hasher.finalize());
        if actual.to_lowercase() != file.sha1.to_lowercase() {
            return Err(anyhow!(
                "Хэш файла не совпал с CurseForge (ожидался {}, получен {actual}). Файл не установлен.",
                file.sha1
            ));
        }
    }
    std::fs::create_dir_all(dest_dir)?;
    let dest = dest_dir.join(&name);
    if dest.exists() {
        return Err(anyhow!(
            "Файл {name} уже есть в папке — удалите его или включите обновления"
        ));
    }
    std::fs::write(&dest, bytes)
        .with_context(|| format!("Не удалось записать {name}"))?;
    Ok(name)
}

// ---------------------------------------------------------------------------
// Установка сборки (modpack) как отдельной сборки лаунчера.
// ---------------------------------------------------------------------------

/// `manifest.json` из архива сборки CurseForge.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfManifest {
    minecraft: CfMinecraft,
    #[serde(default)]
    mod_loaders: Vec<CfModLoader>,
    files: Vec<CfManifestFile>,
    name: String,
    version: String,
}

#[derive(Debug, Deserialize)]
struct CfMinecraft {
    version: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfModLoader {
    id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfManifestFile {
    project_id: u32,
    file_id: u32,
}

/// Маппит загрузчик CurseForge («forge-40.1.0», «fabric-0.14.21»,
/// «neoforge-47.1.106», «quilt-0.24.2») на ключ зависимостей индекса лаунчера.
fn cf_loader_to_dep(id: &str) -> Option<(String, String)> {
    let (name, ver) = id.split_once('-')?;
    if ver.is_empty() {
        return None;
    }
    let key = match name {
        "forge" => "forge",
        "fabric" => "fabric-loader",
        "neoforge" => "neoforge",
        "quilt" => "quilt-loader",
        _ => return None,
    };
    Some((key.to_string(), ver.to_string()))
}

/// Скачивает zip-архив сборки с CDN (с прогрессом во фронтенд).
async fn download_zip(
    app: &tauri::AppHandle,
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
) -> Result<()> {
    use futures::StreamExt;
    use tokio::io::AsyncWriteExt;
    let resp = client
        .get(url)
        .header("User-Agent", ua())
        .send()
        .await
        .context("Не удалось скачать архив сборки с CurseForge")?
        .error_for_status()
        .context("CurseForge CDN вернул ошибку")?;
    let total = resp.content_length().unwrap_or(0);
    let mut stream = resp.bytes_stream();
    let mut file = tokio::fs::File::create(dest).await?;
    let mut downloaded: u64 = 0;
    let mut last_report = std::time::Instant::now();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Ошибка чтения потока скачивания")?;
        downloaded += chunk.len() as u64;
        file.write_all(&chunk).await?;
        if last_report.elapsed().as_millis() >= 150 {
            crate::mrpack::emit_progress(
                app,
                &crate::mrpack::DownloadProgress {
                    phase: "Скачивание сборки CurseForge".into(),
                    current: downloaded,
                    total,
                    file_index: 0,
                    file_total: 1,
                    current_file: "modpack.zip".into(),
                    bytes_per_sec: 0,
                },
            );
            last_report = std::time::Instant::now();
        }
    }
    file.flush().await?;
    Ok(())
}

/// Догружает файлы манифеста со ссылками и хэшами (проекты могут быть удалены —
/// такие файлы пропускаем, остальные обязаны отдаться).
async fn resolve_manifest_files(
    client: &reqwest::Client,
    entries: &[CfManifestFile],
) -> Result<Vec<crate::mrpack::IndexFile>> {
    use std::sync::Arc;
    let semaphore = Arc::new(tokio::sync::Semaphore::new(8));
    let mut tasks: Vec<tokio::task::JoinHandle<Result<Option<crate::mrpack::IndexFile>>>> = Vec::new();
    for e in entries.iter() {
        let client = client.clone();
        let semaphore = semaphore.clone();
        let project_id = e.project_id;
        let file_id = e.file_id;
        tasks.push(tokio::spawn(async move {
            let _permit = semaphore.acquire().await.map_err(|_| anyhow!("sema"))?;
            let file = match file_by_id(&client, project_id, file_id).await {
                Ok(f) => f,
                // Проект/файл удалён на платформе — пропускаем (известное отсутствие).
                Err(e) if e.to_string().starts_with(CF_NOT_FOUND) => return Ok(None),
                // Сеть, рейт-лимит, 5xx — не глотаем, прерываем установку.
                Err(e) => {
                    return Err(anyhow!(
                        "CurseForge не отдал файл проекта {project_id} ({file_id}): {e}"
                    ))
                }
            };
            if file.download_url.is_empty() {
                return Ok(None);
            }
            let mut hashes = std::collections::HashMap::new();            if !file.sha1.is_empty() {
                hashes.insert("sha1".to_string(), file.sha1);
            }
            let name = file
                .file_name
                .rsplit('/')
                .next()
                .unwrap_or(&file.file_name)
                .to_string();
            Ok(Some(crate::mrpack::IndexFile {
                path: format!("mods/{name}"),
                hashes,
                downloads: vec![file.download_url],
                url: None,
                file_size: 0,
                env: None,
            }))
        }));
    }
    let mut out = Vec::new();
    for t in tasks {
        if let Some(f) = t.await.map_err(|e| anyhow!("Задача сломалась: {e}"))?? {
            out.push(f);
        }
    }
    Ok(out)
}

/// Скачивает и устанавливает сборку CurseForge как отдельную сборку
/// (id = `cf-<projectId>`). Внутри — та же механика, что у .mrpack:
/// файлы качаются с доверенного CDN и проверяются по хэшам.
pub async fn install_modpack(
    app: &tauri::AppHandle,
    client: &reqwest::Client,
    pack_id: &str,
    project_id: u32,
    file_id: u32,
) -> Result<crate::mrpack::PackInfo> {
    let file = file_by_id(client, project_id, file_id).await?;

    crate::mrpack::emit_progress(
        app,
        &crate::mrpack::DownloadProgress {
            phase: "Скачивание сборки CurseForge".into(),
            ..Default::default()
        },
    );
    let tmp_dir = std::env::temp_dir().join(format!("mono-cf-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&tmp_dir)?;
    let zip_path = tmp_dir.join("modpack.zip");
    download_zip(app, client, &file.download_url, &zip_path).await?;

    let extract_dir = crate::mrpack::extract_mrpack(app, &zip_path).await?;

    let manifest: CfManifest = serde_json::from_slice(
        &std::fs::read(extract_dir.join("manifest.json"))
            .context("В архиве сборки нет manifest.json")?,
    )
    .context("Не удалось разобрать manifest.json")?;

    let files = resolve_manifest_files(client, &manifest.files).await?;
    if files.is_empty() {
        return Err(anyhow!(
            "У сборки не нашлось ни одного доступного файла (все проекты удалены?)"
        ));
    }

    let mut dependencies = std::collections::HashMap::new();
    dependencies.insert("minecraft".to_string(), manifest.minecraft.version.clone());
    for ml in manifest.mod_loaders.iter() {
        if let Some((key, ver)) = cf_loader_to_dep(&ml.id) {
            dependencies.entry(key).or_insert(ver);
            break;
        }
    }

    let version_id = file_id.to_string();
    let index = crate::mrpack::ModrinthIndex {
        format_version: 1,
        game: "minecraft".into(),
        version_id: version_id.clone(),
        name: manifest.name.clone(),
        summary: None,
        files,
        libraries: Vec::new(),
        dependencies,
    };

    let game_dir = crate::config::version_dir(pack_id, &version_id)?;
    let mut custom = crate::mrpack::download_all_files(app, client, pack_id, &index, &game_dir).await?;

    crate::mrpack::emit_progress(
        app,
        &crate::mrpack::DownloadProgress {
            phase: "Применение overrides".into(),
            ..Default::default()
        },
    );
    crate::mrpack::apply_overrides(app, &extract_dir, &game_dir)?;
    crate::mrpack::collect_override_jars(&extract_dir, &mut custom)?;

    crate::mrpack::write_install_marker(&game_dir, &index, Some(&manifest.version))?;
    std::fs::write(
        game_dir.join(".mono-index.json"),
        serde_json::to_vec_pretty(&index)?,
    )?;
    std::fs::write(
        game_dir.join(".mono-custom.json"),
        serde_json::to_vec_pretty(&custom)?,
    )?;
    crate::config::set_active_version(pack_id, &version_id)?;

    let _ = std::fs::remove_dir_all(&extract_dir);
    let _ = std::fs::remove_dir_all(&tmp_dir);

    let loader = ["fabric-loader", "forge", "neoforge", "quilt-loader"]
        .iter()
        .find(|l| index.dependencies.contains_key(**l))
        .copied()
        .unwrap_or("vanilla")
        .replace("-loader", "");
    let loader_version = index
        .dependencies
        .get(&index_loader_key(&loader))
        .cloned();

    Ok(crate::mrpack::PackInfo {
        name: manifest.name,
        summary: None,
        version_id,
        minecraft_version: manifest.minecraft.version,
        loader: loader.clone(),
        loader_version,
        file_count: index.files.len(),
    })
}

fn index_loader_key(loader: &str) -> String {
    match loader {
        "fabric" => "fabric-loader".to_string(),
        "quilt" => "quilt-loader".to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: u32, date: &str, mcs: &[&str]) -> (String, FileItem) {
        (
            date.to_string(),
            FileItem {
                id,
                display_name: format!("f{id}"),
                file_name: format!("f{id}.jar"),
                is_available: true,
                download_url: Some(format!("https://cdn.example/f{id}.jar")),
                game_versions: mcs.iter().map(|s| s.to_string()).collect(),
                file_date: date.to_string(),
                hashes: vec![],
                dependencies: vec![],
            },
        )
    }

    #[test]
    fn search_item_parses_camelcase_api() {
        // Реальный ответ API: downloadCount / latestFilesIndexes / authors (camelCase).
        let json = r#"{"data":[{"id":1546263,"gameId":432,"name":"Arizona","slug":"arizona",
          "links":{"websiteUrl":"x","wikiUrl":null,"issuesUrl":null,"sourceUrl":null},
          "summary":"a simple mod","status":4,"downloadCount":400,"isFeatured":false,
          "primaryLanguage":"en","authors":[{"id":1,"name":"BugCreator"}],
          "logo":{"thumbnailUrl":"","url":""},"screenshots":[],
          "latestFiles":[{"id":1,"displayName":"","fileName":"a.jar","fileDate":"2024-01-01T00:00:00Z","releaseType":1,"gameVersions":["1.20.1"],"modLoaders":[]}],
          "latestFilesIndexes":[{"gameVersion":"1.20.1","fileId":1}]}]}"#;
        let resp: SearchResp = serde_json::from_str(json).unwrap();
        assert_eq!(resp.data.len(), 1);
        assert_eq!(resp.data[0].id, 1546263);
        assert_eq!(resp.data[0].name, "Arizona");
        assert_eq!(resp.data[0].download_count, 400);
        assert_eq!(resp.data[0].authors[0].name, "BugCreator");
        assert_eq!(resp.data[0].latest_files_indexes[0].file_id, 1);
    }

    #[test]
    fn pick_latest_prefers_mc_version_then_newest() {
        let files = vec![
            item(1, "2024-01-01", &["1.20"]),
            item(2, "2024-06-01", &["1.20.4", "1.20.5"]),
            item(3, "2024-03-01", &["1.20.1"]),
        ];
        // Ожидаемая версия есть — берём последний из подходящих.
        let got = pick_latest(files.clone(), Some("1.20")).unwrap();
        assert_eq!(got.id, 2);
        // Версии нет — последний по дате.
        let got = pick_latest(files.clone(), Some("1.21")).unwrap();
        assert_eq!(got.id, 2);
        // Без версии — последний по дате.
        let got = pick_latest(files.clone(), None).unwrap();
        assert_eq!(got.id, 2);
    }

    #[test]
    fn file_ext_by_class() {
        assert_eq!(file_ext_for_class(CLASS_MODS), "jar");
        assert_eq!(file_ext_for_class(CLASS_RESOURCEPACKS), "zip");
        assert_eq!(file_ext_for_class(CLASS_SHADERPACKS), "zip");
    }

    #[test]
    fn maps_cf_loaders_to_dep_keys() {
        assert_eq!(
            cf_loader_to_dep("forge-40.1.0").unwrap(),
            ("forge".into(), "40.1.0".into())
        );
        assert_eq!(
            cf_loader_to_dep("fabric-0.14.21").unwrap(),
            ("fabric-loader".into(), "0.14.21".into())
        );
        assert_eq!(
            cf_loader_to_dep("neoforge-47.1.106").unwrap(),
            ("neoforge".into(), "47.1.106".into())
        );
        assert_eq!(
            cf_loader_to_dep("quilt-0.24.2").unwrap(),
            ("quilt-loader".into(), "0.24.2".into())
        );
        assert!(cf_loader_to_dep("liteloader-1.16.5").is_none());
        assert!(cf_loader_to_dep("forge-").is_none());
    }
}