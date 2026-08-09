use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use futures::future::try_join_all;
use futures::StreamExt;
use reqwest::Client;
use sha1::{Digest, Sha1};
use sha2::Sha512;
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;

use crate::config;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModrinthIndex {
    pub format_version: u32,
    pub game: String,
    pub version_id: String,
    pub name: String,
    pub summary: Option<String>,
    pub files: Vec<IndexFile>,
    pub dependencies: HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexFile {
    pub path: String,
    pub hashes: HashMap<String, String>,
    #[serde(default)]
    pub downloads: Vec<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub file_size: u64,
    pub env: Option<EnvRequirement>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvRequirement {
    #[serde(default)]
    pub client: String,
    #[serde(default)]
    pub server: String,
}

/// Версия Minecraft и модлоадер, извлечённые из индекса.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PackInfo {
    pub name: String,
    pub summary: Option<String>,
    pub version_id: String,
    pub minecraft_version: String,
    pub loader: String,
    pub loader_version: Option<String>,
    pub file_count: usize,
}

/// Прогресс, отправляемый на фронтенд.
#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct DownloadProgress {
    pub phase: String,
    pub current: u64,
    pub total: u64,
    pub file_index: usize,
    pub file_total: usize,
    pub current_file: String,
    pub bytes_per_sec: u64,
}

fn emit_progress(app: &AppHandle, progress: &DownloadProgress) {
    let _ = app.emit("download-progress", progress);
}

fn resolve_url(file: &IndexFile) -> Option<&str> {
    file.downloads
        .first()
        .map(|s| s.as_str())
        .or(file.url.as_deref())
}

/// Скачивает `.mrpack` по конкретному URL во временный файл.
pub async fn download_mrpack(app: &AppHandle, client: &Client, pack_id: &str, url: &str) -> Result<PathBuf> {
    let dest_dir = config::mrpack_cache_dir(pack_id)?;
    fs::create_dir_all(&dest_dir)?;

    let resp = client
        .get(url)
        .send()
        .await
        .context("Не удалось скачать .mrpack")?
        .error_for_status()
        .with_context(|| {
            format!(
                "GitHub не отдал .mrpack (возможно, релиз удалён или переименован): {url}"
            )
        })?;

    let total = resp.content_length().unwrap_or(0);
    let mut stream = resp.bytes_stream();
    let path = config::mrpack_file_path(pack_id)?;
    let mut file = tokio::fs::File::create(&path).await?;

    let mut downloaded: u64 = 0;
    let mut last_report = std::time::Instant::now();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Ошибка чтения потока скачивания")?;
        downloaded += chunk.len() as u64;
        file.write_all(&chunk).await?;
        if last_report.elapsed().as_millis() >= 150 {
            emit_progress(
                app,
                &DownloadProgress {
                    phase: "Скачивание сборки".into(),
                    current: downloaded,
                    total,
                    file_index: 0,
                    file_total: 1,
                    current_file: "modpack.mrpack".into(),
                    bytes_per_sec: 0,
                },
            );
            last_report = std::time::Instant::now();
        }
    }
    file.flush().await?;
    Ok(path)
}

/// Распаковывает `.mrpack` во временную папку и возвращает её путь.
pub async fn extract_mrpack(mrpack_path: &Path) -> Result<PathBuf> {
    let tmp_dir = std::env::temp_dir().join(format!("nio-mrpack-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&tmp_dir)?;

    let file = fs::File::open(mrpack_path)?;
    let mut archive = zip::ZipArchive::new(file).context("Не удалось открыть .mrpack как zip")?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let entry_name = entry
            .enclosed_name()
            .ok_or_else(|| anyhow!("Некорректное имя файла в архиве"))?;
        let out_path = tmp_dir.join(entry_name);

        if entry.is_dir() {
            fs::create_dir_all(&out_path)?;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut out = fs::File::create(&out_path)?;
        std::io::copy(&mut entry, &mut out)?;
    }

    Ok(tmp_dir)
}

/// Читает `modrinth.index.json` из распакованного архива.
pub fn parse_index(extract_dir: &Path) -> Result<(ModrinthIndex, PackInfo)> {
    let index_path = extract_dir.join("modrinth.index.json");
    let raw = fs::read_to_string(&index_path).context("Не найден modrinth.index.json")?;
    let index: ModrinthIndex = serde_json::from_str(&raw)?;

    let minecraft_version = index
        .dependencies
        .get("minecraft")
        .cloned()
        .ok_or_else(|| anyhow!("В индексе нет зависимости minecraft"))?;

    let loader = ["fabric-loader", "forge", "neoforge", "quilt"]
        .iter()
        .find(|l| index.dependencies.contains_key(**l))
        .copied()
        .unwrap_or("vanilla");

    let loader_version = index.dependencies.get(loader).cloned();

    let info = PackInfo {
        name: index.name.clone(),
        summary: index.summary.clone(),
        version_id: index.version_id.clone(),
        minecraft_version,
        loader: loader.replace("-loader", ""),
        loader_version,
        file_count: index.files.len(),
    };

    Ok((index, info))
}

fn verify_sha1(path: &Path, expected: &str) -> Result<bool> {
    Ok(compute_sha1(path)?.eq_ignore_ascii_case(expected))
}

fn verify_sha512(path: &Path, expected: &str) -> Result<bool> {
    Ok(compute_sha512(path)?.eq_ignore_ascii_case(expected))
}

fn compute_sha1(path: &Path) -> Result<String> {
    let data = fs::read(path)?;
    let mut hasher = Sha1::new();
    hasher.update(&data);
    Ok(format!("{:x}", hasher.finalize()))
}

fn compute_sha512(path: &Path) -> Result<String> {
    let data = fs::read(path)?;
    let mut hasher = Sha512::new();
    hasher.update(&data);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Индекс содержимого других установленных версий (sha1/sha512 -> путь),
/// чтобы при обновлении не скачивать заново неизменившиеся файлы.
fn build_installed_hash_index(pack_id: &str, exclude_version: &str) -> HashMap<String, PathBuf> {
    let mut index = HashMap::new();
    let Ok(root) = config::versions_root(pack_id) else {
        return index;
    };
    let Ok(entries) = fs::read_dir(root) else {
        return index;
    };
    let mut queue = Vec::new();
    for entry in entries.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        if entry.file_name().to_string_lossy() == exclude_version {
            continue;
        }
        if !dir.join(INSTALL_MARKER).exists() {
            continue;
        }
        queue.push(dir);
    }
    while let Some(dir) = queue.pop() {
        let Ok(rd) = fs::read_dir(&dir) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                queue.push(p);
            } else if p.is_file() {
                // Читаем один раз и считаем оба хэша.
                let Ok(data) = fs::read(&p) else { continue };
                let mut s1 = Sha1::new();
                s1.update(&data);
                let mut s512 = Sha512::new();
                s512.update(&data);
                index
                    .entry(format!("{:x}", s1.finalize()))
                    .or_insert(p.clone());
                index.entry(format!("{:x}", s512.finalize())).or_insert(p);
            }
        }
    }
    index
}

fn hashes_ok(path: &Path, hashes: &HashMap<String, String>) -> bool {
    for (algo, expected) in hashes {
        let ok = match algo.as_str() {
            "sha1" => verify_sha1(path, expected).unwrap_or(false),
            "sha512" => verify_sha512(path, expected).unwrap_or(false),
            _ => true,
        };
        if !ok {
            return false;
        }
    }
    true
}

async fn download_file(
    client: &Client,
    url: &str,
    dest: &Path,
    semaphore: Arc<Semaphore>,
) -> Result<u64> {
    let _permit = semaphore.acquire().await?;
    fs::create_dir_all(dest.parent().unwrap_or(Path::new(".")))?;

    let resp = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("Не удалось скачать {url}"))?
        .error_for_status()?;

    let mut file = tokio::fs::File::create(dest).await?;
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        file.write_all(&chunk.context("Ошибка чтения файла")?)
            .await?;
    }
    file.flush().await?;
    Ok(fs::metadata(dest)?.len())
}

/// Скачивает все файлы из индекса параллельно (или копирует из других версий,
/// если файл с тем же хэшем уже установлен) и возвращает количество скачанного.
pub async fn download_all_files(
    app: &AppHandle,
    client: &Client,
    pack_id: &str,
    index: &ModrinthIndex,
    game_dir: &Path,
) -> Result<()> {
    fs::create_dir_all(game_dir)?;

    // Индекс уже установленных файлов в других версиях — переиспользуем их.
    let exclude = game_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let reuse_index = build_installed_hash_index(pack_id, &exclude);

    // Кэш: пропускаем файлы, которые уже есть и совпадают по хэшу.
    let mut tasks = Vec::new();
    let semaphore = Arc::new(Semaphore::new(8));

    for (i, file) in index.files.iter().enumerate() {
        let dest = game_dir.join(&file.path);
        if dest.exists() && hashes_ok(&dest, &file.hashes) {
            continue;
        }

        // Если у файла нет ссылки на скачивание — он приходит из overrides.
        let url = match resolve_url(file) {
            Some(u) => u.to_string(),
            None => continue,
        };
        let dest = dest.clone();
        let client = client.clone();
        let app = app.clone();
        let semaphore = semaphore.clone();
        let path_name = file.path.clone();
        let total_files = index.files.len();

        // Есть ли этот файл в другой установленной версии?
        let key = file
            .hashes
            .get("sha1")
            .or_else(|| file.hashes.get("sha512"))
            .map(|h| h.to_lowercase());
        let cached = key.and_then(|k| reuse_index.get(&k).cloned());

        tasks.push(tokio::spawn(async move {
            let result = if let Some(src) = cached {
                let parent = dest.parent().unwrap_or(Path::new("."));
                tokio::fs::create_dir_all(parent).await?;
                tokio::fs::copy(&src, &dest).await?;
                Ok(tokio::fs::metadata(&dest).await?.len())
            } else {
                download_file(&client, &url, &dest, semaphore).await
            };
            match &result {
                Ok(size) => emit_progress(
                    &app,
                    &DownloadProgress {
                        phase: "Установка модов".into(),
                        current: 0,
                        total: *size,
                        file_index: i,
                        file_total: total_files,
                        current_file: path_name,
                        bytes_per_sec: 0,
                    },
                ),
                Err(e) => emit_progress(
                    &app,
                    &DownloadProgress {
                        phase: format!("Ошибка: {e}").into(),
                        ..Default::default()
                    },
                ),
            }
            result
        }));
    }

    let results = try_join_all(tasks).await?;
    for res in results {
        res?;
    }

    Ok(())
}

/// Копирует папку `overrides` из распакованного архива в папку игры.
pub fn apply_overrides(extract_dir: &Path, game_dir: &Path) -> Result<()> {
    let overrides = extract_dir.join("overrides");
    if !overrides.exists() {
        return Ok(());
    }
    copy_dir_recursive(&overrides, game_dir)?;
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let target = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_recursive(&path, &target)?;
        } else {
            fs::create_dir_all(target.parent().unwrap())?;
            fs::copy(&path, &target)?;
        }
    }
    Ok(())
}

/// Маркер установки: файл с версией в папке игры.
const INSTALL_MARKER: &str = ".nio-installed.json";

/// Установленная версия с её маркерными данными.
#[derive(Debug, Clone, serde::Serialize)]
pub struct InstalledVersion {
    pub version_id: String,
    pub name: String,
    pub source_tag: Option<String>,
}

/// Список установленных версий (папки с маркером) для конкретной сборки.
pub fn installed_versions(pack_id: &str) -> Vec<String> {
    installed_details(pack_id)
        .into_iter()
        .map(|v| v.version_id)
        .collect()
}

/// Детали установленных версий.
pub fn installed_details(pack_id: &str) -> Vec<InstalledVersion> {
    let mut out = Vec::new();
    if let Ok(root) = config::versions_root(pack_id) {
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let dir = entry.path();
                let marker = dir.join(INSTALL_MARKER);
                if !marker.exists() {
                    continue;
                }
                let version_id = entry.file_name().to_string_lossy().to_string();
                let mut name = version_id.clone();
                let mut tag: Option<String> = None;
                if let Ok(raw) = fs::read_to_string(&marker) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) {
                        name = json["name"].as_str().unwrap_or(&name).to_string();
                        tag = json["sourceTag"]
                            .as_str()
                            .map(|s| s.to_string())
                            .filter(|s| !s.is_empty());
                    }
                }
                out.push(InstalledVersion {
                    version_id,
                    name,
                    source_tag: tag,
                });
            }
        }
    }
    out.sort_by(|a, b| a.version_id.cmp(&b.version_id));
    out
}

/// Возвращает индекс установленной версии из её папки (если есть).
pub fn read_version_index(pack_id: &str, version_id: &str) -> Option<ModrinthIndex> {
    let dir = config::version_dir(pack_id, version_id).ok()?;
    for name in [".mcpack.json", ".nio-index.json"] {
        let path = dir.join(name);
        if path.exists() {
            if let Ok(raw) = fs::read_to_string(&path) {
                if let Ok(idx) = serde_json::from_str(&raw) {
                    return Some(idx);
                }
            }
        }
    }
    None
}

/// Определяет, установлена ли сборка (наличие маркера + скачанных матриц).
pub fn is_installed(game_dir: &Path, index: &ModrinthIndex) -> bool {
    let marker = game_dir.join(INSTALL_MARKER);
    if !marker.exists() {
        return false;
    }
    for file in index.files.iter() {
        let dest = game_dir.join(&file.path);
        if !dest.exists() {
            return false;
        }
    }
    true
}

pub fn write_install_marker(game_dir: &Path, index: &ModrinthIndex, source_tag: Option<&str>) -> Result<()> {
    let marker = game_dir.join(INSTALL_MARKER);
    let payload = serde_json::json!({
        "versionId": index.version_id,
        "name": index.name,
        "sourceTag": source_tag.unwrap_or(""),
        "installedAt": chrono::Utc::now().to_rfc3339(),
    });
    fs::write(marker, serde_json::to_vec_pretty(&payload)?)?;
    Ok(())
}

/// Полное скачивание + распаковка + установка конкретной версии.
/// Устанавливается в отдельную папку, которая затем становится активной.
pub async fn install_mrpack(
    app: AppHandle,
    client: &Client,
    pack_id: &str,
    url: &str,
    source_tag: Option<&str>,
) -> Result<PackInfo> {
    emit_progress(
        &app,
        &DownloadProgress {
            phase: "Скачивание сборки".into(),
            ..Default::default()
        },
    );

    let mrpack_path = download_mrpack(&app, client, pack_id, url).await?;

    emit_progress(
        &app,
        &DownloadProgress {
            phase: "Распаковка архива".into(),
            ..Default::default()
        },
    );
    let extract_dir = extract_mrpack(&mrpack_path).await?;
    let (index, info) = parse_index(&extract_dir)?;

    // Своя папка на каждую версию, чтобы можно было переключаться.
    let game_dir = config::version_dir(pack_id, &info.version_id)?;
    download_all_files(&app, client, pack_id, &index, &game_dir).await?;

    emit_progress(
        &app,
        &DownloadProgress {
            phase: "Применение overrides".into(),
            ..Default::default()
        },
    );
    apply_overrides(&extract_dir, &game_dir)?;

    // Маркер установки + копия индекса в папке версии.
    write_install_marker(&game_dir, &index, source_tag)?;
    fs::write(game_dir.join(".nio-index.json"), serde_json::to_vec_pretty(&index)?)?;

    config::set_active_version(pack_id, &info.version_id)?;

    let _ = fs::remove_dir_all(&extract_dir);

    Ok(info)
}
