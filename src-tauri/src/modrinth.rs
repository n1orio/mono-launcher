use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::config;

/// Базовый URL публичного API Modrinth (без ключа, rate limit ~300 req/мин).
const API: &str = "https://api.modrinth.com/v2";

/// Проект Modrinth (мод или модпак) — карточка из поиска.
/// API отдаёт snake_case; на фронтенд сериализуем camelCase.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct ModrinthProject {
    /// В поиске — `project_id`, в полном объекте проекта — `id`.
    #[serde(default, alias = "id")]
    pub project_id: String,
    pub slug: String,
    pub project_type: String,
    pub title: String,
    pub description: String,
    /// Есть только в ответе поиска; у полного /project/{id} — отсутствует.
    #[serde(default)]
    pub author: String,
    pub icon_url: Option<String>,
    pub downloads: u64,
    pub categories: Vec<String>,
    pub latest_version: Option<String>,
    /// Полное описание (markdown) — только в полном объекте проекта.
    #[serde(default)]
    pub body: Option<String>,
    /// Галерея скриншотов — есть и в поиске (строки-URL), и в полном объекте.
    #[serde(default, deserialize_with = "de_gallery")]
    pub gallery: Vec<ModrinthGalleryItem>,
}

/// Скриншот из галереи проекта.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct ModrinthGalleryItem {
    pub url: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub featured: Option<bool>,
}

/// Галерея скриншотов: в ответе поиска — массив URL-строк, у полного
/// проекта — массив объектов; приводим к единому виду.
fn de_gallery<'de, D>(de: D) -> Result<Vec<ModrinthGalleryItem>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Entry {
        Url(String),
        Item(ModrinthGalleryItem),
    }
    let items = Vec::<Entry>::deserialize(de)?;
    Ok(items
        .into_iter()
        .map(|e| match e {
            Entry::Url(url) => ModrinthGalleryItem {
                url,
                ..Default::default()
            },
            Entry::Item(item) => item,
        })
        .collect())
}

/// Ответ поиска: hits + общее число.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchResponse {
    pub hits: Vec<ModrinthProject>,
    /// Общее число совпадений (часть формата API; фронтенду не отдаём).
    #[allow(dead_code)]
    pub total_hits: u64,
}

/// Файл версии (например .jar мода или .mrpack модпака).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct ModrinthFile {
    pub hashes: HashMap<String, String>,
    pub url: String,
    pub filename: String,
    pub primary: Option<bool>,
    pub size: u64,
}

/// Версия проекта Modrinth.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct ModrinthVersion {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub version_number: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub date_published: String,
    pub changelog: Option<String>,
    pub files: Vec<ModrinthFile>,
    pub dependencies: Vec<ModrinthDependency>,
}

/// Зависимость версии (modrinth id или обязательный другой проект).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct ModrinthDependency {
    pub project_id: Option<String>,
    pub version_id: Option<String>,
    pub dependency_type: String,
}

fn ua() -> &'static str {
    "nio-launcher/0.2.2 (desktop launcher)"
}

/// Фильтры поиска Modrinth: категории+загрузчики, версии игры,
/// окружение ("client"/"server"), сортировка (relevance/downloads/follows/newest/updated).
///
/// Все поля с `#[serde(default)]`: IPC приходит через JSON.stringify, который
/// выбрасывает undefined-ключи — `filters:{}` должен валидно десериализоваться.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchFilters {
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub versions: Vec<String>,
    #[serde(default)]
    pub environment: Option<String>,
    #[serde(default)]
    pub index: Option<String>,
}

/// Поиск проектов Modrinth.
/// `kind` — "mod" | "modpack"; query может быть пустым (топ по загрузкам).
pub async fn search_projects(
    client: &reqwest::Client,
    query: &str,
    kind: &str,
    limit: u32,
    filters: &SearchFilters,
) -> Result<Vec<ModrinthProject>> {
    let mut facets: Vec<Vec<String>> = vec![vec![format!("project_type:{kind}")]];
    if !filters.categories.is_empty() {
        facets.push(
            filters
                .categories
                .iter()
                .map(|c| format!("categories:{c}"))
                .collect(),
        );
    }
    if !filters.versions.is_empty() {
        facets.push(
            filters
                .versions
                .iter()
                .map(|v| format!("versions:{v}"))
                .collect(),
        );
    }
    if let Some(env) = &filters.environment {
        match env.as_str() {
            "client" => facets.push(vec![
                "client_side:required".into(),
                "client_side:optional".into(),
            ]),
            "server" => facets.push(vec![
                "server_side:required".into(),
                "server_side:optional".into(),
            ]),
            _ => {}
        }
    }
    let mut url = format!(
        "{API}/search?query={}&limit={}&facets={}",
        urlencode(query),
        limit,
        urlencode(&serde_json::to_string(&facets).unwrap_or_default()),
    );
    if let Some(index) = &filters.index {
        if !index.is_empty() && index != "relevance" {
            url.push_str(&format!("&index={}", urlencode(index)));
        }
    }
    let resp: SearchResponse = client
        .get(&url)
        .header("User-Agent", ua())
        .send()
        .await
        .context("Не удалось связаться с Modrinth API")?
        .error_for_status()
        .context("Modrinth API вернул ошибку")?
        .json()
        .await
        .context("Не удалось прочитать ответ Modrinth")?;
    Ok(resp.hits)
}

/// Доступные загрузчики, версии игры и категории (для фильтров поиска).
/// Возвращаются только теги для указанных типов проектов (mod, modpack,
/// resourcepack, shaderpack, datapack).
pub async fn tags(client: &reqwest::Client, kinds: &[&str]) -> Result<ModrinthTags> {
    #[derive(Deserialize)]
    struct LoaderRaw {
        name: String,
        supported_project_types: Vec<String>,
    }
    #[derive(Deserialize)]
    struct CategoryRaw {
        name: String,
        project_type: String,
    }
    #[derive(Deserialize)]
    struct VersionRaw {
        version: String,
        version_type: String,
    }
    let loaders: Vec<LoaderRaw> = client
        .get(format!("{API}/tag/loader"))
        .header("User-Agent", ua())
        .send()
        .await
        .context("Не удалось связаться с Modrinth API")?
        .error_for_status()
        .context("Modrinth API вернул ошибку")?
        .json()
        .await
        .context("Не удалось прочитать теги Modrinth")?;
    let categories: Vec<CategoryRaw> = client
        .get(format!("{API}/tag/category"))
        .header("User-Agent", ua())
        .send()
        .await
        .context("Не удалось связаться с Modrinth API")?
        .error_for_status()
        .context("Modrinth API вернул ошибку")?
        .json()
        .await
        .context("Не удалось прочитать категории Modrinth")?;
    let versions: Vec<VersionRaw> = client
        .get(format!("{API}/tag/game_version"))
        .header("User-Agent", ua())
        .send()
        .await
        .context("Не удалось связаться с Modrinth API")?
        .error_for_status()
        .context("Modrinth API вернул ошибку")?
        .json()
        .await
        .context("Не удалось прочитать версии Modrinth")?;
    let relevant = |p: &str| kinds.contains(&p);
    Ok(ModrinthTags {
        loaders: loaders
            .into_iter()
            .filter(|t| t.supported_project_types.iter().any(|p| relevant(p)))
            .map(|t| t.name)
            .collect(),
        categories: categories
            .into_iter()
            .filter(|t| relevant(&t.project_type))
            .map(|t| t.name)
            .collect(),
        // Только релизы; API уже отдаёт по убыванию даты.
        versions: versions
            .into_iter()
            .filter(|t| t.version_type == "release")
            .map(|t| t.version)
            .collect(),
    })
}

/// Все теги для фильтров: версии отсортированы от новых к старым.
#[derive(Debug, Clone, Serialize)]
pub struct ModrinthTags {
    pub loaders: Vec<String>,
    pub categories: Vec<String>,
    pub versions: Vec<String>,
}

/// Версии проекта, опционально отфильтрованные по версии игры и загрузчику.
pub async fn project_versions(
    client: &reqwest::Client,
    project_id: &str,
    game_version: Option<&str>,
    loader: Option<&str>,
) -> Result<Vec<ModrinthVersion>> {
    let mut url = format!("{API}/project/{project_id}/version?featured=true");
    if let Some(gv) = game_version {
        url.push_str(&format!(
            "&game_versions={}",
            urlencode(&format!("[\"{gv}\"]"))
        ));
    }
    if let Some(l) = loader {
        url.push_str(&format!("&loaders={}", urlencode(&format!("[\"{l}\"]"))));
    }
    let versions: Vec<ModrinthVersion> = client
        .get(&url)
        .header("User-Agent", ua())
        .send()
        .await
        .context("Не удалось связаться с Modrinth API")?
        .error_for_status()
        .context("Modrinth API вернул ошибку")?
        .json()
        .await
        .context("Не удалось прочитать версии Modrinth")?;
    Ok(versions)
}

/// Проверка обновлений по sha1: POST /version_files/update.
/// Возвращает «проект id -> новая версия», когда есть более свежая.
pub async fn check_updates(
    client: &reqwest::Client,
    hashes: &HashMap<String, String>,
    game_versions: &[String],
    loaders: &[String],
) -> Result<HashMap<String, ModrinthVersion>> {
    if hashes.is_empty() {
        return Ok(HashMap::new());
    }
    let body = serde_json::json!({
        "hashes": hashes,
        "algorithm": "sha1",
        "game_versions": game_versions,
        "loaders": loaders,
    });
    let resp: HashMap<String, ModrinthVersion> = client
        .post(format!("{API}/version_files/update"))
        .header("User-Agent", ua())
        .json(&body)
        .send()
        .await
        .context("Не удалось связаться с Modrinth API")?
        .error_for_status()
        .context("Modrinth API вернул ошибку")?
        .json()
        .await
        .context("Не удалось прочитать ответ Modrinth")?;
    Ok(resp)
}

/// Проект по id (для названия модпака и т.п.).
pub async fn project_by_id(client: &reqwest::Client, project_id: &str) -> Result<ModrinthProject> {
    let project: ModrinthProject = client
        .get(format!("{API}/project/{project_id}"))
        .header("User-Agent", ua())
        .send()
        .await
        .context("Не удалось связаться с Modrinth API")?
        .error_for_status()
        .context("Modrinth API вернул ошибку")?
        .json()
        .await
        .context("Не удалось прочитать проект Modrinth")?;
    Ok(project)
}

/// Версия по id.
pub async fn version_by_id(client: &reqwest::Client, version_id: &str) -> Result<ModrinthVersion> {
    let version: ModrinthVersion = client
        .get(format!("{API}/version/{version_id}"))
        .header("User-Agent", ua())
        .send()
        .await
        .context("Не удалось связаться с Modrinth API")?
        .error_for_status()
        .context("Modrinth API вернул ошибку")?
        .json()
        .await
        .context("Не удалось прочитать версию Modrinth")?;
    Ok(version)
}

/// Скачивает файл версии Modrinth в целевую папку с проверкой sha1.
/// Возвращает (имя файла, sha1).
pub async fn download_file(
    client: &reqwest::Client,
    file: &ModrinthFile,
    dest_dir: &Path,
) -> Result<(String, String)> {
    let file_name = safe_file_name(&file.filename);
    let dest = dest_dir.join(&file_name);
    let resp = client
        .get(&file.url)
        .header("User-Agent", ua())
        .send()
        .await
        .context("Не удалось скачать файл с Modrinth")?
        .error_for_status()
        .context("Modrinth CDN вернул ошибку")?;
    let bytes = resp.bytes().await.context("Не удалось прочитать файл")?;
    let actual_sha1 = sha1_bytes(&bytes);
    // Проверка хэша: sha1 почти всегда есть; sha512 — если sha1 отсутствует.
    if let Some(expected) = file.hashes.get("sha1") {
        if actual_sha1.to_lowercase() != expected.to_lowercase() {
            return Err(anyhow!(
                "Хэш файла {file_name} не совпал с Modrinth (ожидался {expected}, получен {actual_sha1}). Файл не установлен."
            ));
        }
    }
    std::fs::create_dir_all(dest_dir)?;
    std::fs::write(&dest, bytes)
        .with_context(|| format!("Не удалось записать {file_name}"))?;
    Ok((file_name, actual_sha1))
}

/// Заменяет содержимое локального файла (обновление мода): скачивает новое и
/// перезаписывает, сохраняя расширение исходного имени.
pub async fn update_file_to(
    client: &reqwest::Client,
    file: &ModrinthFile,
    existing_path: &Path,
) -> Result<()> {
    let tmp = existing_path.with_extension("nio-update");
    let resp = client
        .get(&file.url)
        .header("User-Agent", ua())
        .send()
        .await
        .context("Не удалось скачать обновление с Modrinth")?
        .error_for_status()
        .context("Modrinth CDN вернул ошибку")?;
    let bytes = resp.bytes().await.context("Не удалось прочитать файл")?;
    if let Some(expected) = file.hashes.get("sha1") {
        let actual = sha1_bytes(&bytes);
        if actual.to_lowercase() != expected.to_lowercase() {
            let _ = std::fs::remove_file(&tmp);
            return Err(anyhow!(
                "Хэш обновления не совпал с Modrinth (ожидался {expected}, получен {actual})."
            ));
        }
    }
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, existing_path)?;
    Ok(())
}

/// Нормализует имя файла: убирает path-инъекции, но сохраняет расширение.
fn safe_file_name(raw: &str) -> String {
    let name = raw.rsplit('/').next().unwrap_or(raw);
    let cleaned: String = name
        .chars()
        .filter(|c| !matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
        .collect();
    if cleaned.is_empty() {
        "mod.jar".into()
    } else {
        cleaned
    }
}
fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Скачивает иконку проекта в файл (например `packs/<id>/icon.png`).
pub async fn download_icon(client: &reqwest::Client, icon_url: &str, dest: &Path) -> Result<()> {
    let bytes = client
        .get(icon_url)
        .header("User-Agent", ua())
        .send()
        .await
        .context("Не удалось скачать иконку")?
        .error_for_status()
        .context("Иконка недоступна")?
        .bytes()
        .await?;
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(dest, &bytes)?;
    Ok(())
}

/// sha1 байтов (hex, нижний регистр).
fn sha1_bytes(bytes: &[u8]) -> String {
    use sha1::Digest;
    let mut hasher = sha1::Sha1::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Метаданные установленного из Modrinth файла (трекинг обновлений).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackedMod {
    /// Имя файла (с расширением).
    pub file_name: String,
    /// Папка игры: mods / resourcepacks / shaderpacks / datapacks.
    #[serde(default = "default_track_folder")]
    pub folder: String,
    /// Для датапаков — мир, куда установлен файл (saves/<world>/datapacks).
    #[serde(default)]
    pub world: Option<String>,
    /// id версии Modrinth, из которой установлен файл.
    pub version_id: String,
    /// id проекта Modrinth (для страницы и обновлений).
    pub project_id: String,
    /// sha1 установленного файла.
    pub sha1: String,
    /// Версия игры/лоадер на момент установки (для поиска обновлений).
    pub game_version: String,
    pub loader: String,
}

fn default_track_folder() -> String {
    "mods".to_string()
}

/// Файл трекинга установленных модов версии.
fn track_file(pack_id: &str) -> Result<PathBuf> {
    Ok(config::active_game_dir(pack_id)?.join(".nio-modrinth.json"))
}

/// Текущий список отслеживаемых модов активной версии.
pub fn tracked_mods(pack_id: &str) -> Vec<TrackedMod> {
    let Ok(path) = track_file(pack_id) else {
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

/// Сохраняет список отслеживаемых модов.
pub fn save_tracked_mods(pack_id: &str, mods: &[TrackedMod]) -> Result<()> {
    let path = track_file(pack_id)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, serde_json::to_string_pretty(mods)?)?;
    Ok(())
}

/// Добавляет/заменяет запись о моде (по имени файла) и сохраняет.
pub fn upsert_tracked_mod(pack_id: &str, m: &TrackedMod) -> Result<()> {
    let mut mods = tracked_mods(pack_id);
    mods.retain(|t| t.file_name != m.file_name);
    mods.push(m.clone());
    save_tracked_mods(pack_id, &mods)
}

/// Убирает запись о моде по имени файла (при удалении файла).
pub fn remove_tracked_mod(pack_id: &str, file_name: &str) -> Result<()> {
    let mut mods = tracked_mods(pack_id);
    let len_before = mods.len();
    mods.retain(|t| t.file_name != file_name);
    if mods.len() != len_before {
        save_tracked_mods(pack_id, &mods)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_query_strings() {
        assert_eq!(urlencode("sodium extra"), "sodium%20extra");
        assert_eq!(urlencode("fabric-1.21"), "fabric-1.21");
        assert_eq!(urlencode("я"), "%D1%8F");
    }

    #[test]
    fn sanitizes_filenames() {
        assert_eq!(safe_file_name("sodium-0.5.8.jar"), "sodium-0.5.8.jar");
        assert!(!safe_file_name("evil.jar").contains('/'));
        assert_eq!(safe_file_name(""), "mod.jar");
    }

    #[test]
    fn search_filters_deserializes_empty_obj() {
        // IPC (JSON.stringify) выбрасывает undefined-ключи — приходит `{}`.
        let f: SearchFilters = serde_json::from_str("{}").unwrap();
        assert!(f.categories.is_empty());
        assert!(f.versions.is_empty());
        assert!(f.environment.is_none());
        assert!(f.index.is_none());
        // Частичные фильтры тоже валидны.
        let f: SearchFilters = serde_json::from_str(r#"{"environment":"client"}"#).unwrap();
        assert_eq!(f.environment.as_deref(), Some("client"));
        assert!(f.categories.is_empty());
    }

    #[test]
    fn sha1_hex_helpers() {
        let hex = sha1_bytes(b"hello");
        assert_eq!(hex.len(), 40);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn tracked_mod_defaults_to_mods_folder() {
        // Старые записи без folder/world читаются как mods/.
        let t: TrackedMod =
            serde_json::from_str(r#"{"fileName":"x.jar","versionId":"v1","projectId":"p1","sha1":"s","gameVersion":"1.21","loader":"fabric"}"#)
                .unwrap();
        assert_eq!(t.folder, "mods");
        assert!(t.world.is_none());
        // Новые записи сохраняют папку и мир.
        let json = serde_json::to_string(&TrackedMod {
            file_name: "d.zip".into(),
            folder: "datapacks".into(),
            world: Some("My World".into()),
            version_id: "v2".into(),
            project_id: "p2".into(),
            sha1: "s".into(),
            game_version: "1.21".into(),
            loader: "vanilla".into(),
        })
        .unwrap();
        let t: TrackedMod = serde_json::from_str(&json).unwrap();
        assert_eq!(t.folder, "datapacks");
        assert_eq!(t.world.as_deref(), Some("My World"));
    }

    #[tokio::test]
    #[ignore = "требует сеть"]
    async fn live_search_works() {
        let client = reqwest::Client::new();
        let empty = SearchFilters::default();
        let mods = search_projects(&client, "sodium", "mod", 3, &empty).await.unwrap();
        assert!(!mods.is_empty());
        assert!(mods.iter().any(|p| p.slug == "sodium"));
        let packs = search_projects(&client, "better", "modpack", 3, &empty)
            .await
            .unwrap();
        assert!(!packs.is_empty());
        // Фильтры: fabric + 1.21.4 + сортировка по загрузкам.
        let filtered = search_projects(
            &client,
            "",
            "mod",
            3,
            &SearchFilters {
                categories: vec!["fabric".into()],
                versions: vec!["1.21.4".into()],
                environment: Some("client".into()),
                index: Some("downloads".into()),
            },
        )
        .await
        .unwrap();
        assert!(!filtered.is_empty());
        assert!(filtered
            .iter()
            .all(|p| p.categories.iter().any(|c| c == "fabric")));
        let tags = tags(&client, &["mod"]).await.unwrap();
        assert!(tags.loaders.contains(&"fabric".to_string()));
        assert!(tags.versions.contains(&"1.21.4".to_string()));
        assert!(!tags.categories.is_empty());
        let versions = project_versions(&client, &mods[0].project_id, None, None)
            .await
            .unwrap();
        assert!(!versions.is_empty());
        let by_id = version_by_id(&client, &versions[0].id).await.unwrap();
        assert_eq!(by_id.id, versions[0].id);
        let project = project_by_id(&client, &versions[0].project_id).await.unwrap();
        assert_eq!(project.project_id, versions[0].project_id);
    }
}
