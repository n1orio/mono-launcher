use std::path::PathBuf;

use anyhow::Result;
use dirs::data_dir;
use serde::{Deserialize, Serialize};

/// Описание сборки: id используется в путях и командах IPC.
#[derive(Debug, Clone, Copy)]
pub struct PackDef {
    pub id: &'static str,
    pub name: &'static str,
    pub url: &'static str,
    /// Ник блога на Boosty: задан → сборка платная (подписка обязательна).
    pub boosty_blog: Option<&'static str>,
    /// Минимальная оперативка (МБ) для запуска сборки: задан → лаунчер
    /// предупреждает и не даёт запустить при меньшем выделении.
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

/// Все поддерживаемые сборки. GitHub-endpoints выводятся из `url`.
/// `boosty_blog` — ник блога издателя на Boosty: укажите свой, чтобы сборка
/// стала платной (подписка на блог обязательна для установки/запуска).
/// `min_ram_mb` — минимальная оперативка (МБ) для запуска сборки.
pub const PACKS: &[PackDef] = &[PackDef {
    id: "untold-legends",
    name: "Untold Legends",
    url: "https://github.com/n1orio/Untold-legends/releases/latest/download/Untold.legends.mrpack",
    boosty_blog: None,
    min_ram_mb: None,
}];

pub fn pack_by_id(id: &str) -> Option<&'static PackDef> {
    PACKS.iter().find(|p| p.id == id)
}

pub fn default_pack_id() -> &'static str {
    PACKS[0].id
}

/// Встроенные сборки как общий тип `PackInfo`.
pub fn builtin_packs() -> Vec<PackInfo> {
    PACKS
        .iter()
        .map(|p| PackInfo {
            id: p.id.to_string(),
            name: p.name.to_string(),
            url: p.url.to_string(),
            builtin: true,
            kind: "remote".into(),
            boosty_blog: p.boosty_blog.map(String::from),
            min_ram_mb: p.min_ram_mb,
            icon: pack_icon_path(p.id),
            banner: pack_banner_path(p.id),
        })
        .collect()
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
    let list = serde_json::from_str(&raw).unwrap_or_default();
    Ok(list)
}

fn save_user_packs(list: &[UserPack]) -> Result<()> {
    let file = user_packs_file()?;
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&file, serde_json::to_string_pretty(list)?)?;
    Ok(())
}

/// Все сборки: встроенные + пользовательские.
pub fn all_packs() -> Result<Vec<PackInfo>> {
    let mut out = builtin_packs();
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
    if let Some(p) = pack_by_id(id) {
        return Ok(Some(PackInfo {
            id: p.id.to_string(),
            name: p.name.to_string(),
            url: p.url.to_string(),
            builtin: true,
            kind: "remote".into(),
            boosty_blog: p.boosty_blog.map(String::from),
            min_ram_mb: p.min_ram_mb,
            icon: pack_icon_path(p.id),
            banner: pack_banner_path(p.id),
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
