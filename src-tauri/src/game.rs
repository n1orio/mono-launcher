use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use sha1::Digest;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tauri::{AppHandle, Emitter};

use crate::auth::UserSession;
use crate::config;

/// GET + JSON с проверкой Content-Type и понятными ошибками.
/// На случай, когда сервер отдаёт HTML/пустую страницу вместо JSON
/// (Mojang иногда отдаёт HTML с 200) — reqwest тогда падал бы
/// с «error decoding response body».
async fn fetch_json<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
) -> Result<T> {
    let resp = client
        .get(url)
        .header("Accept", "application/json")
        .send()
        .await
        .with_context(|| format!("Не удалось получить ответ от {url}"))?
        .error_for_status()
        .with_context(|| format!("Сервер {url} вернул ошибку"))?;
    let ct = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    if !ct.to_lowercase().contains("json") {
        return Err(anyhow!(
            "Сервер {url} ответил не JSON (тип: {ct}). Возможно, CDN/Mojang временно недоступны — попробуйте позже."
        ));
    }
    resp.json::<T>()
        .await
        .with_context(|| format!("Не удалось разобрать JSON от {url}"))
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct VersionManifest {
    latest: Latest,
    versions: Vec<ManifestEntry>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Latest {
    release: String,
    snapshot: String,
}

#[derive(Debug, Deserialize)]
struct ManifestEntry {
    id: String,
    url: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VersionJson {
    id: String,
    #[serde(default)]
    arguments: Arguments,
    #[serde(default)]
    main_class: String,
    #[serde(default)]
    minecraft_arguments: String,
    #[serde(default)]
    libraries: Vec<Library>,
    #[serde(default)]
    downloads: Downloads,
    #[serde(default)]
    asset_index: Option<AssetIndex>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct Arguments {
    #[serde(default)]
    game: Vec<serde_json::Value>,
    #[serde(default)]
    jvm: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct Downloads {
    #[serde(default)]
    client: Artifact,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
struct Artifact {
    url: String,
    sha1: String,
    size: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct AssetIndex {
    id: String,
    url: String,
    sha1: String,
    size: u64,
    #[serde(default)]
    total_size: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct Library {
    name: String,
    #[serde(default)]
    downloads: LibraryDownloads,
    #[serde(default)]
    rules: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct LibraryDownloads {
    #[serde(default)]
    artifact: Artifact,
    #[serde(default)]
    classifiers: HashMap<String, Artifact>,
}

fn path_sep() -> char {
    if cfg!(windows) {
        ';'
    } else {
        ':'
    }
}

fn os_name() -> &'static str {
    #[cfg(target_os = "windows")]
    return "windows";
    #[cfg(target_os = "macos")]
    return "osx";
    #[cfg(all(unix, not(target_os = "macos")))]
    return "linux";
}

fn arch() -> &'static str {
    if std::env::consts::ARCH == "x86_64" {
        "64"
    } else if std::env::consts::ARCH == "aarch64" {
        "arm64"
    } else {
        "32"
    }
}

fn maven_path(name: &str) -> PathBuf {
    // Формат: group:artifact:version[:classifier]
    let mut parts = name.split(':').collect::<Vec<_>>();
    let classifier = if parts.len() > 3 {
        parts.pop()
    } else {
        None
    };
    let version = parts.pop().unwrap_or("");
    let artifact = parts.pop().unwrap_or("");
    let group = parts.join(".");
    let file_name = match classifier {
        Some(c) => format!("{artifact}-{version}-{c}.jar"),
        None => format!("{artifact}-{version}.jar"),
    };
    PathBuf::from(group.replace('.', "/"))
        .join(artifact)
        .join(version)
        .join(file_name)
}

fn maven_url(name: &str) -> String {
    // https://libraries.minecraft.net/ используется для большинства библиотек.
    format!("https://libraries.minecraft.net/{}", maven_path(name).display())
}

/// Проверяет правила библиотеки для текущей ОС и «фич» (features не задаём вовсе —
/// правила с ними не применяются, т.е. аргументы вроде --width/--quickPlay отсекаются).
fn rules_allow(rules: &[serde_json::Value]) -> bool {
    if rules.is_empty() {
        return true;
    }
    let mut any_applied = false;
    let mut allowed = false;
    for rule in rules {
        let os = &rule["os"];
        if let Some(feats) = rule.get("features") {
            if feats.is_object() && !feats.as_object().map(|m| m.is_empty()).unwrap_or(true) {
                continue;
            }
        }
        let name = os["name"].as_str().map(|s| s.to_string());
        let os_ok = match &name {
            Some(n) if n != os_name() => false,
            _ => true,
        };
        if !os_ok {
            continue;
        }
        let arch_ok = match os["arch"].as_str() {
            Some(a) => a == arch(),
            None => true,
        };
        if arch_ok {
            any_applied = true;
            allowed = rule["action"].as_str().unwrap_or("disallow") == "allow";
        }
    }
    allowed && any_applied
}

/// Скачивает файл в `dest`, если его там нет (проверяя sha1).
async fn ensure_download(client: &reqwest::Client, url: &str, sha1: &str, dest: &Path) -> Result<()> {
    if dest.exists() {
        return Ok(());
    }
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let resp = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("Не удалось скачать {url}"))?
        .error_for_status()?;
    let bytes = resp.bytes().await?;
    tokio::fs::write(dest, &bytes).await?;
    if !sha1.is_empty() {
        let mut hasher = sha1::Sha1::new();
        hasher.update(&bytes);
        let got = format!("{:x}", hasher.finalize());
        if !got.eq_ignore_ascii_case(sha1) {
            tokio::fs::remove_file(dest).await.ok();
            return Err(anyhow!("Хэш не совпал: {}", dest.display()));
        }
    }
    Ok(())
}

struct ResolvedLibraries {
    classpath: Vec<PathBuf>,
    natives: Vec<PathBuf>,
}

/// Скачивает и разрешает все библиотеки для версии.
async fn resolve_libraries(
    client: &reqwest::Client,
    version_json: &VersionJson,
    libraries_dir: &Path,
) -> Result<ResolvedLibraries> {
    tokio::fs::create_dir_all(libraries_dir).await?;
    let mut classpath = Vec::new();
    let mut natives = Vec::new();
    let natives_dir = libraries_dir.join("natives");

    for lib in &version_json.libraries {
        if !rules_allow(&lib.rules) {
            continue;
        }

        let artifact_url = if !lib.downloads.artifact.url.is_empty() {
            lib.downloads.artifact.url.clone()
        } else {
            maven_url(&lib.name)
        };
        let artifact_path = if !lib.downloads.artifact.url.is_empty() {
            libraries_dir.join(maven_path(&lib.name))
        } else {
            libraries_dir.join(maven_path(&lib.name))
        };
        let artifact_sha = lib.downloads.artifact.sha1.clone();
        ensure_download(client, &artifact_url, &artifact_sha, &artifact_path).await?;
        classpath.push(artifact_path.clone());

        // Нативные библиотеки. В version.json 1.21.1 natives — отдельные записи
        // (org.lwjgl:lwjgl:3.3.3:natives-linux с правилами на ОС), а не classifiers.
        // Извлекаем их .so/.dll/.dylib в natives_dir: без этого -Djava.library.path
        // пуст и LWJGL падает с "Failed to locate library: liblwjgl.so".
        let natives_classifier = format!("natives-{}", os_name());
        let name_classifier = lib.name.split(':').last().unwrap_or("");
        let is_native_entry = name_classifier == natives_classifier;

        let native_jar = if let Some(classifier) = lib.downloads.classifiers.get(&natives_classifier) {
            let jar = libraries_dir.join(maven_path(&lib.name).with_file_name(format!(
                "{}-{}.jar",
                maven_path(&lib.name)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy(),
                natives_classifier
            )));
            ensure_download(client, &classifier.url, &classifier.sha1, &jar).await?;
            Some(jar)
        } else if is_native_entry {
            Some(artifact_path.clone())
        } else {
            None
        };

        if let Some(jar) = native_jar {
            // Извлекаем natives в отдельную папку (синхронно, чтобы не держать ZipFile в async).
            {
                let file = std::fs::File::open(&jar)?;
                let mut archive = zip::ZipArchive::new(file)?;
                for i in 0..archive.len() {
                    let mut entry = archive.by_index(i)?;
                    // Пропускаем записи-каталоги (например, META-INF/): писать в них
                    // нельзя — open() падает с EISDIR («Это каталог», os error 21).
                    if entry.is_dir() {
                        continue;
                    }
                    let name = entry
                        .enclosed_name()
                        .ok_or_else(|| anyhow!("Bad entry"))?
                        .to_path_buf();
                    let target = natives_dir.join(&name);
                    if let Some(p) = target.parent() {
                        std::fs::create_dir_all(p)?;
                    }
                    let mut buf = Vec::new();
                    std::io::Read::read_to_end(&mut entry, &mut buf)?;
                    std::fs::write(&target, &buf)?;
                }
            }
            natives.push(natives_dir.clone());
        }
    }

    Ok(ResolvedLibraries {
        classpath,
        natives,
    })
}

/// Определяет модлоадер и его версию из сохранённого индекса сборки.
fn detect_loader(index: &serde_json::Value) -> Option<(String, String)> {
    let deps = index["dependencies"].as_object()?;
    for key in ["neoforge", "forge", "fabric-loader", "quilt-loader"] {
        if let Some(v) = deps.get(key) {
            if let Some(ver) = v.as_str() {
                if !ver.is_empty() {
                    return Some((key.trim_end_matches("-loader").to_string(), ver.to_string()));
                }
            }
        }
    }
    None
}

/// Получает профиль запуска модлоадера:
/// - neoforge: запускает официальный инсталлятор (создаёт srg/extra/client
///   артефакты в libraries и версионный json в versions/) и читает его профиль;
///   при неудаче — version.json из installer jar'а
/// - forge: version.json из installer jar'а
/// - fabric/quilt: профиль с их meta-API
/// Профиль наследует ванильный (inheritsFrom), поэтому библиотеки и аргументы объединяются.
async fn fetch_loader_profile(
    client: &reqwest::Client,
    loader: &str,
    mc_version: &str,
    loader_version: &str,
    cache_dir: &PathBuf,
    root: &Path,
) -> Result<Option<VersionJson>> {
    match loader {
        "neoforge" | "forge" => {
            let (base, name) = if loader == "neoforge" {
                ("https://maven.neoforged.net/releases/net/neoforged/neoforge", "neoforge")
            } else {
                ("https://maven.minecraftforge.net/net/minecraftforge/forge", "forge")
            };
            let installer_url = format!("{base}/{loader_version}/{name}-{loader_version}-installer.jar");
            let installer_path = cache_dir
                .join("loaders")
                .join(format!("{name}-{loader_version}-installer.jar"));
            ensure_download(client, &installer_url, "", &installer_path).await?;

            if loader == "neoforge" {
                run_neoforge_installer(client, mc_version, loader_version, &installer_path, root).await?;
                let launch_id = format!("{name}-{loader_version}");
                let installed = root.join("versions").join(&launch_id).join(format!("{launch_id}.json"));
                if installed.exists() {
                    let raw = tokio::fs::read(&installed).await?;
                    return Ok(Some(serde_json::from_slice(&raw)?));
                }
            }

            let file = std::fs::File::open(&installer_path)?;
            let mut archive = zip::ZipArchive::new(file)?;
            let mut entry = archive.by_name("version.json")?;
            let mut buf = Vec::new();
            std::io::Read::read_to_end(&mut entry, &mut buf)?;
            Ok(Some(serde_json::from_slice(&buf)?))
        }
        "fabric" => {
            let url = format!(
                "https://meta.fabricmc.net/v2/versions/loader/{mc_version}/{loader_version}/{loader_version}/profile/json"
            );
            fetch_json::<VersionJson>(client, &url).await.map(Some)
        }
        "quilt" => {
            let url = format!(
                "https://meta.quiltmc.org/v3/versions/loader/{mc_version}/{loader_version}/profile/json"
            );
            fetch_json::<VersionJson>(client, &url).await.map(Some)
        }
        _ => Ok(None),
    }
}

/// Запускает официальный NeoForge-инсталлятор (--installClient), который создаёт
/// в libraries/ клиентские артефакты (net/minecraft/client/<mc>-<neoform>/client-…-srg.jar,
/// -extra.jar, net/neoforged/neoforge/<ver>/neoforge-<ver>-client.jar, -universal.jar),
/// а в versions/neoforge-<ver>/ — профиль запуска. Эти файлы НЕ лежат в classpath:
/// их находит сам FML через -DlibraryDirectory. Запускаем один раз — проверяем по
/// наличию установленного профиля.
async fn run_neoforge_installer(
    client: &reqwest::Client,
    mc_version: &str,
    loader_version: &str,
    installer_path: &Path,
    root: &Path,
) -> Result<()> {
    let launch_id = format!("neoforge-{loader_version}");
    let version_dir = root.join("versions");
    let json_path = version_dir.join(&launch_id).join(format!("{launch_id}.json"));
    if json_path.exists() {
        return Ok(());
    }

    // Инсталлятору нужен ванильный профиль (versions/<mc>/<mc>.json + .jar)
    // и launcher_profiles.json.
    let mc_dir = version_dir.join(mc_version);
    tokio::fs::create_dir_all(&mc_dir).await?;
    let mc_json = mc_dir.join(format!("{mc_version}.json"));
    let mc_jar = mc_dir.join(format!("{mc_version}.jar"));
    if !mc_json.exists() {
        let manifest: VersionManifest = fetch_json(
            &client,
            "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json",
        )
        .await?;
        let entry = manifest
            .versions
            .iter()
            .find(|v| v.id == mc_version)
            .ok_or_else(|| anyhow!("Версия {} не найдена в манифесте", mc_version))?;
        let resp = client.get(&entry.url).send().await?.error_for_status()?;
        let bytes = resp.bytes().await?;
        tokio::fs::write(&mc_json, &bytes).await?;
    }
    if !mc_jar.exists() {
        let raw = tokio::fs::read(&mc_json).await?;
        let vanilla: VersionJson = serde_json::from_slice(&raw)?;
        ensure_download(client, &vanilla.downloads.client.url, &vanilla.downloads.client.sha1, &mc_jar).await?;
    }
    let profiles = root.join("launcher_profiles.json");
    if !profiles.exists() {
        tokio::fs::write(&profiles, "{\"profiles\":{}}").await?;
    }

    let java = find_java()?;
    let log_file = config::launch_log_file().ok();
    if let Some(path) = &log_file {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
    }
    append_log(&log_file, "=== NeoForge installer: starting ===");
    let output = tokio::process::Command::new(&java)
        .arg("-jar")
        .arg(installer_path)
        .arg("--installClient")
        .arg(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Не удалось запустить NeoForge-инсталлятор")?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    for line in text.lines() {
        if !line.is_empty() {
            append_log(&log_file, &format!("[installer] {line}"));
        }
    }
    if !output.status.success() {
        let tail: Vec<&str> = text.lines().rev().take(30).collect();
        let mut tail = tail.into_iter().collect::<Vec<_>>();
        tail.reverse();
        return Err(anyhow!(
            "NeoForge-инсталлятор завершился с ошибкой: {}\n{}",
            output.status,
            tail.join("\n")
        ));
    }
    if !json_path.exists() {
        return Err(anyhow!("Инсталлятор не создал профиль {launch_id}"));
    }
    append_log(&log_file, "=== NeoForge installer: done ===");
    Ok(())
}

/// Скачивает «игровой» jar модера (forge universal, который содержит патченый клиент).
async fn resolve_loader_client_jar(
    client: &reqwest::Client,
    loader: &str,
    loader_version: &str,
    libraries_dir: &Path,
) -> Result<Option<PathBuf>> {
    match loader {
        "neoforge" => {
            let path = libraries_dir.join(format!("net/neoforged/neoforge/{loader_version}/neoforge-{loader_version}.jar"));
            let url = format!(
                "https://maven.neoforged.net/releases/net/neoforged/neoforge/{loader_version}/neoforge-{loader_version}-universal.jar"
            );
            ensure_download(client, &url, "", &path).await?;
            Ok(Some(path))
        }
        "forge" => {
            let path = libraries_dir.join(format!("net/minecraftforge/forge/{loader_version}/forge-{loader_version}.jar"));
            let url = format!("https://maven.minecraftforge.net/net/minecraftforge/forge/{loader_version}/forge-{loader_version}.jar");
            let mut saved = ensure_download(client, &url, "", &path).await.is_ok();
            if !path.exists() {
                let url2 = format!("https://maven.minecraftforge.net/net/minecraftforge/forge/{loader_version}/forge-{loader_version}-universal.jar");
                saved = ensure_download(client, &url2, "", &path).await.is_ok();
            }
            if saved && path.exists() {
                Ok(Some(path))
            } else {
                Ok(None)
            }
        }
        _ => Ok(None),
    }
}

/// Объединяет библиотеки ванильного и лодер-профиля.
/// Ключ — полное имя (включая классификатор): в ванильном 1.21.1 каждый
/// вариант natives-<os> идёт отдельной записью, и дедупликация по
/// group:artifact выкинула бы ядро lwjgl (классификатор natives-windows-x86
/// перекрывал бы org.lwjgl:lwjgl:3.3.3), из-за чего sodium падал бы с
/// NoClassDefFoundError: org/lwjgl/Version.
fn merge_libraries(vanilla: Vec<Library>, loader: Vec<Library>) -> Vec<Library> {
    let mut map: HashMap<String, Library> = HashMap::new();
    for lib in vanilla {
        map.insert(lib.name.clone(), lib);
    }
    for lib in loader {
        map.insert(lib.name.clone(), lib);
    }
    map.into_values().collect()
}

async fn resolve_assets(
    client: &reqwest::Client,
    version_json: &VersionJson,
    assets_root: &Path,
) -> Result<String> {
    let asset_index = version_json
        .asset_index
        .as_ref()
        .ok_or_else(|| anyhow!("Нет assetIndex в профиле"))?;
    let indexes_dir = assets_root.join("indexes");
    let objects_dir = assets_root.join("objects");
    let index_path = indexes_dir.join(format!("{}.json", asset_index.id));
    ensure_download(
        client,
        &asset_index.url,
        &asset_index.sha1,
        &index_path,
    )
    .await?;

    let raw = tokio::fs::read_to_string(&index_path).await?;
    let index: serde_json::Value = serde_json::from_str(&raw)?;
    let objects = index["objects"].as_object().cloned().unwrap_or_default();

    // Скачиваем объекты (ассеты) — параллельно, с лимитом.
    let semaphore = Arc::new(tokio::sync::Semaphore::new(16));
    let mut tasks = Vec::new();
    for (_name, obj) in objects {
        let hash = obj["hash"].as_str().unwrap_or("").to_string();
        if hash.is_empty() {
            continue;
        }
        let prefix = &hash[..2];
        let dest = objects_dir.join(prefix).join(&hash);
        if dest.exists() {
            continue;
        }
        let url = format!(
            "https://resources.download.minecraft.net/{}/{}",
            prefix, hash
        );
        let client = client.clone();
        let sem = semaphore.clone();
        tasks.push(tokio::spawn(async move {
            let _p = sem.acquire().await?;
            ensure_download(&client, &url, &hash, &dest).await
        }));
    }

    let results = futures::future::join_all(tasks).await;
    for r in results {
        r??;
    }

    Ok(asset_index.id.clone())
}

async fn resolve_client_jar(
    client: &reqwest::Client,
    version_json: &VersionJson,
    versions_dir: &Path,
) -> Result<PathBuf> {
    let jar_path = versions_dir
        .join(&version_json.id)
        .join(format!("{}.jar", version_json.id));
    ensure_download(
        client,
        &version_json.downloads.client.url,
        &version_json.downloads.client.sha1,
        &jar_path,
    )
    .await?;
    Ok(jar_path)
}

fn split_args(args: &[serde_json::Value]) -> Vec<String> {
    let mut out = Vec::new();
    for arg in args {
        match arg {
            serde_json::Value::String(s) => out.push(s.clone()),
            serde_json::Value::Object(obj) => {
                // conditionallyAllowed / rules
                if rules_allow(&obj["rules"].as_array().cloned().unwrap_or_default()) {
                    if let Some(v) = obj["value"].as_str() {
                        out.push(v.to_string());
                    } else if let Some(v) = obj["value"].as_array() {
                        for item in v {
                            if let Some(s) = item.as_str() {
                                out.push(s.to_string());
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    out
}

/// Собирает и запускает процесс Java для Minecraft.
pub async fn launch_game(
    pack_id: &str,
    ram_gb: u32,
    session: UserSession,
    app: AppHandle,
    width: u32,
    height: u32,
) -> Result<()> {
    let root = config::launcher_root()?;
    let assets_root = root.join("assets");
    let libraries_dir = root.join("libraries");
    let versions_dir = root.join("versions-libs");

    // Проверяем Java ДО скачиваний: без неё нечего качать сотни мегабайт.
    let java = find_java()?;
    let java_ok = tokio::process::Command::new(&java)
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false);
    if !java_ok {
        return Err(anyhow!(
            "Java не найдена. Установите Java 17+ (например OpenJDK 21) или положите \
             Java в папку {}",
            config::java_root()?.display()
        ));
    }

    // 1. Определяем версию Minecraft и модлоадер из активной установленной версии.
    let game_dir = config::active_game_dir(pack_id)?;
    let index_path = game_dir.join(".nio-index.json");
    if !index_path.exists() {
        return Err(anyhow!("Сборка не установлена. Нажмите «Скачать и играть»."));
    }
    let raw = tokio::fs::read_to_string(&index_path).await?;
    let index: serde_json::Value = serde_json::from_str(&raw)?;
    let minecraft_version = index["dependencies"]["minecraft"]
        .as_str()
        .ok_or_else(|| anyhow!("Не найдена версия Minecraft"))?;
    let loader = detect_loader(&index);

    // 2. Получаем manifest и ванильный version json.
    let client = reqwest::Client::new();
    emit_log(&app, "sys", "Получение манифеста версий Minecraft…");
    let manifest: VersionManifest = fetch_json(
        &client,
        "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json",
    )
    .await?;
    let entry = manifest
        .versions
        .iter()
        .find(|v| v.id == minecraft_version)
        .ok_or_else(|| anyhow!("Версия {} не найдена в манифесте", minecraft_version))?;

    let vanilla: VersionJson = fetch_json(&client, &entry.url).await?;

    // 3. Профиль модлоадера (если есть). Он наследует ванильный профиль,
    //    поэтому библиотеки и аргументы объединяются.
    let loader_profile = match &loader {
        Some((name, ver)) => {
            fetch_loader_profile(&client, name, minecraft_version, ver, &libraries_dir, &root).await?
        }
        None => None,
    };

    // Идентификатор «версии» для запуска: neoforge-21.1.248 или vanilla id.
    let launch_id = match &loader {
        Some((name, ver)) => format!("{name}-{ver}"),
        None => vanilla.id.clone(),
    };

    // 4. Объединяем библиотеки, аргументы и main class.
    let merged_libraries = match &loader_profile {
        Some(lp) => merge_libraries(vanilla.libraries.clone(), lp.libraries.clone()),
        None => vanilla.libraries.clone(),
    };
    let jvm_args = match &loader_profile {
        Some(lp) => {
            let mut v = split_args(&vanilla.arguments.jvm);
            v.extend(split_args(&lp.arguments.jvm));
            v
        }
        None => split_args(&vanilla.arguments.jvm),
    };
    let mut game_args = split_args(&vanilla.arguments.game);
    if let Some(lp) = &loader_profile {
        game_args.extend(split_args(&lp.arguments.game));
    }
    // NeoForge/Forge (FML через modlauncher) требуют явный launch target;
    // обычно он есть в профиле, но подстрахуемся (иначе ImmediateWindowHandler NPE).
    if matches!(loader.as_ref(), Some((name, _)) if name == "neoforge" || name == "forge")
        && !game_args.iter().any(|a| a == "--launchTarget")
    {
        game_args.insert(0, "--launchTarget".into());
        game_args.insert(1, "forgeclient".into());
    }
    // Early Display FML требует размеры окна (иначе NoSuchElementException
    // в DisplayWindow.updateModuleReads). Prism всегда передаёт width/height.
    if !game_args.iter().any(|a| a == "--width") {
        game_args.extend([
            "--width".into(),
            width.to_string(),
            "--height".into(),
            height.to_string(),
        ]);
    }
    let main_class = if let Some(lp) = &loader_profile {
        if !lp.main_class.is_empty() {
            lp.main_class.clone()
        } else if !vanilla.main_class.is_empty() {
            vanilla.main_class.clone()
        } else {
            "net.minecraft.client.main.Main".into()
        }
    } else if !vanilla.main_class.is_empty() {
        vanilla.main_class.clone()
    } else {
        "net.minecraft.client.main.Main".into()
    };

    // Также обрабатываем старый формат minecraftArguments (легаси-версии).
    if game_args.is_empty() && !vanilla.minecraft_arguments.is_empty() {
        game_args = vanilla
            .minecraft_arguments
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
    }

    // 5. Скачиваем библиотеки, ассеты и клиентский jar.
    emit_log(&app, "sys", "Скачивание библиотек и ассетов (первый запуск — долго)…");
    let libs = resolve_libraries(&client, &VersionJson { libraries: merged_libraries, ..vanilla.clone() }, &libraries_dir).await?;
    let asset_index_id = resolve_assets(&client, &vanilla, &assets_root).await?;
    let client_jar = if matches!(loader.as_ref(), Some((name, _)) if name == "neoforge") {
        // NeoForge: в classpath кладём «версионный» jar — копию ванильного клиента
        // в versions/neoforge-<ver>/neoforge-<ver>.jar. Именно его исключает
        // -DignoreList=…,${version_name}.jar (PR neoforged/NeoForge#1718). Патченые
        // srg/extra/client jar'ы в classpath НЕ попадают — их находит сам FML.
        let vanilla_jar = resolve_client_jar(&client, &vanilla, &versions_dir).await?;
        let jar_dir = root.join("versions").join(&launch_id);
        tokio::fs::create_dir_all(&jar_dir).await?;
        let jar = jar_dir.join(format!("{launch_id}.jar"));
        if !jar.exists() {
            tokio::fs::copy(&vanilla_jar, &jar).await?;
        }
        jar
    } else if let Some((name, ver)) = &loader {
        // forge использует свой патченый клиент; остальные — ванильный.
        match resolve_loader_client_jar(&client, name, ver, &libraries_dir).await? {
            Some(jar) => jar,
            None => resolve_client_jar(&client, &vanilla, &versions_dir).await?,
        }
    } else {
        resolve_client_jar(&client, &vanilla, &versions_dir).await?
    };

    // 6. Собираем classpath.
    let mut classpath = Vec::new();
    classpath.push(client_jar);
    classpath.extend(libs.classpath);
    let classpath_str = classpath
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(&path_sep().to_string());

    let natives_str = libs
        .natives
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(&path_sep().to_string());

    // 7. Плейсхолдеры.
    let mut placeholders: HashMap<String, String> = HashMap::new();
    placeholders.insert("${auth_player_name}".into(), session.username.clone());
    placeholders.insert("${auth_session}".into(), session.access_token.clone());
    placeholders.insert("${auth_uuid}".into(), session.uuid.clone());
    placeholders.insert("${auth_access_token}".into(), session.access_token.clone());
    placeholders.insert("${auth_xuid}".into(), String::new());
    placeholders.insert("${clientid}".into(), String::new());
    placeholders.insert("${user_properties}".into(), "{}".into());
    placeholders.insert("${user_type}".into(), session.user_type.clone());
    placeholders.insert("${version_name}".into(), launch_id.clone());
    placeholders.insert("${version_type}".into(), "release".into());
    placeholders.insert("${assets_root}".into(), assets_root.to_string_lossy().to_string());
    placeholders.insert("${assets_index_name}".into(), asset_index_id.clone());
    placeholders.insert("${game_directory}".into(), game_dir.to_string_lossy().to_string());
    placeholders.insert("${natives_directory}".into(), natives_str.clone());
    placeholders.insert("${classpath}".into(), classpath_str.clone());
    placeholders.insert(
        "${classpath_separator}".into(),
        path_sep().to_string(),
    );
    placeholders.insert(
        "${library_directory}".into(),
        libraries_dir.to_string_lossy().to_string(),
    );

    // 8. Собираем финальные аргументы процесса.
    let mut final_args = Vec::new();
    final_args.push(java);
    final_args.push(format!("-Xmx{}G", ram_gb));
    final_args.push(format!("-Xms{}G", (ram_gb / 2).max(1)));

    for a in jvm_args {
        // `-XstartOnFirstThread` — macOS-специфичный флаг; на Linux/Windows он падает.
        if a.starts_with("-XstartOnFirstThread") && !cfg!(target_os = "macos") {
            continue;
        }
        final_args.push(replace_placeholders(&a, &placeholders));
    }
    final_args.push("-cp".into());
    final_args.push(classpath_str.clone());
    final_args.push(main_class);

    for a in game_args {
        final_args.push(replace_placeholders(&a, &placeholders));
    }

    // 9. Запускаем с перехватом вывода (stdout/stderr -> событие "launch-log" + файл).
    let mut cmd = Command::new(&final_args[0]);
    cmd.args(&final_args[1..])
        .current_dir(&game_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(debug_assertions)]
    println!("Запуск: {}", final_args.join(" "));

    let mut child = cmd
        .spawn()
        .context("Не удалось запустить Java. Убедитесь, что установлен Java 17+")?;
    let pid = child.id();

    // Лог пишем в файл (перезаписывая старый) и шлём фронтенду.
    let log_file = config::launch_log_file().ok();
    if let Some(path) = &log_file {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
    }

        if let Some(out) = child.stdout.take() {
        let app2 = app.clone();
        let log2 = log_file.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(out).lines();
            loop {
                let line = match reader.next_line().await {
                    Ok(Some(line)) => line,
                    _ => break,
                };
                if !line.is_empty() {
                    append_log(&log2, &line);
                    let _ = app2.emit("launch-log", LogLine { stream: "out".into(), line });
                }
            }
        });
    }
    if let Some(err) = child.stderr.take() {
        let app2 = app.clone();
        let log2 = log_file.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(err).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if !line.is_empty() {
                    append_log(&log2, &line);
                    let _ = app2.emit("launch-log", LogLine { stream: "err".into(), line });
                }
            }
        });
    }

    // Учёт времени игры в экземпляре: пишем в .nio-playtime.json каждые 30 секунд
    // и финально при завершении процесса.
    let version_id = game_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let pack_id_owned = pack_id.to_string();
    let app2 = app.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        let mut last = std::time::Instant::now();
        let exit = loop {
            tokio::select! {
                _ = interval.tick() => {
                    let delta = last.elapsed().as_secs();
                    last = std::time::Instant::now();
                    if !version_id.is_empty() && delta > 0 {
                        crate::mrpack::add_playtime(&pack_id_owned, &version_id, delta);
                    }
                }
                status = child.wait() => break status.unwrap_or_default(),
            }
        };
        let delta = last.elapsed().as_secs();
        if !version_id.is_empty() && delta >= 1 {
            let total = crate::mrpack::add_playtime(&pack_id_owned, &version_id, delta);
            let _ = app2.emit(
                "playtime-updated",
                PlaytimeUpdate {
                    version_id: version_id.clone(),
                    total_seconds: total,
                },
            );
        }
        let success = exit.success();
        let _ = app2.emit(
            "game-exited",
            GameExited {
                success,
                code: exit.code().unwrap_or(i32::MIN),
            },
        );
        let msg = if success {
            "Процесс Minecraft завершился (код 0)".to_string()
        } else {
            format!("Процесс Minecraft завершился с ошибкой: {exit}")
        };
        append_log(&log_file, &format!("\n=== {msg} ==="));
        let _ = app2.emit(
            "launch-log",
            LogLine { stream: "sys".into(), line: msg },
        );
    });

    emit_log(&app, "sys", &format!("Process started, PID {pid:?}"));

    Ok(())
}

#[derive(serde::Serialize, Clone)]
pub struct LogLine {
    pub stream: String,
    pub line: String,
}

#[derive(serde::Serialize, Clone)]
pub struct PlaytimeUpdate {
    pub version_id: String,
    pub total_seconds: u64,
}

#[derive(serde::Serialize, Clone)]
pub struct GameExited {
    pub success: bool,
    pub code: i32,
}

fn emit_log(app: &AppHandle, stream: &str, line: &str) {
    let _ = app.emit("launch-log", LogLine { stream: stream.to_string(), line: line.to_string() });
}

fn append_log(file: &Option<PathBuf>, line: &str) {
    use std::io::Write;
    if let Some(path) = file {
        let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) else {
            return;
        };
        let _ = writeln!(f, "{line}");
    }
}

fn replace_placeholders(arg: &str, map: &HashMap<String, String>) -> String {
    let mut out = arg.to_string();
    for (k, v) in map {
        out = out.replace(k, v);
    }
    out
}

/// Ищет Java: сначала встроенную в лаунчер, затем из PATH.
fn find_java() -> Result<String> {
    let bundled = config::java_root()?.join(
        #[cfg(target_os = "windows")]
        "bin/java.exe",
        #[cfg(not(target_os = "windows"))]
        "bin/java",
    );
    if bundled.exists() {
        return Ok(bundled.to_string_lossy().to_string());
    }
    Ok("java".into())
}
