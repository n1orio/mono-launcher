use std::path::PathBuf;

use anyhow::Result;
use dirs::data_dir;
use serde::{Deserialize, Serialize};

/// Описание сборки: id используется в путях и командах IPC.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PackDef {
    pub id: String,
    pub name: String,
    pub url: String,
    /// Ник блога на Boosty: задан → сборка платная (подписка обязательна).
    #[serde(default, rename = "boostyBlog")]
    pub boosty_blog: Option<String>,
    /// Минимальная оперативка (МБ) для запуска сборки: задан → лаунчер
    /// предупреждает и не даёт запустить при меньшем выделении.
    #[serde(default, rename = "minRam")]
    pub min_ram_mb: Option<u32>,
}

/// Пользовательская сборка из реестра `packs.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPack {
    pub id: String,
    pub name: String,
    pub url: String,
    /// "remote" — сборка из GitHub Releases (.mrpack), "local" — своя сборка
    /// (создана в лаунчере или скачана с Modrinth).
    #[serde(default = "default_kind")]
    pub kind: String,
    /// Ник блога на Boosty: задан → сборка платная (подписка обязательна).
    /// В сборки, добавленные по ссылке, попадает из pack.json издателя.
    #[serde(default, rename = "boostyBlog")]
    pub boosty_blog: Option<String>,
    /// Минимальная оперативка (МБ): попадает из pack.json при добавлении.
    #[serde(default, rename = "minRam")]
    pub min_ram_mb: Option<u32>,
}

fn default_kind() -> String {
    "remote".into()
}

/// Единое описание сборки — встроенной или добавленной пользователем.
#[derive(Debug, Clone)]
pub struct PackInfo {
    pub id: String,
    pub name: String,
    pub url: String,
    pub builtin: bool,
    /// "remote" | "local" (см. `UserPack::kind`).
    pub kind: String,
    /// Ник блога на Boosty: задан → сборка платная (подписка обязательна).
    pub boosty_blog: Option<String>,
    /// Минимальная оперативка (МБ), см. `PackDef::min_ram_mb`.
    pub min_ram_mb: Option<u32>,
    /// Локальная иконка сборки (путь к `packs/<id>/icon.png`), если есть.
    pub icon: Option<String>,
    /// Локальный баннер сборки (путь к `packs/<id>/banner.png`), если есть.
    pub banner: Option<String>,
}

/// Файл, из которого читаются встроенные сборки. Обновляется из `builtin-packs.json`
/// репозитория лаунчера при старте/по команде — так сборки меняются без пересборки.
fn builtin_packs_file() -> Result<PathBuf> {
    Ok(launcher_root()?.join("builtin-packs.json"))
}

/// Встроенные сборки из локального файла (обновляется из репозитория).
/// Если файла нет или он битый — возвращаем пустой список: лаунчер сам
/// подтянет актуальный список из `builtin-packs.json` репозитория.
pub fn builtin_packs() -> Vec<PackDef> {
    let Ok(path) = builtin_packs_file() else {
        return Vec::new();
    };
    let Ok(raw) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    serde_json::from_str::<Vec<PackDef>>(&raw).unwrap_or_default()
}

/// Сохраняет список встроенных сборок (из `builtin-packs.json` репозитория),
/// чтобы следующие запуски читали его без сети.
pub fn save_builtin_packs(list: &[PackDef]) -> Result<()> {
    let file = builtin_packs_file()?;
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = file.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(list)?)?;
    std::fs::rename(&tmp, &file)?;
    Ok(())
}

/// Первая встроенная сборка как дефолтная (иначе пустая строка).
pub fn default_pack_id() -> String {
    builtin_packs()
        .first()
        .map(|p| p.id.clone())
        .unwrap_or_default()
}

/// Файл реестра пользовательских сборок.
fn user_packs_file() -> Result<PathBuf> {
    Ok(launcher_root()?.join("packs.json"))
}

/// Все сборки, добавленные пользователем (пусто, если файла нет).
pub fn user_packs() -> Result<Vec<UserPack>> {
    let file = user_packs_file()?;
    if !file.exists() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(&file)?;
    match serde_json::from_str(&raw) {
        Ok(list) => Ok(list),
        Err(_) => {
            // Битый файл не выбрасываем молча: бэкапим, чтобы данные можно было
            // восстановить, а пользователю возвращаем пустой список.
            if let Some(parent) = file.parent() {
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let _ = std::fs::rename(&file, parent.join(format!("packs.corrupt-{ts}.json")));
            }
            Ok(Vec::new())
        }
    }
}

fn save_user_packs(list: &[UserPack]) -> Result<()> {
    let file = user_packs_file()?;
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // Атомарно: пишем во временный файл и переименовываем, чтобы сбой на середине
    // не оставил битый packs.json (и не потерял список сборок).
    let tmp = file.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(list)?)?;
    std::fs::rename(&tmp, &file)?;
    Ok(())
}

/// Все сборки: встроенные + пользовательские.
pub fn all_packs() -> Result<Vec<PackInfo>> {
    let mut out = builtin_packs()
        .into_iter()
        .map(|p| PackInfo {
            id: p.id.clone(),
            name: pack_name(&p.id, &p.name),
            url: p.url,
            builtin: true,
            kind: "remote".into(),
            boosty_blog: p.boosty_blog,
            min_ram_mb: p.min_ram_mb,
            icon: pack_icon_path(&p.id),
            banner: pack_banner_path(&p.id),
        })
        .collect::<Vec<_>>();
    for p in user_packs()? {
        let icon = pack_icon_path(&p.id);
        let banner = pack_banner_path(&p.id);
        out.push(PackInfo {
            id: p.id,
            name: p.name,
            url: p.url,
            builtin: false,
            kind: p.kind,
            boosty_blog: p.boosty_blog,
            min_ram_mb: p.min_ram_mb,
            icon,
            banner,
        });
    }
    Ok(out)
}

/// Ищет сборку (сначала встроенную, потом пользовательскую).
pub fn find_pack(id: &str) -> Result<Option<PackInfo>> {
    if let Some(p) = builtin_packs().into_iter().find(|p| p.id == id) {
        return Ok(Some(PackInfo {
            id: p.id.clone(),
            name: pack_name(&p.id, &p.name),
            url: p.url,
            builtin: true,
            kind: "remote".into(),
            boosty_blog: p.boosty_blog,
            min_ram_mb: p.min_ram_mb,
            icon: pack_icon_path(&p.id),
            banner: pack_banner_path(&p.id),
        }));
    }
    Ok(user_packs()?
        .into_iter()
        .find(|p| p.id == id)
        .map(|p| {
            let icon = pack_icon_path(&p.id);
            let banner = pack_banner_path(&p.id);
            PackInfo {
                id: p.id,
                name: p.name,
                url: p.url,
                builtin: false,
                kind: p.kind,
                boosty_blog: p.boosty_blog,
                min_ram_mb: p.min_ram_mb,
                icon,
                banner,
            }
        }))
}

/// Пользовательское имя встроенной сборки (`packs/<id>/name.txt`), если задано.
pub fn pack_name_override(pack_id: &str) -> Option<String> {
    let path = pack_dir(pack_id).ok()?.join("name.txt");
    let raw = std::fs::read_to_string(path).ok()?;
    let t = raw.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// Имя сборки с учётом пользовательского переопределения (только для встроенных).
fn pack_name(pack_id: &str, default: &str) -> String {
    pack_name_override(pack_id).unwrap_or_else(|| default.to_string())
}

/// Устанавливает новое название сборки:
/// — встроенная: переопределение в `packs/<id>/name.txt`;
/// — пользовательская: обновляет `name` в `packs.json`.
pub fn set_pack_name(pack_id: &str, name: &str) -> Result<()> {
    let name = name.trim();
    if name.is_empty() {
        return Err(anyhow::anyhow!("Название не может быть пустым"));
    }
    if builtin_packs().iter().any(|p| p.id == pack_id) {
        let path = pack_dir(pack_id)?.join("name.txt");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, name)?;
        return Ok(());
    }
    let mut list = user_packs()?;
    let mut found = false;
    for p in list.iter_mut() {
        if p.id == pack_id {
            p.name = name.to_string();
            found = true;
            break;
        }
    }
    if !found {
        return Err(anyhow::anyhow!("Сборка не найдена: {pack_id}"));
    }
    save_user_packs(&list)
}

/// Файл с временем последнего запуска сборок (map `pack_id` → unix-секунды).
fn recent_packs_file() -> Result<PathBuf> {
    Ok(launcher_root()?.join("recent.json"))
}

/// Читает карту последних запусков (pack_id → unix-секунды).
fn read_recent() -> std::collections::HashMap<String, u64> {
    let Ok(path) = recent_packs_file() else { return Default::default() };
    let Ok(raw) = std::fs::read_to_string(path) else { return Default::default() };
    serde_json::from_str(&raw).unwrap_or_default()
}

/// Отмечает запуск сборки: фиксирует текущее время последнего запуска.
pub fn mark_pack_launched(pack_id: &str) {
    let Ok(path) = recent_packs_file() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut map = read_recent();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    map.insert(pack_id.to_string(), now);
    // Ограничиваем рост файла: держим только актуальные записи.
    let mut sorted: Vec<(String, u64)> = map.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted.truncate(14);
    let trimmed: std::collections::HashMap<String, u64> = sorted.into_iter().collect();
    let _ = std::fs::write(&path, serde_json::to_string(&trimmed).unwrap_or_default());
}

/// Id сборок по убыванию времени последнего запуска.
pub fn recent_pack_ids() -> Vec<String> {
    let mut v: Vec<(String, u64)> = read_recent().into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1));
    v.into_iter().map(|(id, _)| id).collect()
}

/// Добавляет пользовательскую сборку в реестр. Ошибка, если id занят.
/// `kind` — "remote" или "local" (см. `UserPack::kind`).
pub fn add_user_pack(
    id: &str,
    name: &str,
    url: &str,
    kind: &str,
    boosty_blog: Option<&str>,
    min_ram_mb: Option<u32>,
) -> Result<()> {
    let mut list = user_packs()?;
    if list.iter().any(|p| p.id == id) {
        return Err(anyhow::anyhow!("Сборка с таким id уже добавлена: {id}"));
    }
    list.push(UserPack {
        id: id.to_string(),
        name: name.to_string(),
        url: url.to_string(),
        kind: if kind == "local" { "local".into() } else { "remote".into() },
        boosty_blog: boosty_blog
            .map(|b| b.trim().to_string())
            .filter(|b| !b.is_empty()),
        min_ram_mb,
    });
    save_user_packs(&list)
}

/// Удаляет пользовательскую сборку из реестра. Возвращает false, если её не было.
pub fn remove_user_pack(id: &str) -> Result<bool> {
    let mut list = user_packs()?;
    let before = list.len();
    list.retain(|p| p.id != id);
    if list.len() == before {
        return Ok(false);
    }
    save_user_packs(&list)?;
    // Заодно чистим локальные данные сборки.
    let dir = pack_dir(id)?;
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    Ok(true)
}

/// Базовая папка всех данных лаунчера.
pub fn launcher_root() -> Result<PathBuf> {
    let base = data_dir()
        .ok_or_else(|| anyhow::anyhow!("Не удалось определить папку данных"))?
        .join("MonoLauncher");
    Ok(base)
}

/// Папка конкретной сборки (профили, кэш mrpack, активная версия).
pub fn pack_dir(pack_id: &str) -> Result<PathBuf> {
    Ok(launcher_root()?.join("packs").join(pack_id))
}

/// Файл-флаг «сборка заблокирована» (наличие = правки отключены).
/// Пока сборка заблокирована, лаунчер запрещает менять её файлы; разблокировка
/// (отвязка) отдаёт владение пользователю.
fn pack_lock_file(pack_id: &str) -> Result<PathBuf> {
    Ok(pack_dir(pack_id)?.join(".mono-lock"))
}

/// Заблокирована ли сборка (правки файлов отключены).
pub fn pack_locked(pack_id: &str) -> bool {
    pack_lock_file(pack_id).map(|p| p.exists()).unwrap_or(false)
}

/// Включает/снимает блокировку правок сборки.
pub fn set_pack_locked(pack_id: &str, locked: bool) -> Result<()> {
    let path = pack_lock_file(pack_id)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if locked {
        std::fs::write(&path, b"locked")?;
    } else {
        let _ = std::fs::remove_file(&path);
    }
    Ok(())
}

/// Путь к иконке сборки (`packs/<id>/icon.png`) — локальный файл,
/// подставляется в `PackInfo::icon`, если существует.
pub fn pack_icon_path(pack_id: &str) -> Option<String> {
    let path = pack_dir(pack_id).ok()?.join("icon.png");
    path.exists().then(|| path.to_string_lossy().into_owned())
}

/// Путь к баннеру сборки (`packs/<id>/banner.png`) — локальный файл,
/// подставляется в `PackInfo::banner`, если существует.
pub fn pack_banner_path(pack_id: &str) -> Option<String> {
    let path = pack_dir(pack_id).ok()?.join("banner.png");
    path.exists().then(|| path.to_string_lossy().into_owned())
}

/// Папка установленных версий сборки.
pub fn versions_root(pack_id: &str) -> Result<PathBuf> {
    Ok(pack_dir(pack_id)?.join("versions"))
}

/// Папка игры конкретной версии.
pub fn version_dir(pack_id: &str, version_id: &str) -> Result<PathBuf> {
    Ok(versions_root(pack_id)?.join(version_id))
}

/// Файл активной версии сборки.
pub fn active_version_file(pack_id: &str) -> Result<PathBuf> {
    Ok(pack_dir(pack_id)?.join("active.json"))
}

/// Возвращает активную версию или пустую строку.
pub fn active_version(pack_id: &str) -> Result<String> {
    let path = active_version_file(pack_id)?;
    if !path.exists() {
        return Ok(String::new());
    }
    let raw = std::fs::read_to_string(&path)?;
    let json: serde_json::Value = serde_json::from_str(&raw)?;
    Ok(json["versionId"].as_str().unwrap_or("").to_string())
}

pub fn set_active_version(pack_id: &str, version_id: &str) -> Result<()> {
    let path = active_version_file(pack_id)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(
        &path,
        serde_json::json!({ "versionId": version_id }).to_string(),
    )?;
    Ok(())
}

/// Папка игры, которая используется при запуске (активная версия сборки).
pub fn active_game_dir(pack_id: &str) -> Result<PathBuf> {
    let active = active_version(pack_id)?;
    if active.is_empty() {
        Ok(versions_root(pack_id)?.join("main"))
    } else {
        version_dir(pack_id, &active)
    }
}

/// Папка, куда распаковывается и кэшируется `.mrpack`.
pub fn mrpack_cache_dir(pack_id: &str) -> Result<PathBuf> {
    Ok(pack_dir(pack_id)?.join("mrpack-cache"))
}

/// Папка скачанного `.mrpack` файла.
pub fn mrpack_file_path(pack_id: &str) -> Result<PathBuf> {
    Ok(mrpack_cache_dir(pack_id)?.join("modpack.mrpack"))
}

/// Файл лога запуска игры (перезаписывается при каждом запуске).
pub fn launch_log_file() -> Result<PathBuf> {
    Ok(launcher_root()?.join("launch.log"))
}

/// Папка установленной Java.
pub fn java_root() -> Result<PathBuf> {
    Ok(launcher_root()?.join("runtime"))
}

/// Файл с выбранной пользователем Java (путь, или пусто = авто).
fn java_selection_file() -> Result<PathBuf> {
    Ok(launcher_root()?.join("java.txt"))
}

/// Выбранная пользователем Java, или None если авто (детект).
pub fn java_selection() -> Option<String> {
    let path = java_selection_file().ok()?;
    let raw = std::fs::read_to_string(path).ok()?;
    let t = raw.trim().to_string();
    if t.is_empty() {
        None
    } else {
        Some(t)
    }
}

/// Сохраняет выбор Java («auto»/пусто = автоматически).
pub fn set_java_selection(path: Option<&str>) -> Result<()> {
    let file = java_selection_file()?;
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    match path {
        Some(p) if !p.trim().is_empty() => std::fs::write(&file, p.trim())?,
        _ => {
            if file.exists() {
                std::fs::remove_file(&file)?;
            }
        }
    }
    Ok(())
}

/// Файл с версией Java для конкретной сборки (мажорный номер, или пусто = авто).
fn pack_java_file(pack_id: &str) -> Result<PathBuf> {
    Ok(pack_dir(pack_id)?.join("java.txt"))
}

/// Заданная для сборки версия Java (мажорный номер), если пользователь её указал.
pub fn pack_java(pack_id: &str) -> Option<u32> {
    pack_java_file(pack_id)
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| s.trim().parse().ok())
}

/// Сохраняет версию Java для сборки (None — авто по версии Minecraft).
pub fn set_pack_java(pack_id: &str, major: Option<u32>) -> Result<()> {
    let file = pack_java_file(pack_id)?;
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    match major {
        Some(m) => std::fs::write(&file, m.to_string())?,
        None => {
            if file.exists() {
                std::fs::remove_file(&file)?;
            }
        }
    }
    Ok(())
}

/// Файл с тумблером Discord Rich Presence (по умолчанию включено).
fn discord_rp_file() -> Result<PathBuf> {
    Ok(launcher_root()?.join("discord-rp.txt"))
}

/// Rich Presence включён? (файла нет или содержимое не "0" — включено).
pub fn discord_rp_enabled() -> bool {
    discord_rp_file()
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|s| s.trim() != "0")
        .unwrap_or(true)
}

pub fn set_discord_rp_enabled(on: bool) -> Result<()> {
    let file = discord_rp_file()?;
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&file, if on { "1" } else { "0" })?;
    // Если выключили во время игры — сразу гасим активный статус.
    if !on {
        crate::discord_rp::stop_presence();
    }
    Ok(())
}

/// Файл с тумблером плашки предупреждения о кастомных модах (по умолчанию показано).
fn warn_custom_mods_file() -> Result<PathBuf> {
    Ok(launcher_root()?.join("warn-custom-mods.txt"))
}

/// Плашка предупреждения о кастомных модах показана? (по умолчанию да).
pub fn warn_custom_mods_enabled() -> bool {
    warn_custom_mods_file()
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|s| s.trim() != "0")
        .unwrap_or(true)
}

pub fn set_warn_custom_mods_enabled(on: bool) -> Result<()> {
    let file = warn_custom_mods_file()?;
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&file, if on { "1" } else { "0" })?;
    Ok(())
}
