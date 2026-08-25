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

#[derive(Debug, Serialize)]
pub struct UpdateInfo {
    pub current_version: Option<String>,
    pub latest_version: Option<String>,
    pub has_update: bool,
}

/// Всё, что нужно фронтенду для выбора версии.
#[derive(Debug, Serialize)]
pub struct VersionsInfo {
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
fn resolve_pack(pack_id: Option<String>) -> Result<PackInfo, String> {
    let id = pack_id.unwrap_or_else(default_pack_id);
    config::find_pack(&id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Сборка не найдена: {id}"))
}

/// Список сборок: встроенные + добавленные пользователем.
#[tauri::command]
fn list_packs() -> Result<Vec<PackDescriptor>, String> {
    config::all_packs()
        .map(|packs| {
            packs
                .into_iter()
                .map(|p| PackDescriptor {
                    id: p.id,
                    name: p.name,
                    url: p.url,
                    builtin: p.builtin,
                    kind: p.kind,
                    author: None,
                    boosty_blog: p.boosty_blog,
                    min_ram_mb: p.min_ram_mb,
                    icon: p.icon,
                    banner: p.banner,
                })
                .collect()
        })
        .map_err(|e| e.to_string())
}

/// Добавляет сборку по прямой ссылке на `.mrpack`.
/// `blog` (из deep link) — ник блога на Boosty.
async fn add_pack_impl(
    _client: &reqwest::Client,
    url: &str,
    name: Option<&str>,
    blog: Option<&str>,
) -> Result<PackDescriptor, String> {
    let url = url.trim().to_string();
    if url.is_empty() {
        return Err("URL не может быть пустым.".into());
    }
    if !url.to_ascii_lowercase().ends_with(".mrpack") {
        return Err("URL должен быть прямой ссылкой на .mrpack".into());
    }
    for existing in config::all_packs().map_err(|e| e.to_string())? {
        if existing.url == url {
            return Err(format!("Сборка «{}» уже добавлена", existing.name));
        }
    }
    let file_stem = url
        .rsplit('/')
        .next()
        .unwrap_or("pack")
        .trim_end_matches(".mrpack")
        .to_string();
    let pack_id = if file_stem.is_empty() {
        "pack".to_string()
    } else {
        file_stem
    };
    let pack_name = name
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .map(String::from)
        .unwrap_or_else(|| pack_id.clone());
    let blog = blog.map(str::trim).filter(|b| !b.is_empty()).map(String::from);
    config::add_user_pack(&pack_id, &pack_name, &url, "remote", blog.as_deref(), None)
        .map_err(|e| e.to_string())?;
    Ok(PackDescriptor {
        id: pack_id,
        name: pack_name,
        url,
        builtin: false,
        kind: "remote".into(),
        author: None,
        boosty_blog: blog,
        min_ram_mb: None,
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

/// Добавляет сборку из локального .mrpack (drag&drop файла в окно).
/// Файл копируется в кэш при первой установке через file://-схему.
#[tauri::command]
async fn add_pack_file_command(
    state: State<'_, AppState>,
    path: String,
    name: Option<String>,
) -> Result<PackDescriptor, String> {
    use std::path::PathBuf;
    let _guard = add_pack_lock().lock().await;
    let p = PathBuf::from(path.trim());
    if !p.is_file() {
        return Err("Файл не найден".into());
    }
    if p.extension().and_then(|e| e.to_str()) != Some("mrpack") {
        return Err("Нужен файл .mrpack".into());
    }
    let abs = p.canonicalize().map_err(|e| e.to_string())?;
    #[cfg(windows)]
    let url = format!(
        "file:///{}",
        abs.display().to_string().replace('\\', "/")
    );
    #[cfg(not(windows))]
    let url = format!("file://{}", abs.display());
    add_pack_impl(&state.client, &url, name.as_deref(), None).await
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

/// Меняет URL сборки (для установки конкретной версии с сервера Mono).
#[tauri::command]
fn set_pack_url_command(pack_id: String, url: String) -> Result<(), String> {
    config::set_pack_url(&pack_id, &url).map_err(|e| e.to_string())
}

/// Id сборок в порядке убывания времени последнего запуска.
#[tauri::command]
fn recent_packs_command() -> Vec<String> {
    config::recent_pack_ids()
}

/// Скачивает иконку сборки в `packs/<id>/icon.png` (если её ещё нет):
/// — сборки с Modrinth (`mrn-<id>`): иконка проекта;
/// — сборки с CurseForge (`cf-<id>`): логотип проекта.
/// Возвращает, нашлась ли иконка.
#[tauri::command]
async fn fetch_pack_icon_command(
    state: State<'_, AppState>,
    pack_id: String,
) -> Result<bool, String> {
    let _pack = config::find_pack(&pack_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Сборка не найдена".to_string())?;
    let dest = config::pack_dir(&pack_id)
        .map_err(|e| e.to_string())?
        .join("icon.png");
    if dest.exists() {
        return Ok(true);
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
        libraries: Vec::new(),
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
    // Check if already added by URL
    for existing in config::all_packs().map_err(|e| e.to_string())? {
        if existing.url == pack_url {
            return Ok((
                PackDescriptor {
                    id: existing.id,
                    name: existing.name,
                    url: existing.url,
                    builtin: existing.builtin,
                    kind: existing.kind,
                    author: None,
                    boosty_blog: existing.boosty_blog.clone(),
                    min_ram_mb: existing.min_ram_mb,
                    icon: existing.icon,
                    banner: existing.banner,
                },
                true,
            ));
        }
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
    _state: State<'_, AppState>,
    pack_id: Option<String>,
) -> Result<VersionsInfo, String> {
    let pack = resolve_pack(pack_id)?;
    let installed = mrpack::installed_details(&pack.id);
    let active = config::active_version(&pack.id)
        .ok()
        .filter(|v| !v.is_empty());
    Ok(VersionsInfo {
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

/// Лента новостей: релизы лаунчера (GitHub) + новости бэкенда Mono, свежие сверху.
/// Локализованный ченджлог релиза лежит в его ассете `latest.json`
/// (`notes_localized[locale]`); фолбэк — тело релиза.
#[tauri::command]
async fn get_news_command(
    _app: AppHandle,
    state: State<'_, AppState>,
    locale: String,
) -> Result<Vec<NewsItem>, String> {
    let client = &state.client;
    let mut items: Vec<NewsItem> = Vec::new();

    // 1) Релизы лаунчера с GitHub.
    const GH_RELEASES: &str =
        "https://api.github.com/repos/n1orio/mono-launcher/releases?per_page=5";
    if let Ok(resp) = client
        .get(GH_RELEASES)
        .header("User-Agent", "MonoLauncher")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
    {
        if let Ok(releases) = resp.json::<Vec<serde_json::Value>>().await {
            for r in releases {
                let tag = r["tag_name"].as_str().unwrap_or_default().to_string();
                if tag.is_empty() {
                    continue;
                }
                let mut body = r["body"].as_str().unwrap_or_default().to_string();
                if let Some(assets) = r["assets"].as_array() {
                    let manifest_url = assets
                        .iter()
                        .find(|a| a["name"].as_str() == Some("latest.json"))
                        .and_then(|a| a["browser_download_url"].as_str());
                    if let Some(mu) = manifest_url {
                        if let Ok(m) = client
                            .get(mu)
                            .header("User-Agent", "MonoLauncher")
                            .send()
                            .await
                        {
                            if let Ok(man) = m.json::<serde_json::Value>().await {
                                if let Some(loc) = man["notes_localized"][locale.as_str()].as_str()
                                {
                                    body = loc.to_string();
                                } else if let Some(n) = man["notes"].as_str() {
                                    body = n.to_string();
                                }
                            }
                        }
                    }
                }
                let version = tag.trim_start_matches("launcher-v");
                let title = r["name"]
                    .as_str()
                    .map(str::to_string)
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| format!("MonoLauncher {version}"));
                items.push(NewsItem {
                    kind: "update".into(),
                    pack_id: "launcher".into(),
                    pack_name: "Mono Launcher".into(),
                    title,
                    body,
                    url: r["html_url"].as_str().unwrap_or_default().to_string(),
                    tag: Some(tag),
                    category: None,
                    date: r["published_at"].as_str().map(str::to_string),
                });
            }
        }
    }

    // 2) Новости бэкенда Mono (общие + по сборкам); имя сборки — из каталога.
    let catalog = auth::mono_pack_catalog(client).await.unwrap_or_default();
    if let Ok(news) = auth::mono_pack_news(client).await {
        for n in news {
            let pack_id = n.pack_id.clone().unwrap_or_default();
            let pack_name = catalog
                .iter()
                .find(|p| p.id == pack_id)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "Mono".into());
            items.push(NewsItem {
                kind: if n.kind == "update" { "update" } else { "post" }.into(),
                pack_id,
                pack_name,
                title: n.title,
                body: n.body,
                url: String::new(),
                tag: None,
                category: None,
                date: Some(n.created_at),
            });
        }
    }

    // Свежие сверху; записи без даты — в конец.
    items.sort_by(|a, b| b.date.cmp(&a.date));
    Ok(items)
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

/// Проверяет наличие новой версии `.mrpack`.
#[tauri::command]
async fn check_for_updates(
    _state: State<'_, AppState>,
    pack_id: Option<String>,
) -> Result<UpdateInfo, String> {
    let pack = resolve_pack(pack_id)?;
    let installed = mrpack::installed_details(&pack.id);
    let active = config::active_version(&pack.id)
        .ok()
        .filter(|v| !v.is_empty());
    let current = installed.iter()
        .find(|v| Some(&v.version_id) == active.as_ref())
        .map(|v| v.version_id.clone());

    // Ищем сборку на бэкенде по URL (подойдёт URL любой из прошлых версий —
    // их URL сохраняются в pack_versions). Последняя версия на сервере = versions[0].
    let latest = match auth::mono_pack_id_by_url(&_state.client, &pack.url).await {
        Ok(Some(id)) => auth::mono_pack_detail(&_state.client, "", &id)
            .await
            .ok()
            .and_then(|d| d.versions.first().map(|v| v.version.clone())),
        _ => None,
    };

    // Обновление есть, если последняя версия на сервере отличается от активной
    // и её ещё не устанавливали локально.
    let has_update = matches!((&current, &latest), (Some(cur), Some(latest))
        if latest != cur && !installed.iter().any(|v| &v.version_id == latest));

    Ok(UpdateInfo {
        current_version: current,
        latest_version: latest,
        has_update,
    })
}

/// Полное скачивание и установка сборки.
/// Возвращает URL последней версии сборки с бэкенда, если сборка там найдена по URL.
/// При любых ошибках возвращает None (установка идёт по исходной ссылке).
async fn resolve_latest_url(client: &reqwest::Client, url: &str) -> Option<(String, String)> {
    let norm = |u: &str| u.trim().trim_end_matches('/').to_lowercase();
    let file = url.split('?').next()?.split('#').next()?.rsplit('/').next()?.to_string();
    if file.is_empty() {
        return None;
    }
    let base = crate::config::backend_url().trim_end_matches('/').to_string();
    let cat: Vec<serde_json::Value> = client
        .get(format!("{base}/packs"))
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;
    let id = cat
        .iter()
        .filter_map(|c| {
            let cu = c.get("url")?.as_str()?;
            if norm(cu) == norm(url) || cu.ends_with(&format!("/{file}")) {
                c.get("id")?.as_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .next()?;
    let detail: serde_json::Value = client
        .get(format!("{base}/packs/{id}"))
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;
    let v0 = detail.get("versions")?.get(0)?.clone();
    let u = v0.get("url")?.as_str()?.to_string();
    let label = v0.get("version").and_then(|x| x.as_str()).map(|x| x.to_string());
    Some((u, label.unwrap_or_default()))
}

#[tauri::command]
async fn install_mrpack(
    app: AppHandle,
    state: State<'_, AppState>,
    pack_id: Option<String>,
    _tag: Option<String>,
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
    // Ручная установка конкретной версии (тег передан): URL уже выставлен клиентом.
    // Иначе актуализируем URL: для сборок с бэкенда ставим последнюю версию.
    let (url, label) = if let Some(t) = _tag.as_deref().filter(|t| !t.is_empty()) {
        (pack.url.clone(), Some(t.to_string()))
    } else {
        let mut url = pack.url.clone();
        let mut label: Option<String> = None;
        if let Some((latest, lv)) = resolve_latest_url(&client, &pack.url).await {
            if latest != url {
                url = latest.clone();
                let _ = config::set_pack_url(&pack.id, &latest);
            }
            label = Some(lv);
        }
        (url, label)
    };
    let installed = mrpack::install_mrpack(app, &client, &pack.id, &url, None)
        .await
        .map_err(|e| e.to_string())?;
    // Метка версии: у экспортированных .mrpack внутри часто лежит дефолтная 1.0.0 —
    // показываем метку бэкенда, чтобы «Активная версия» была честной.
    if let Some(tag) = label.filter(|t| !t.is_empty() && *t != installed.version_id) {
        let _ = config::set_active_version(&pack.id, &tag);
        if let (Ok(from), Ok(to)) = (
            config::version_dir(&pack.id, &installed.version_id),
            config::version_dir(&pack.id, &tag),
        ) {
            if from.exists() && !to.exists() {
                let _ = std::fs::rename(&from, &to);
            }
        }
    }
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
        active_source_tag: {
            let active = config::active_version(&pack.id)
                .ok()
                .filter(|v| !v.is_empty());
            active.as_ref().and_then(|a| {
                mrpack::installed_details(&pack.id)
                    .into_iter()
                    .find(|v| v.version_id == *a)
                    .map(|v| v.source_tag.clone().unwrap_or_else(|| v.version_id.clone()))
            })
        },
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
/// Тихо обновляет Microsoft-сессию по refresh_token (если он сохранён).
#[tauri::command]
async fn ms_refresh_session_command(state: State<'_, AppState>) -> Result<Option<auth::UserSession>, String> {
    let Some(session) = auth::load_session().map_err(|e| e.to_string())? else { return Ok(None); };
    if session.user_type != "microsoft" {
        return Ok(Some(session));
    }
    let Some(rt) = session.refresh_token.clone() else { return Ok(Some(session)); };
    match auth::ms_refresh(&state.client, &rt).await {
        Ok(fresh) => {
            save_session(&fresh).map_err(|e| e.to_string())?;
            let _ = auth::upsert_account(&fresh);
            Ok(Some(fresh))
        }
        Err(_) => Ok(Some(session)),
    }
}

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

/// Регистрация аккаунта Mono (собственный бэкенд лаунчера).
/// Профиль Mono хранится отдельно от игровых аккаунтов (mono.json).
#[tauri::command]
async fn mono_register_command(
    state: State<'_, AppState>,
    username: String,
    password: String,
) -> Result<auth::MonoProfile, String> {
    auth::mono_register(&state.client, &username, &password)
        .await
        .map_err(|e| e.to_string())
}

/// Вход в аккаунт Mono.
#[tauri::command]
async fn mono_login_command(
    state: State<'_, AppState>,
    username: String,
    password: String,
) -> Result<auth::MonoProfile, String> {
    auth::mono_login(&state.client, &username, &password)
        .await
        .map_err(|e| e.to_string())
}

/// Отдаёт текущий локальный профиль Mono (или None) — не трогает игровые аккаунты.
#[tauri::command]
fn mono_profile_command() -> Result<Option<auth::MonoProfile>, String> {
    auth::load_mono_profile().map_err(|e| e.to_string())
}

/// Разлогин на сервере Mono (отзывает токены) и удаляет локальный профиль.
#[tauri::command]
async fn mono_logout_command(state: State<'_, AppState>) -> Result<Option<auth::MonoProfile>, String> {
    if let Some(profile) = auth::load_mono_profile().map_err(|e| e.to_string())? {
        auth::mono_logout(&state.client, &profile.access_token).await;
    }
    auth::clear_mono_profile().map_err(|e| e.to_string())?;
    Ok(None)
}

/// Загрузка экспортированной сборки (.mrpack) на storage через бэкенд Mono.
#[tauri::command]
async fn upload_pack_command(
    state: State<'_, AppState>,
    access_token: String,
    file_path: String,
    name: String,
    description: String,
    version: String,
    changelog: String,
    min_ram_mb: Option<i64>,
    boosty_blog: Option<String>,
    meta: Option<serde_json::Value>,
    icon_url: Option<String>,
) -> Result<auth::MonoPackPublic, String> {
    auth::mono_upload_pack(
        &state.client,
        &access_token,
        &file_path,
        &name,
        &description,
        &version,
        &changelog,
        min_ram_mb,
        boosty_blog,
        meta,
        icon_url,
    )
    .await
    .map_err(|e| e.to_string())
}

/// Каталог сборок Mono (публичный).
#[tauri::command]
async fn pack_catalog_command(state: State<'_, AppState>) -> Result<Vec<auth::PackCatalog>, String> {
    auth::mono_pack_catalog(&state.client)
        .await
        .map_err(|e| e.to_string())
}

/// Сборки текущего автора Mono.
#[tauri::command]
async fn pack_mine_command(
    state: State<'_, AppState>,
    access_token: String,
) -> Result<Vec<auth::PackCatalog>, String> {
    auth::mono_pack_mine(&state.client, &access_token)
        .await
        .map_err(|e| e.to_string())
}

/// Новости Mono (глобальные и по сборкам).
#[tauri::command]
async fn pack_news_command(state: State<'_, AppState>) -> Result<Vec<auth::PackNewsPublic>, String> {
    auth::mono_pack_news(&state.client)
        .await
        .map_err(|e| e.to_string())
}

/// Деталь сборки Mono; пустой access_token — без авторизации.
#[tauri::command]
async fn pack_detail_command(
    state: State<'_, AppState>,
    access_token: String,
    id: String,
) -> Result<auth::PackDetail, String> {
    auth::mono_pack_detail(&state.client, &access_token, &id)
        .await
        .map_err(|e| e.to_string())
}

/// Частичное обновление описания сборки.
#[tauri::command]
async fn pack_update_command(
    state: State<'_, AppState>,
    access_token: String,
    id: String,
    body: serde_json::Value,
) -> Result<auth::PackDetail, String> {
    auth::mono_pack_update(&state.client, &access_token, &id, body)
        .await
        .map_err(|e| e.to_string())
}

/// Удаляет сборку с бэкенда Mono и storage.
#[tauri::command]
async fn pack_delete_command(
    state: State<'_, AppState>,
    access_token: String,
    id: String,
) -> Result<(), String> {
    auth::mono_pack_delete(&state.client, &access_token, &id)
        .await
        .map_err(|e| e.to_string())
}

/// Загружает новую версию .mrpack для сборки.
#[tauri::command]
async fn pack_add_version_command(
    state: State<'_, AppState>,
    access_token: String,
    id: String,
    file_path: String,
    version: String,
    changelog: String,
) -> Result<auth::PackVersionPublic, String> {
    auth::mono_pack_add_version(&state.client, &access_token, &id, &file_path, &version, &changelog)
        .await
        .map_err(|e| e.to_string())
}

/// Удаляет версию сборки.
#[tauri::command]
async fn pack_delete_version_command(
    state: State<'_, AppState>,
    access_token: String,
    id: String,
    version_id: String,
) -> Result<(), String> {
    auth::mono_pack_delete_version(&state.client, &access_token, &id, &version_id)
        .await
        .map_err(|e| e.to_string())
}

/// Резолвит id сборки на бэкенде по URL файла (для диплинк-сборок).
#[tauri::command]
async fn pack_id_by_url_command(
    state: State<'_, AppState>,
    url: String,
) -> Result<Option<String>, String> {
    auth::mono_pack_id_by_url(&state.client, &url)
        .await
        .map_err(|e| e.to_string())
}

/// Загружает скриншот сборки на storage (возвращает обновлённую meta).
#[tauri::command]
async fn pack_upload_screenshot_command(
    state: State<'_, AppState>,
    access_token: String,
    id: String,
    file_path: String,
    caption: String,
) -> Result<serde_json::Value, String> {
    auth::mono_pack_upload_screenshot(&state.client, &access_token, &id, &file_path, &caption)
        .await
        .map_err(|e| e.to_string())
}

/// Удаляет скриншот сборки по индексу (возвращает обновлённую meta).
#[tauri::command]
async fn pack_delete_screenshot_command(
    state: State<'_, AppState>,
    access_token: String,
    id: String,
    index: usize,
) -> Result<serde_json::Value, String> {
    auth::mono_pack_delete_screenshot(&state.client, &access_token, &id, index)
        .await
        .map_err(|e| e.to_string())
}

/// Добавляет новость к сборке.
#[tauri::command]
async fn pack_add_news_command(
    state: State<'_, AppState>,
    access_token: String,
    id: String,
    kind: String,
    title: String,
    body: String,
) -> Result<auth::PackNewsPublic, String> {
    auth::mono_pack_add_news(&state.client, &access_token, &id, &kind, &title, &body)
        .await
        .map_err(|e| e.to_string())
}

/// Удаляет новость сборки.
#[tauri::command]
async fn pack_delete_news_command(
    state: State<'_, AppState>,
    access_token: String,
    id: String,
    news_id: String,
) -> Result<(), String> {
    auth::mono_pack_delete_news(&state.client, &access_token, &id, &news_id)
        .await
        .map_err(|e| e.to_string())
}

/// Оценивает сборку (value: 1 или -1).
#[tauri::command]
async fn pack_rate_command(
    state: State<'_, AppState>,
    access_token: String,
    id: String,
    value: i64,
) -> Result<serde_json::Value, String> {
    auth::mono_pack_rate(&state.client, &access_token, &id, value)
        .await
        .map_err(|e| e.to_string())
}

// ==== Комментарии ====

#[tauri::command]
async fn mono_list_comments_command(
    state: State<'_, AppState>,
    pack_id: String,
) -> Result<Vec<auth::CommentWithReplies>, String> {
    auth::mono_list_comments(&state.client, &pack_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn mono_create_comment_command(
    state: State<'_, AppState>,
    access_token: String,
    pack_id: String,
    body: String,
    parent_id: Option<String>,
) -> Result<auth::CommentPublic, String> {
    auth::mono_create_comment(&state.client, &access_token, &pack_id, &body, parent_id.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn mono_update_comment_command(
    state: State<'_, AppState>,
    access_token: String,
    pack_id: String,
    comment_id: String,
    body: String,
) -> Result<auth::CommentPublic, String> {
    auth::mono_update_comment(&state.client, &access_token, &pack_id, &comment_id, &body)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn mono_delete_comment_command(
    state: State<'_, AppState>,
    access_token: String,
    pack_id: String,
    comment_id: String,
) -> Result<(), String> {
    auth::mono_delete_comment(&state.client, &access_token, &pack_id, &comment_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn mono_rate_comment_command(
    state: State<'_, AppState>,
    access_token: String,
    pack_id: String,
    comment_id: String,
    value: i64,
) -> Result<serde_json::Value, String> {
    auth::mono_rate_comment(&state.client, &access_token, &pack_id, &comment_id, value)
        .await
        .map_err(|e| e.to_string())
}

// ==== Профили ====

#[tauri::command]
async fn mono_get_profile_command(
    state: State<'_, AppState>,
    user_id: String,
) -> Result<auth::ProfilePublic, String> {
    auth::mono_get_profile(&state.client, &user_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn mono_get_profile_full_command(
    state: State<'_, AppState>,
    user_id: String,
) -> Result<auth::ProfileDetail, String> {
    auth::mono_get_profile_full(&state.client, &user_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn mono_update_profile_command(
    state: State<'_, AppState>,
    access_token: String,
    bio: Option<String>,
    avatar_url: Option<String>,
) -> Result<auth::ProfilePublic, String> {
    auth::mono_update_profile(&state.client, &access_token, bio.as_deref(), avatar_url.as_deref())
        .await
        .map_err(|e| e.to_string())
}

// ==== Сканер модов ====

#[tauri::command]
async fn mono_scan_mod_command(
    state: State<'_, AppState>,
    access_token: String,
    file_path: String,
) -> Result<auth::ScanResult, String> {
    auth::mono_scan_mod(&state.client, &access_token, &file_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn mono_check_hash_command(
    state: State<'_, AppState>,
    sha256: String,
) -> Result<auth::ScanResult, String> {
    auth::mono_check_hash(&state.client, &sha256)
        .await
        .map_err(|e| e.to_string())
}

// ==== Соавторы ====

#[tauri::command]
async fn mono_list_collaborators_command(
    state: State<'_, AppState>,
    access_token: String,
    pack_id: String,
) -> Result<Vec<auth::CollaboratorPublic>, String> {
    auth::mono_list_collaborators(&state.client, &access_token, &pack_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn mono_add_collaborator_command(
    state: State<'_, AppState>,
    access_token: String,
    pack_id: String,
    username: String,
    perm_edit_meta: bool,
    perm_manage_versions: bool,
    perm_manage_news: bool,
) -> Result<auth::CollaboratorPublic, String> {
    auth::mono_add_collaborator(
        &state.client, &access_token, &pack_id, &username,
        perm_edit_meta, perm_manage_versions, perm_manage_news,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn mono_update_collaborator_command(
    state: State<'_, AppState>,
    access_token: String,
    pack_id: String,
    collab_id: String,
    perm_edit_meta: Option<bool>,
    perm_manage_versions: Option<bool>,
    perm_manage_news: Option<bool>,
) -> Result<auth::CollaboratorPublic, String> {
    auth::mono_update_collaborator(
        &state.client, &access_token, &pack_id, &collab_id,
        perm_edit_meta, perm_manage_versions, perm_manage_news,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn mono_remove_collaborator_command(
    state: State<'_, AppState>,
    access_token: String,
    pack_id: String,
    collab_id: String,
) -> Result<(), String> {
    auth::mono_remove_collaborator(&state.client, &access_token, &pack_id, &collab_id)
        .await
        .map_err(|e| e.to_string())
}

// ==== Админ ====

#[tauri::command]
async fn mono_admin_list_users_command(
    state: State<'_, AppState>,
    access_token: String,
) -> Result<Vec<auth::AdminUser>, String> {
    auth::mono_admin_list_users(&state.client, &access_token)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn mono_admin_list_packs_command(
    state: State<'_, AppState>,
    access_token: String,
) -> Result<Vec<auth::AdminPack>, String> {
    auth::mono_admin_list_packs(&state.client, &access_token)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn mono_admin_list_comments_command(
    state: State<'_, AppState>,
    access_token: String,
) -> Result<Vec<auth::AdminComment>, String> {
    auth::mono_admin_list_comments(&state.client, &access_token)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn mono_admin_create_user_command(
    state: State<'_, AppState>,
    access_token: String,
    payload: auth::AdminCreateUser,
) -> Result<auth::AdminUser, String> {
    auth::mono_admin_create_user(&state.client, &access_token, &payload)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn mono_admin_ban_user_command(
    state: State<'_, AppState>,
    access_token: String,
    user_id: String,
    reason: Option<String>,
) -> Result<(), String> {
    auth::mono_admin_ban_user(&state.client, &access_token, &user_id, reason.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn mono_admin_unban_user_command(
    state: State<'_, AppState>,
    access_token: String,
    user_id: String,
) -> Result<(), String> {
    auth::mono_admin_unban_user(&state.client, &access_token, &user_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn mono_admin_delete_user_command(
    state: State<'_, AppState>,
    access_token: String,
    user_id: String,
) -> Result<(), String> {
    auth::mono_admin_delete_user(&state.client, &access_token, &user_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn mono_admin_delete_pack_command(
    state: State<'_, AppState>,
    access_token: String,
    pack_id: String,
) -> Result<(), String> {
    auth::mono_admin_delete_pack(&state.client, &access_token, &pack_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn mono_admin_delete_comment_command(
    state: State<'_, AppState>,
    access_token: String,
    comment_id: String,
) -> Result<(), String> {
    auth::mono_admin_delete_comment(&state.client, &access_token, &comment_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn mono_admin_set_role_command(
    state: State<'_, AppState>,
    access_token: String,
    user_id: String,
    role: String,
) -> Result<(), String> {
    auth::mono_admin_set_role(&state.client, &access_token, &user_id, &role)
        .await
        .map_err(|e| e.to_string())
}

// ==== Auth v2 ====

#[tauri::command]
async fn mono_forgot_password_command(
    state: State<'_, AppState>,
    email: String,
) -> Result<(), String> {
    auth::mono_forgot_password(&state.client, &email)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn mono_reset_password_command(
    state: State<'_, AppState>,
    token: String,
    password: String,
) -> Result<(), String> {
    auth::mono_reset_password(&state.client, &token, &password)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn mono_confirm_email_command(
    state: State<'_, AppState>,
    access_token: String,
) -> Result<(), String> {
    auth::mono_confirm_email(&state.client, &access_token)
        .await
        .map_err(|e| e.to_string())
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
    let pack_id = pack_id.unwrap_or_else(default_pack_id);
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
/// Опционально сохраняет пару для автопродления access-токена (окно входа Boosty).
#[tauri::command]
async fn set_boosty_command(
    state: State<'_, AppState>,
    pack_id: String,
    token: String,
    refresh_token: Option<String>,
    device_id: Option<String>,
    token_expires_at: Option<u64>,
) -> Result<license::LicenseInfo, String> {
    license::set_license(
        &state.client,
        &pack_id,
        &token,
        refresh_token,
        device_id,
        token_expires_at,
    )
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

/// Сохраняет глобальную привязку аккаунта Boosty (на лаунчер целиком).
/// Любая платная сборка без собственной записи использует её токены.
#[tauri::command]
fn set_global_boosty_command(
    token: String,
    refresh_token: Option<String>,
    device_id: Option<String>,
    token_expires_at: Option<u64>,
) -> Result<(), String> {
    license::set_global_license(&token, refresh_token, device_id, token_expires_at)
        .map_err(|e| e.to_string())
}

/// Привязан ли глобальный аккаунт Boosty.
#[tauri::command]
fn global_boosty_linked_command() -> Result<bool, String> {
    Ok(license::global_linked())
}

/// Удаляет глобальную привязку аккаунта Boosty.
#[tauri::command]
fn clear_global_boosty_command() -> Result<(), String> {
    license::clear_global_license().map_err(|e| e.to_string())
}

/// Скрипт, внедряемый в окно входа Boosty: следит за авторизацией и, когда
/// Boosty сохранит токены в localStorage, перенаправляет окно на
/// `https://boosty.to/?__mono_auth=<encodeURIComponent(JSON)>`.
/// Rust читает URL окна и забирает токены (без IPC с чужого домена).
const BOOSTY_CAPTURE_JS: &str = r#"
(() => {
  try {
    if (window.top !== window) return;
    if (!location.origin.includes('boosty.to')) return;
    if (sessionStorage.getItem('__mono_boosty_done')) return;
    const tick = setInterval(() => {
      try {
        const raw = localStorage.getItem('auth');
        if (!raw) return;
        const a = JSON.parse(raw);
        const access = a.accessToken || a.access_token || '';
        const refresh = a.refreshToken || a.refresh_token || '';
        const dev = localStorage.getItem('_clientId') || '';
        if (!access) return;
        let exp = a.expiresAt || a.expires_at || 0;
        if (typeof exp === 'number' && exp > 1e12) exp = Math.floor(exp / 1000);
        sessionStorage.setItem('__mono_boosty_done', '1');
        clearInterval(tick);
        const payload = JSON.stringify({ t: access, r: refresh, d: dev, e: exp });
        location.replace('https://boosty.to/?__mono_auth=' + encodeURIComponent(payload));
      } catch (e) {}
    }, 700);
  } catch (e) {}
})();
"#;

/// Открывает окно входа Boosty (отдельный WebviewWindow с блогом пользователя).
#[tauri::command]
fn boosty_login_begin_command(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("boosty") {
        let _ = win.close();
    }
    let url: tauri::Url = "https://boosty.to/"
        .parse()
        .map_err(|e| format!("Некорректный URL Boosty: {e}"))?;
    tauri::WebviewWindowBuilder::new(&app, "boosty", tauri::WebviewUrl::External(url))
        .title("Вход в Boosty")
        .inner_size(430.0, 700.0)
        .resizable(true)
        .initialization_script(BOOSTY_CAPTURE_JS)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Разбирает URL окна Boosty и извлекает захваченные токены (`__mono_auth`).
fn parse_boosty_auth_url(url: &str) -> Option<license::BoostyAuth> {
    let (_, query) = url.split_once('?')?;
    for pair in query.split('&') {
        let (key, value) = pair.split_once('=')?;
        if key != "__mono_auth" {
            continue;
        }
        let raw = pct_decode(value);
        let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
        let access = v.get("t").and_then(|x| x.as_str()).filter(|s| !s.is_empty())?;
        let refresh = v.get("r").and_then(|x| x.as_str()).unwrap_or("");
        let device = v.get("d").and_then(|x| x.as_str()).unwrap_or("");
        let exp = v.get("e").and_then(|x| x.as_u64()).unwrap_or(0);
        return Some(license::BoostyAuth {
            access_token: access.to_string(),
            refresh_token: refresh.to_string(),
            device_id: device.to_string(),
            token_expires_at: exp,
        });
    }
    None
}

/// Опрос окна входа Boosty: вернул токены — забираем их и закрываем окно.
#[tauri::command]
fn boosty_poll_command(app: AppHandle) -> Result<Option<license::BoostyAuth>, String> {
    let Some(win) = app.get_webview_window("boosty") else {
        return Ok(None);
    };
    let url = win.url().map_err(|e| e.to_string())?;
    if let Some(auth) = parse_boosty_auth_url(url.as_str()) {
        let _ = win.close();
        return Ok(Some(auth));
    }
    Ok(None)
}

/// Закрывает окно входа Boosty (отмена с фронтенда).
#[tauri::command]
fn boosty_login_cancel_command(app: AppHandle) {
    if let Some(win) = app.get_webview_window("boosty") {
        let _ = win.close();
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ScreenshotInfo {
    path: String,
    modified: i64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ScreenshotList {
    installed: bool,
    screenshots: Vec<ScreenshotInfo>,
}

/// Скриншоты активной установленной версии: папка `screenshots` игрового
/// каталога (полные пути — фронт отдаёт их через asset-протокол).
/// `modified` — время создания/изменения файла (epoch, секунды) для даты скрина.
#[tauri::command]
fn list_screenshots_command(pack_id: Option<String>) -> Result<ScreenshotList, String> {
    let pack_id = pack_id.unwrap_or_else(default_pack_id);
    let installed = config::active_version_file(&pack_id)
        .map(|f| f.exists())
        .unwrap_or(false);
    let mut screenshots = Vec::new();
    if installed {
        if let Ok(dir) = config::active_game_dir(&pack_id) {
            let shots_dir = dir.join("screenshots");
            if shots_dir.is_dir() {
                if let Ok(rd) = std::fs::read_dir(&shots_dir) {
                    let mut files: Vec<ScreenshotInfo> = rd
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
                        .filter_map(|e| {
                            let modified = e
                                .metadata()
                                .ok()
                                .and_then(|m| m.modified().ok())
                                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                .map(|d| d.as_secs() as i64)
                                .unwrap_or(0);
                            Some(ScreenshotInfo {
                                path: e.path().to_string_lossy().to_string(),
                                modified,
                            })
                        })
                        .collect();
                    files.sort_by(|a, b| a.modified.cmp(&b.modified));
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

/// Один файл-дубликат: путь, папка (mods/resourcepacks/shaderpacks) и имя —
/// удобно для показа и удаления.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DuplicateFile {
    path: String,
    folder: String,
    name: String,
}

/// Группа одинаковых по содержимому файлов (sha1). `size_bytes` — размер одного.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DuplicateGroup {
    files: Vec<DuplicateFile>,
    size_bytes: u64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DuplicatesResult {
    groups: Vec<DuplicateGroup>,
    wasted_bytes: u64,
}

/// Анализ дубликатов в папках mods / resourcepacks / shaderpacks активной
/// версии: группируем по размеру, затем по sha1 содержимого (полное чтение
/// только среди файлов одинакового размера). `wasted_bytes` — объём, который
/// освободится, если в каждой группе оставить по одному файлу.
#[tauri::command]
fn analyze_duplicates_command(pack_id: Option<String>) -> Result<DuplicatesResult, String> {
    let pack_id = pack_id.unwrap_or_else(default_pack_id);
    let dir = config::active_game_dir(&pack_id).map_err(|e| e.to_string())?;

    // Сначала по размеру файла — дешёвый фильтр, чтобы не хэшировать всё подряд.
    let mut by_size: HashMap<u64, Vec<std::path::PathBuf>> = HashMap::new();
    for sub in ["mods", "resourcepacks", "shaderpacks"] {
        let p = dir.join(sub);
        if !p.is_dir() {
            continue;
        }
        if let Ok(rd) = std::fs::read_dir(&p) {
            for entry in rd.flatten() {
                let Ok(ft) = entry.file_type() else { continue };
                if !ft.is_file() {
                    continue;
                }
                let Ok(meta) = entry.metadata() else { continue };
                if meta.len() == 0 {
                    continue;
                }
                by_size
                    .entry(meta.len())
                    .or_default()
                    .push(entry.path());
            }
        }
    }

    let mut groups: Vec<DuplicateGroup> = Vec::new();
    for (size, files) in by_size {
        if files.len() < 2 {
            continue;
        }
        let mut by_hash: HashMap<String, Vec<std::path::PathBuf>> = HashMap::new();
        for f in files {
            if let Ok(h) = crate::mrpack::compute_sha1(&f) {
                by_hash.entry(h).or_default().push(f);
            }
        }
        for (_, dup) in by_hash {
            if dup.len() < 2 {
                continue;
            }
            let folder = dup[0]
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            let files: Vec<DuplicateFile> = dup
                .iter()
                .map(|p| DuplicateFile {
                    path: p.to_string_lossy().to_string(),
                    folder: folder.clone(),
                    name: p
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                })
                .collect();
            groups.push(DuplicateGroup {
                files,
                size_bytes: size,
            });
        }
    }

    // Сначала группы с наибольшим «мусором» (n-1)×size.
    groups.sort_by(|a, b| {
        let wa = (a.files.len() as u64 - 1) * a.size_bytes;
        let wb = (b.files.len() as u64 - 1) * b.size_bytes;
        wb.cmp(&wa)
    });
    let wasted_bytes = groups
        .iter()
        .map(|g| (g.files.len() as u64 - 1) * g.size_bytes)
        .sum();

    Ok(DuplicatesResult {
        groups,
        wasted_bytes,
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
    let pack_id = pack_id.unwrap_or_else(default_pack_id);
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
    let pack_id = pack_id.unwrap_or_else(default_pack_id);
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

/// Флаг «крестик сворачивает в трей» (читается в on_window_event).
fn close_to_tray_flag() -> &'static std::sync::atomic::AtomicBool {
    static FLAG: std::sync::OnceLock<std::sync::atomic::AtomicBool> = std::sync::OnceLock::new();
    FLAG.get_or_init(|| std::sync::atomic::AtomicBool::new(false))
}

/// Включает/выключает сворачивание в трей при закрытии окна.
#[tauri::command]
fn set_close_to_tray_command(enabled: bool) {
    close_to_tray_flag().store(enabled, std::sync::atomic::Ordering::Relaxed);
}

/// Автозапуск лаунчера вместе с системой (tauri-plugin-autostart).
#[tauri::command]
fn autostart_set_command(app: AppHandle, enabled: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    let manager = app.autolaunch();
    if enabled {
        manager.enable().map_err(|e| e.to_string())
    } else {
        manager.disable().map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn autostart_get_command(app: AppHandle) -> bool {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch().is_enabled().unwrap_or(false)
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
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        // Single-instance должен быть первым: ловит deep link аргументы
        // запущенного экземпляра на Linux/Windows.
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if let Some(url) = argv.iter().find(|a| a.starts_with(DEEP_LINK_PREFIX)) {
                handle_deep_link(app, url);
            }
        }))
        .plugin(tauri_plugin_deep_link::init())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if close_to_tray_flag().load(std::sync::atomic::Ordering::Relaxed) {
                    // Сворачиваем в трей вместо выхода.
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .setup(|app| {
            #[cfg(desktop)]
            register_deep_link_handlers(app.handle());

            // Трей: показать окно / выйти.
            #[cfg(desktop)]
            {
                use tauri::menu::{Menu, MenuItem};
                use tauri::tray::TrayIconBuilder;

                let show = MenuItem::with_id(app, "show", "Показать Mono", true, None::<&str>)?;
                let quit = MenuItem::with_id(app, "quit", "Выход", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&show, &quit])?;
                let _tray = TrayIconBuilder::with_id("main-tray")
                    .icon(app.default_window_icon().cloned().unwrap())
                    .tooltip("Mono Launcher")
                    .menu(&menu)
                    .show_menu_on_left_click(false)
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "show" => {
                            if let Some(win) = app.get_webview_window("main") {
                                let _ = win.show();
                                let _ = win.unminimize();
                                let _ = win.set_focus();
                            }
                        }
                        "quit" => app.exit(0),
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let tauri::tray::TrayIconEvent::Click { button, .. } = event {
                            if button == tauri::tray::MouseButton::Left {
                                if let Some(win) =
                                    tray.app_handle().get_webview_window("main")
                                {
                                    if win.is_visible().unwrap_or(false) {
                                        let _ = win.set_focus();
                                    } else {
                                        let _ = win.show();
                                        let _ = win.set_focus();
                                    }
                                }
                            }
                        }
                    })
                    .build(app)?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_packs,
            add_pack_command,
            add_pack_file_command,
            remove_pack_command,
            set_close_to_tray_command,
            autostart_set_command,
            autostart_get_command,
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
            ms_refresh_session_command,
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
            mono_register_command,
            mono_login_command,
            mono_profile_command,
            mono_logout_command,
            upload_pack_command,
            pack_catalog_command,
            pack_mine_command,
            pack_news_command,
            pack_detail_command,
            pack_update_command,
            pack_delete_command,
            pack_add_version_command,
            pack_delete_version_command,
            pack_upload_screenshot_command,
            pack_delete_screenshot_command,
            pack_add_news_command,
            pack_delete_news_command,
            pack_rate_command,
            launch_game_command,
            game::stop_game_command,
            ping_server_command,
            get_local_skin_command,
            set_local_skin_command,
            clear_local_skin_command,
            skin_api_url_command,
            list_screenshots_command,
            analyze_duplicates_command,
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
            set_boosty_command,
            license_status_command,
            clear_license_command,
            set_global_boosty_command,
            global_boosty_linked_command,
            clear_global_boosty_command,
            boosty_login_begin_command,
            boosty_poll_command,
            boosty_login_cancel_command,
            modrinth_search_command,
            modrinth_tags_command,
            modrinth_project_versions_command,
            modrinth_project_command,
            set_pack_icon_command,
            set_pack_banner_command,
            set_pack_name_command,
            set_pack_url_command,
            pack_id_by_url_command,
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
            mono_list_comments_command,
            mono_create_comment_command,
            mono_update_comment_command,
            mono_delete_comment_command,
            mono_rate_comment_command,
            mono_get_profile_command,
            mono_get_profile_full_command,
            mono_update_profile_command,
            mono_scan_mod_command,
            mono_check_hash_command,
            mono_list_collaborators_command,
            mono_add_collaborator_command,
            mono_update_collaborator_command,
            mono_remove_collaborator_command,
            mono_admin_list_users_command,
            mono_admin_list_packs_command,
            mono_admin_list_comments_command,
            mono_admin_create_user_command,
            mono_admin_ban_user_command,
            mono_admin_unban_user_command,
            mono_admin_delete_user_command,
            mono_admin_delete_pack_command,
            mono_admin_delete_comment_command,
            mono_admin_set_role_command,
            mono_forgot_password_command,
            mono_reset_password_command,
            mono_confirm_email_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
