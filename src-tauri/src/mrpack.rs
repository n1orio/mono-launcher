use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

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
    /// Maven-библиотеки, объявленные модами сборки (напр. lwjgl-lmdb/lwjgl-zstd
    /// с natives-linux для Voxy). Отсутствуют в ванильном version.json.
    #[serde(default)]
    pub libraries: Vec<IndexLibrary>,
    pub dependencies: HashMap<String, String>,
}

/// Библиотека из mrpack: maven-координата + базовый URL репозитория.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexLibrary {
    /// Maven-координата вида `org.lwjgl:lwjgl-lmdb:3.3.2[:classifier]`
    pub path: String,
    #[serde(default)]
    pub hashes: HashMap<String, String>,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub url: Option<String>,
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

pub(crate) fn emit_progress(app: &AppHandle, progress: &DownloadProgress) {
    let _ = app.emit("download-progress", progress);
}

/// Контекст для стримингового прогресса одного файла сборки.
#[derive(Clone)]
pub struct DlCtx {
    pub app: AppHandle,
    pub phase: String,
    pub file_index: usize,
    pub file_total: usize,
    pub current_file: String,
}

/// Эмитит прогресс текущего файла. Троттлинг (~80 мс) не даёт заваливать UI
/// частыми событиями, из-за которых плашка прогресса «лагала».
fn emit_file_progress(ctx: &DlCtx, done: u64, total: u64, last: &mut Instant, force: bool) {
    if !force && last.elapsed() < Duration::from_millis(80) {
        return;
    }
    *last = Instant::now();
    let _ = ctx.app.emit(
        "download-progress",
        &DownloadProgress {
            phase: ctx.phase.clone(),
            current: done,
            total,
            file_index: ctx.file_index,
            file_total: ctx.file_total,
            current_file: ctx.current_file.clone(),
            bytes_per_sec: 0,
        },
    );
}

fn resolve_url(file: &IndexFile) -> Option<&str> {
    file.downloads
        .first()
        .map(|s| s.as_str())
        .or(file.url.as_deref())
}

/// Файл сборки, загруженный не с доверенного CDN (Modrinth/CurseForge).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CustomFile {
    pub path: String,
    pub url: String,
}

/// Хосты, которым доверяем как источникам файлов сборки.
/// Всё остальное считается пользовательскими (кастомными) файлами.
const TRUSTED_DOWNLOAD_HOSTS: [&str; 4] = [
    "cdn.modrinth.com",
    "dl.modrinth.com",
    "mediafiles.forgecdn.net",
    "edge.forgecdn.net",
];

fn custom_file(file: &IndexFile) -> Option<CustomFile> {
    let url = resolve_url(file)?.to_string();
    let host = url.split("://").nth(1)?.split('/').next()?.to_lowercase();
    let trusted = TRUSTED_DOWNLOAD_HOSTS
        .iter()
        .any(|h| host == *h || host.ends_with(&format!(".{h}")));
    if trusted {
        None
    } else {
        Some(CustomFile {
            path: file.path.clone(),
            url,
        })
    }
}

/// Проверяет, что путь файла из индекса сборки безопасен: относительный и без
/// обхода каталогов (`..`, absolute, `.`). Предохраняет от записи за пределами
/// `game_dir` враждебной сборкой.
fn safe_rel_path(rel: &str) -> Result<&str> {
    if rel.is_empty() {
        return Err(anyhow!("Пустой путь в индексе сборки"));
    }
    let p = Path::new(rel);
    if !p.is_relative() {
        return Err(anyhow!("Недопустимый абсолютный путь в индексе: {rel}"));
    }
    for c in p.components() {
        match c {
            std::path::Component::Normal(_) | std::path::Component::CurDir => {}
            _ => return Err(anyhow!("Недопустимый путь в индексе сборки: {rel}")),
        }
    }
    Ok(rel)
}

/// Скачивает `.mrpack` по конкретному URL во временный файл (с повторами
/// при обрыве потока — CDN/GitHub иногда режут соединение на середине).
pub async fn download_mrpack(
    app: &AppHandle,
    client: &Client,
    pack_id: &str,
    url: &str,
) -> Result<PathBuf> {
    let dest_dir = config::mrpack_cache_dir(pack_id)?;
    fs::create_dir_all(&dest_dir)?;
    let path = config::mrpack_file_path(pack_id)?;

    // Локальный .mrpack (drag&drop файла): просто копируем в кэш.
    if let Some(src) = url.strip_prefix("file://") {
        tokio::fs::copy(src, &path)
            .await
            .context("Не удалось скопировать локальный .mrpack")?;
        return Ok(path);
    }

    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 0..3 {
        match download_mrpack_once(app, client, url, &path).await {
            Ok(()) => return Ok(path),
            Err(e) => {
                last_err = Some(e);
                if attempt < 2 {
                    tokio::time::sleep(std::time::Duration::from_millis(600 * (attempt + 1)))
                        .await;
                }
            }
        }
    }
    Err(last_err.unwrap())
}

async fn download_mrpack_once(
    app: &AppHandle,
    client: &Client,
    url: &str,
    path: &Path,
) -> Result<()> {
    let resp = client
        .get(url)
        .send()
        .await
        .context("Не удалось скачать .mrpack")?
        .error_for_status()
        .with_context(|| {
            format!("GitHub не отдал .mrpack (возможно, релиз удалён или переименован): {url}")
        })?;

    let total = resp.content_length().unwrap_or(0);
    let mut stream = resp.bytes_stream();
    let mut file = tokio::fs::File::create(path).await?;

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
    Ok(())
}

/// Распаковывает `.mrpack` во временную папку и возвращает её путь.
pub async fn extract_mrpack(app: &AppHandle, mrpack_path: &Path) -> Result<PathBuf> {
    let tmp_dir = std::env::temp_dir().join(format!("mono-mrpack-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&tmp_dir)?;

    let file = fs::File::open(mrpack_path)?;
    let mut archive = zip::ZipArchive::new(file).context("Не удалось открыть .mrpack как zip")?;
    let total = archive.len();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let entry_name = entry
            .enclosed_name()
            .ok_or_else(|| anyhow!("Некорректное имя файла в архиве"))?;
        let out_path = tmp_dir.join(entry_name);

        emit_progress(
            app,
            &DownloadProgress {
                phase: "Распаковка архива".into(),
                current: i as u64,
                total: total as u64,
                file_index: i,
                file_total: total,
                current_file: entry.name().to_string(),
                bytes_per_sec: 0,
            },
        );

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

pub(crate) fn verify_sha1(path: &Path, expected: &str) -> Result<bool> {
    Ok(compute_sha1(path)?.eq_ignore_ascii_case(expected))
}

fn verify_sha512(path: &Path, expected: &str) -> Result<bool> {
    Ok(compute_sha512(path)?.eq_ignore_ascii_case(expected))
}

pub(crate) fn compute_sha1(path: &Path) -> Result<String> {
    let mut hasher = Sha1::new();
    stream_update(path, |b| {
        hasher.update(b);
        Ok(())
    })?;
    Ok(format!("{:x}", hasher.finalize()))
}

fn compute_sha512(path: &Path) -> Result<String> {
    let mut hasher = Sha512::new();
    stream_update(path, |b| {
        hasher.update(b);
        Ok(())
    })?;
    Ok(format!("{:x}", hasher.finalize()))
}

/// Поблочно читает файл и отдаёт каждый блок в `f` (потоковый хэш,
/// без загрузки всего файла в память).
fn stream_update(path: &Path, mut f: impl FnMut(&[u8]) -> Result<()>) -> Result<()> {
    let file = fs::File::open(path)?;
    let mut reader = std::io::BufReader::with_capacity(256 * 1024, file);
    let mut buf = vec![0u8; 256 * 1024];
    use std::io::Read;
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        f(&buf[..n])?;
    }
    Ok(())
}

/// Читает файл один раз и считает оба хэша (sha1 + sha512).
fn compute_dual_hash(path: &Path) -> Result<(String, String)> {
    let mut s1 = Sha1::new();
    let mut s512 = Sha512::new();
    stream_update(path, |b| {
        s1.update(b);
        s512.update(b);
        Ok(())
    })?;
    Ok((
        format!("{:x}", s1.finalize()),
        format!("{:x}", s512.finalize()),
    ))
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
                if let Ok((s1, s512)) = compute_dual_hash(&p) {
                    index.entry(s1).or_insert(p.clone());
                    index.entry(s512).or_insert(p);
                }
            }
        }
    }
    index
}

// ---------- Глобальный файловый кэш ----------

/// Глобальный контент-адресный кэш `<корень>/file-cache/<sha1>`:
/// один раз скачанный мод доступен всем сборкам без повторного скачивания.
pub fn file_cache_dir() -> Result<PathBuf> {
    Ok(config::launcher_root()?.join("file-cache"))
}

fn cache_path(sha1: &str) -> Result<PathBuf> {
    Ok(file_cache_dir()?.join(sha1.to_lowercase()))
}

/// Ищет файл в глобальном кэше и проверяет его целостность.
fn cache_get(sha1: &str, hashes: &HashMap<String, String>) -> Option<PathBuf> {
    let path = cache_path(sha1).ok()?;
    (path.exists() && hashes_ok(&path, hashes)).then_some(path)
}

/// Кладёт проверенный файл в кэш (best-effort): сначала жёсткой ссылкой,
/// при неудаче — копией. Раскладка диска может отличаться — копия надёжнее.
fn cache_put(src: &Path, sha1: &str) {
    if sha1.is_empty() {
        return;
    }
    let Ok(dir) = file_cache_dir() else { return };
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let Ok(dst) = cache_path(sha1) else { return };
    if dst.exists() || fs::hard_link(src, &dst).is_ok() {
        return;
    }
    let _ = fs::copy(src, &dst);
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

/// Скачивает файл по URL с повторами при обрыве потока (до 3 попыток).
/// Хэши всё равно сверяются после скачивания, поэтому частичный файл
/// не переживёт — `File::create` перезаписывает с нуля.
async fn download_file(
    client: &Client,
    url: &str,
    dest: &Path,
    semaphore: Arc<Semaphore>,
    ctx: DlCtx,
) -> Result<u64> {
    let _permit = semaphore.acquire().await?;
    fs::create_dir_all(dest.parent().unwrap_or(Path::new(".")))?;
    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 0..3 {
        match download_file_once(client, url, dest, &ctx).await {
            Ok(len) => return Ok(len),
            Err(e) => {
                last_err = Some(e);
                if attempt < 2 {
                    tokio::time::sleep(Duration::from_millis(600 * (attempt + 1))).await;
                }
            }
        }
    }
    Err(last_err.unwrap())
}

async fn download_file_once(
    client: &Client,
    url: &str,
    dest: &Path,
    ctx: &DlCtx,
) -> Result<u64> {
    let resp = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("Не удалось скачать {url}"))?
        .error_for_status()?;

    let total = resp.content_length().unwrap_or(0);
    let mut file = tokio::fs::File::create(dest).await?;
    let mut stream = resp.bytes_stream();
    let mut done: u64 = 0;
    let mut last = Instant::now();
    emit_file_progress(ctx, 0, total, &mut last, true);
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Ошибка чтения файла")?;
        file.write_all(&chunk).await?;
        done += chunk.len() as u64;
        emit_file_progress(ctx, done, total, &mut last, false);
    }
    file.flush().await?;
    let len = fs::metadata(dest)?.len();
    emit_file_progress(ctx, len, len, &mut last, true);
    Ok(len)
}

/// Скачивает все файлы из индекса параллельно (или копирует из других версий,
/// если файл с тем же хэшем уже установлен) и возвращает количество скачанного.
/// Каждый скачанный файл сверяется с хэшами из индекса, а кастомным считается
/// файл с недоверенного источника (не Modrinth/CurseForge CDN).
pub async fn download_all_files(
    app: &AppHandle,
    client: &Client,
    pack_id: &str,
    index: &ModrinthIndex,
    game_dir: &Path,
) -> Result<Vec<CustomFile>> {
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
    let mut custom = Vec::new();

    for (i, file) in index.files.iter().enumerate() {
        let rel = safe_rel_path(&file.path)?;
        let dest = game_dir.join(rel);
        if dest.exists() && hashes_ok(&dest, &file.hashes) {
            continue;
        }

        // Если у файла нет ссылки на скачивание — он приходит из overrides.
        let url = match resolve_url(file) {
            Some(u) => u.to_string(),
            None => continue,
        };
        if let Some(cf) = custom_file(file) {
            custom.push(cf);
        }
        let dest = dest.clone();
        let client = client.clone();
        let app = app.clone();
        let semaphore = semaphore.clone();
        let path_name = rel.to_string();
        let hashes = file.hashes.clone();
        let total_files = index.files.len();
        let ctx = DlCtx {
            app: app.clone(),
            phase: "Установка модов".into(),
            file_index: i,
            file_total: total_files,
            current_file: path_name.clone(),
        };

        // Есть ли этот файл в другой установленной версии или в глобальном кэше?
        let key = file
            .hashes
            .get("sha1")
            .or_else(|| file.hashes.get("sha512"))
            .map(|h| h.to_lowercase());
        let sha1 = file.hashes.get("sha1").map(|h| h.to_lowercase());
        let cached = key
            .clone()
            .and_then(|k| reuse_index.get(&k).cloned())
            .or_else(|| key.clone().and_then(|k| cache_get(&k, &file.hashes)));

        tasks.push(tokio::spawn(async move {
            let result = if let Some(src) = cached {
                let parent = dest.parent().unwrap_or(Path::new("."));
                tokio::fs::create_dir_all(parent).await?;
                tokio::fs::copy(&src, &dest).await?;
                if let Some(s1) = &sha1 {
                    cache_put(&src, s1);
                }
                Ok(tokio::fs::metadata(&dest).await?.len())
            } else {
                download_file(&client, &url, &dest, semaphore, ctx).await
            };
            // Пост-загрузочная проверка целостности против хэшей из индекса:
            // ловит подмену файла по пути от источника до диска.
            let result = result.and_then(|size| {
                if hashes.is_empty() || hashes_ok(&dest, &hashes) {
                    if let Some(s1) = &sha1 {
                        cache_put(&dest, s1);
                    }
                    Ok(size)
                } else {
                    let _ = fs::remove_file(&dest);
                    Err(anyhow!(
                        "Файл не прошёл проверку хэша (возможно, подмена при скачивании): {path_name}"
                    ))
                }
            });
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
                        phase: format!("Ошибка: {e}"),
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

    Ok(custom)
}

/// Копирует папку `overrides` из распакованного архива в папку игры
/// с показом прогресса (фаза «Применение overrides»).
pub fn apply_overrides(app: &AppHandle, extract_dir: &Path, game_dir: &Path) -> Result<()> {
    let overrides = extract_dir.join("overrides");
    if !overrides.exists() {
        return Ok(());
    }
    let mut files: Vec<PathBuf> = Vec::new();
    collect_files(&overrides, Path::new(""), &mut files);
    let total = files.len();
    for (i, rel) in files.iter().enumerate() {
        let src = overrides.join(rel);
        let dst = game_dir.join(rel);
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&src, &dst)?;
        emit_progress(
            app,
            &DownloadProgress {
                phase: "Применение overrides".into(),
                current: i as u64,
                total: total as u64,
                file_index: i,
                file_total: total,
                current_file: rel.to_string_lossy().to_string(),
                bytes_per_sec: 0,
            },
        );
    }
    Ok(())
}

/// Собирает относительные пути всех файлов (без каталогов) в `src`.
fn collect_files(src: &Path, prefix: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(src) else { return };
    for entry in rd.flatten() {
        let path = entry.path();
        let rel = prefix.join(entry.file_name());
        if path.is_dir() {
            collect_files(&path, &rel, out);
        } else if path.is_file() {
            out.push(rel);
        }
    }
}

/// Маркер установки: файл с версией в папке игры.
const INSTALL_MARKER: &str = ".mono-installed.json";

/// Установленная версия с её маркерными данными.
#[derive(Debug, Clone, serde::Serialize)]
pub struct InstalledVersion {
    pub version_id: String,
    pub name: String,
    pub source_tag: Option<String>,
    pub total_seconds: u64,
}

const PLAYTIME_FILE: &str = ".mono-playtime.json";

/// Накопленное время игры в версии (секунды).
pub fn read_playtime(dir: &Path) -> u64 {
    let path = dir.join(PLAYTIME_FILE);
    let Ok(raw) = fs::read_to_string(&path) else {
        return 0;
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return 0;
    };
    json["totalSeconds"].as_u64().unwrap_or(0)
}

/// Записывает суммарное время игры для версии.
pub fn write_playtime(pack_id: &str, version_id: &str, total_seconds: u64) -> Result<()> {
    let dir = config::version_dir(pack_id, version_id)?;
    fs::create_dir_all(&dir)?;
    fs::write(
        dir.join(PLAYTIME_FILE),
        serde_json::json!({ "totalSeconds": total_seconds }).to_string(),
    )?;
    Ok(())
}

/// Добавляет секунды к накопленному времени и возвращает новое значение.
pub fn add_playtime(pack_id: &str, version_id: &str, seconds: u64) -> u64 {
    let dir = match config::version_dir(pack_id, version_id) {
        Ok(d) => d,
        Err(_) => return 0,
    };
    let total = read_playtime(&dir) + seconds;
    let _ = write_playtime(pack_id, version_id, total);
    total
}

/// Суммарное время игры во всех установленных версиях сборки (секунды).
pub fn pack_playtime_seconds(pack_id: &str) -> u64 {
    let Ok(root) = config::versions_root(pack_id) else {
        return 0;
    };
    let Ok(dirs) = fs::read_dir(&root) else {
        return 0;
    };
    dirs.flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| read_playtime(&e.path()))
        .sum()
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
                let total_seconds = read_playtime(&dir);
                out.push(InstalledVersion {
                    version_id,
                    name,
                    source_tag: tag,
                    total_seconds,
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
    for name in [".mcpack.json", ".mono-index.json"] {
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

pub fn write_install_marker(
    game_dir: &Path,
    index: &ModrinthIndex,
    source_tag: Option<&str>,
) -> Result<()> {
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

/// Результат проверки целостности сборки.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyResult {
    pub checked: usize,
    pub ok: usize,
    /// Что сломано: «путь — причина».
    pub broken: Vec<String>,
}

/// Проверяет файлы активной версии по хэшам из её индекса
/// (файлы без хэшей — только по наличию).
pub fn verify_pack(pack_id: &str) -> Result<VerifyResult> {
    let game_dir = config::active_game_dir(pack_id)?;
    let index_path = game_dir.join(".mono-index.json");
    let raw = fs::read_to_string(&index_path)
        .context("Сборка не установлена (нет индекса). Нажмите «Скачать и играть».")?;
    let index: ModrinthIndex = serde_json::from_str(&raw)?;

    // Клиентские файлы сборки (серверные в установку не попадают).
    let files: Vec<&IndexFile> = index
        .files
        .iter()
        .filter(|f| f.env.as_ref().map(|e| e.client.as_str()) != Some("server"))
        .collect();

    // Хэширование — CPU/IO-задача: проверяем файлы параллельно на всех ядрах.
    // Возвращаем индексы битых файлов в порядке исходного списка.
    let workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(16);
    let chunk_size = files.len().div_ceil(workers).max(1);
    let broken_indexes: Vec<usize> = std::thread::scope(|scope| {
        let game_dir_ref = &game_dir;
        let handles: Vec<_> = files
            .chunks(chunk_size)
            .enumerate()
            .map(|(ci, chunk)| {
                scope.spawn(move || {
                    let mut bad = Vec::new();
                    for (i, file) in chunk.iter().enumerate() {
                        // Небезопасный/неотносительный путь считаем битым.
                        let ok = match safe_rel_path(&file.path) {
                            Ok(rel) => {
                                let dest = game_dir_ref.join(rel);
                                dest.exists()
                                    && (file.hashes.is_empty() || hashes_ok(&dest, &file.hashes))
                            }
                            Err(_) => false,
                        };
                        if !ok {
                            // Глобальный индекс: фиксированный размер чанка,
                            // т.к. последний чанк может быть короче.
                            bad.push(ci * chunk_size + i);
                        }
                    }
                    bad
                })
            })
            .collect();
        let mut all: Vec<usize> = Vec::new();
        for h in handles {
            if let Ok(mut v) = h.join() {
                all.append(&mut v);
            }
        }
        all.sort_unstable();
        all
    });

    let broken: Vec<String> = broken_indexes
        .into_iter()
        .map(|i| {
            let f = files[i];
            let dest = game_dir.join(&f.path);
            if !dest.exists() {
                format!("{} — отсутствует", f.path)
            } else {
                format!("{} — повреждён (хэш не совпал)", f.path)
            }
        })
        .collect();
    let ok = files.len().saturating_sub(broken.len());
    Ok(VerifyResult {
        checked: files.len(),
        ok,
        broken,
    })
}

/// Полное скачивание + распаковка + установка конкретной версии.
/// Маркер со списком кастомных файлов установленной версии.
const CUSTOM_MODS_FILE: &str = ".mono-custom.json";

/// Собирает `.jar`-файлы из папки `overrides` сборки — это моды, которые
/// Prism положил в архив без записей об источнике (кастомные).
pub(crate) fn collect_override_jars(extract_dir: &Path, custom: &mut Vec<CustomFile>) -> Result<()> {
    let overrides = extract_dir.join("overrides");
    if !overrides.exists() {
        return Ok(());
    }
    let mut queue: Vec<PathBuf> = vec![overrides.clone()];
    while let Some(dir) = queue.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let p = entry.path();
            if p.is_dir() {
                queue.push(p);
            } else if p.extension().is_some_and(|e| e.eq_ignore_ascii_case("jar")) {
                let rel = p
                    .strip_prefix(&overrides)
                    .map_err(|_| anyhow!("Путь за пределами overrides"))?;
                custom.push(CustomFile {
                    path: rel.to_string_lossy().to_string(),
                    url: "overrides (внутри .mrpack)".to_string(),
                });
            }
        }
    }
    Ok(())
}

/// Список кастомных файлов установленной версии (пусто — всё с доверенных CDN).
pub fn read_custom_mods(pack_id: &str, version: &str) -> Vec<CustomFile> {
    let Ok(dir) = config::version_dir(pack_id, version) else {
        return Vec::new();
    };
    let Ok(data) = fs::read(dir.join(CUSTOM_MODS_FILE)) else {
        return Vec::new();
    };
    serde_json::from_slice(&data).unwrap_or_default()
}

/// Maven-координата `group:artifact:version[:classifier]` → относительный путь
/// в репозитории (`group/a/b/version/artifact-version[-classifier].jar`).
fn maven_coord_to_rel_path(coord: &str) -> Result<String> {
    let parts: Vec<&str> = coord.split(':').collect();
    if parts.len() < 3 {
        return Err(anyhow!("Некорректная maven-координата: {coord}"));
    }
    let (group, artifact, version) = (parts[0], parts[1], parts[2]);
    let classifier = parts.get(3).filter(|c| !c.is_empty());
    let file = match classifier {
        Some(c) => format!("{artifact}-{version}-{c}.jar"),
        None => format!("{artifact}-{version}.jar"),
    };
    Ok(format!(
        "{}/{}/{}/{}/{}",
        group.replace('.', "/"),
        artifact,
        version,
        version,
        file
    ))
}

const PACK_LIBRARIES_FILE: &str = ".mono-libraries.json";

/// Скачивает библиотеки, объявленные в mrpack (`index.libraries`), в общий
/// каталог `libraries/` и возвращает абсолютные пути для classpath.
pub async fn download_pack_libraries(
    app: &AppHandle,
    client: &Client,
    index: &ModrinthIndex,
) -> Result<Vec<PathBuf>> {
    if index.libraries.is_empty() {
        return Ok(Vec::new());
    }
    let root = config::launcher_root()?;
    let libraries_dir = root.join("libraries");
    let total = index.libraries.len();
    let mut out = Vec::with_capacity(total);
    for (i, lib) in index.libraries.iter().enumerate() {
        let rel = maven_coord_to_rel_path(&lib.path)?;
        let dest = libraries_dir.join(&rel);
        let base = lib
            .url
            .as_deref()
            .unwrap_or("https://libraries.minecraft.net/");
        let base = base.trim_end_matches('/');
        let url = format!("{base}/{rel}");
        if !dest.exists() || !hashes_ok(&dest, &lib.hashes) {
            emit_progress(
                app,
                &DownloadProgress {
                    phase: "Библиотеки модов".into(),
                    current: i as u64,
                    total: total as u64,
                    file_index: i,
                    file_total: total,
                    current_file: lib.path.clone(),
                    bytes_per_sec: 0,
                },
            );
            let ctx = DlCtx {
                app: app.clone(),
                phase: "Библиотеки модов".into(),
                file_index: i,
                file_total: total,
                current_file: lib.path.clone(),
            };
            download_file_once(client, &url, &dest, &ctx).await.with_context(|| {
                format!("Не удалось скачать библиотеку сборки {url}")
            })?;
            if !hashes_ok(&dest, &lib.hashes) {
                let _ = fs::remove_file(&dest);
                return Err(anyhow!("Хэш библиотеки {} не совпал", lib.path));
            }
        }
        out.push(dest);
    }
    Ok(out)
}

/// Пути библиотек сборки, сохранённые при установке версии.
pub fn read_pack_libraries(pack_id: &str) -> Vec<PathBuf> {
    let dir = match config::active_game_dir(pack_id) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    match fs::read(dir.join(PACK_LIBRARIES_FILE)) {
        Ok(data) => serde_json::from_slice(&data).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

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
    let extract_dir = extract_mrpack(&app, &mrpack_path).await?;
    let (index, info) = parse_index(&extract_dir)?;

    // Своя папка на каждую версию, чтобы можно было переключаться.
    let game_dir = config::version_dir(pack_id, &info.version_id)?;
    let mut custom = download_all_files(&app, client, pack_id, &index, &game_dir).await?;

    emit_progress(
        &app,
        &DownloadProgress {
            phase: "Библиотеки модов".into(),
            ..Default::default()
        },
    );
    let pack_libs = download_pack_libraries(&app, client, &index).await?;

    emit_progress(
        &app,
        &DownloadProgress {
            phase: "Применение overrides".into(),
            ..Default::default()
        },
    );
    apply_overrides(&app, &extract_dir, &game_dir)?;
    collect_override_jars(&extract_dir, &mut custom)?;

    // Маркер установки + копия индекса в папке версии.
    write_install_marker(&game_dir, &index, source_tag)?;
    fs::write(
        game_dir.join(".mono-index.json"),
        serde_json::to_vec_pretty(&index)?,
    )?;
    fs::write(
        game_dir.join(CUSTOM_MODS_FILE),
        serde_json::to_vec_pretty(&custom)?,
    )?;
    fs::write(
        game_dir.join(PACK_LIBRARIES_FILE),
        serde_json::to_vec_pretty(&pack_libs)?,
    )?;

    config::set_active_version(pack_id, &info.version_id)?;

    let _ = fs::remove_dir_all(&extract_dir);

    Ok(info)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file_with_url(url: &str) -> IndexFile {
        IndexFile {
            path: "mods/x.jar".into(),
            hashes: HashMap::new(),
            downloads: vec![url.into()],
            url: None,
            file_size: 0,
            env: None,
        }
    }

    #[test]
    fn trusts_modrinth_and_curseforge_cdns() {
        assert!(custom_file(&file_with_url("https://cdn.modrinth.com/data/abc/xyz.jar")).is_none());
        assert!(custom_file(&file_with_url(
            "https://dl.modrinth.com/mod/abc/1.0/xyz.jar"
        ))
        .is_none());
        assert!(custom_file(&file_with_url(
            "https://mediafiles.forgecdn.net/files/1234/5/mod.jar"
        ))
        .is_none());
        assert!(custom_file(&file_with_url(
            "https://edge.forgecdn.net/files/1234/5/mod.jar"
        ))
        .is_none());
    }

    #[test]
    fn flags_untrusted_sources_as_custom() {
        for url in [
            "https://github.com/user/repo/releases/download/v1/mod.jar",
            "https://legacy.curseforge.com/files/123/mod.jar",
            "https://pastebin.com/raw/abc",
            "http://example.com/mod.jar",
        ] {
            let cf = custom_file(&file_with_url(url))
                .unwrap_or_else(|| panic!("{url} должен считаться кастомным"));
            assert_eq!(cf.path, "mods/x.jar");
            assert_eq!(cf.url, url);
        }
    }

    #[test]
    fn file_without_download_url_is_not_custom() {
        let f = IndexFile {
            path: "config/x.json".into(),
            hashes: HashMap::new(),
            downloads: vec![],
            url: None,
            file_size: 0,
            env: None,
        };
        assert!(custom_file(&f).is_none());
    }
}
