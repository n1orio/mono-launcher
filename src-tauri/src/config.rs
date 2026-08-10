use std::path::PathBuf;

use anyhow::Result;
use dirs::data_dir;

/// Описание сборки: id используется в путях и командах IPC.
#[derive(Debug, Clone, Copy)]
pub struct PackDef {
    pub id: &'static str,
    pub name: &'static str,
    pub url: &'static str,
}

/// Все поддерживаемые сборки. GitHub-endpoints выводятся из `url`.
pub const PACKS: &[PackDef] = &[PackDef {
    id: "untold-legends",
    name: "Untold Legends",
    url: "https://github.com/n1orio/Untold-legends/releases/latest/download/Untold.legends.mrpack",
}];

pub fn pack_by_id(id: &str) -> Option<&'static PackDef> {
    PACKS.iter().find(|p| p.id == id)
}

pub fn default_pack_id() -> &'static str {
    PACKS[0].id
}

/// Базовая папка всех данных лаунчера.
pub fn launcher_root() -> Result<PathBuf> {
    let base = data_dir()
        .ok_or_else(|| anyhow::anyhow!("Не удалось определить папку данных"))?
        .join("NioLauncher");
    Ok(base)
}

/// Папка конкретной сборки (профили, кэш mrpack, активная версия).
pub fn pack_dir(pack_id: &str) -> Result<PathBuf> {
    Ok(launcher_root()?.join("packs").join(pack_id))
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
    std::fs::write(&path, serde_json::json!({ "versionId": version_id }).to_string())?;
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