//! CurseForge: поиск файлов (моды/ресурспаки/шейдеры) и установка в сборку.
//!
//! API v1 (api.curseforge.com) требует ключ: `x-api-key`. Ключ задаётся одним
//! из способов:
//! 1) файл `<данные лаунчера>/curseforge-key.txt` (одной строкой),
//! 2) переменная окружения NIO_CURSEFORGE_KEY,
//! 3) константа CURSEFORGE_KEY ниже.
//! Получить ключ: console.curseforge.com → API keys (нужен аккаунт Twitch/CurseForge).
//! Файлы скачиваются с CDN forgecdn.net — отдельный доступ не нужен.

use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::config;

const API_BASE: &str = "https://api.curseforge.com/v1";
const GAME_MINECRAFT: u32 = 432;

/// Классы проектов Minecraft на CurseForge.
pub const CLASS_MODS: u32 = 6;
pub const CLASS_RESOURCEPACKS: u32 = 12;
pub const CLASS_SHADERPACKS: u32 = 6552;

const CURSEFORGE_KEY: &str = "CHANGE_ME";

pub fn api_key_from_cfg() -> Option<String> {
    let file =
        std::fs::read_to_string(config::launcher_root().ok()?.join("curseforge-key.txt")).ok();
    let env = std::env::var("NIO_CURSEFORGE_KEY").ok();
    for candidate in [file, env].into_iter().flatten() {
        let t = candidate.trim().to_string();
        if !t.is_empty() && t != "CHANGE_ME" {
            return Some(t);
        }
    }
    if CURSEFORGE_KEY != "CHANGE_ME" {
        return Some(CURSEFORGE_KEY.to_string());
    }
    None
}

fn require_api_key() -> Result<String> {
    api_key_from_cfg().ok_or_else(|| {
        anyhow!(
            "CurseForge требует API-ключ.\n\
             Получите его на console.curseforge.com → API keys (бесплатно, нужен аккаунт CurseForge),\n\
             затем запишите одной строкой в файл:\n\
             {}",
            config::launcher_root()
                .map(|p| p.join("curseforge-key.txt").display().to_string())
                .unwrap_or_else(|_| "<данные лаунчера>/curseforge-key.txt".into())
        )
    })
}

fn ua() -> String {
    format!("nio-launcher/{}", env!("CARGO_PKG_VERSION"))
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
}

#[derive(Debug, Deserialize)]
struct SearchResp {
    data: Vec<SearchItem>,
}

#[derive(Debug, Deserialize)]
struct SearchItem {
    id: u32,
    name: String,
    #[serde(alias = "summary")]
    summary: String,
    download_count: u64,
    authors: Vec<Author>,
    #[serde(default)]
    latest_files_indexes: Vec<LatestFileIndex>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Author {
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LatestFileIndex {
    game_version: String,
    file_id: u32,
}

/// Поиск проектов по классу (моды/ресурспаки/шейдеры).
pub async fn search(
    client: &reqwest::Client,
    query: &str,
    class_id: u32,
) -> Result<Vec<CurseSearchHit>> {
    let key = require_api_key()?;
    let resp: SearchResp = client
        .get(format!("{API_BASE}/mods/search"))
        .header("x-api-key", &key)
        .header("User-Agent", ua())
        .query(&[
            ("gameId", GAME_MINECRAFT.to_string()),
            ("classId", class_id.to_string()),
            ("searchFilter", query.trim().to_string()),
            ("pageSize", "20".into()),
            ("sortField", "6".into()), // популярность
        ])
        .send()
        .await
        .context("Не удалось связаться с CurseForge")?
        .error_for_status()
        .context("CurseForge отклонил запрос (проверьте API-ключ)")?
        .json()
        .await?;
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
        })
        .collect())
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
}

#[derive(Debug, Clone, Deserialize)]
struct FileHash {
    algorithm: u32, // 1 = sha1
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

    let sha1 = f
        .hashes
        .iter()
        .find(|h| h.algorithm == 1)
        .map(|h| h.value.clone())
        .unwrap_or_default();
    Ok(CurseFile {
        file_id: f.id,
        project_id,
        file_name: f.file_name,
        download_url: f
            .download_url
            .unwrap_or_else(|| format!("https://www.curseforge.com/minecraft/mc-mods/{project_id}/download/{}/file", f.id)),
        sha1,
        game_version: f.game_versions.first().cloned().unwrap_or_default(),
    })
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
            },
        )
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
}