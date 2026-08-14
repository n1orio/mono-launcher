use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::config;
use crate::curseforge;
use crate::modrinth;

/// Запись в папке игры: мод/ресурспак/шейдер/мир.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameFileEntry {
    /// Имя с расширением (без суффикса `.disabled`).
    pub name: String,
    /// Человекочитаемое имя без расширения.
    pub display_name: String,
    /// "file" | "dir"
    pub kind: String,
    /// Включён (файл не переименован в *.disabled).
    pub enabled: bool,
    pub size_bytes: u64,
    /// unix-секунды последнего изменения файла (для сортировки «по дате»).
    pub modified: i64,
    /// Точная страница мода на Modrinth (из downloads в .mono-index.json), если файл оттуда.
    pub modrinth_url: Option<String>,
    /// slug проекта Modrinth (из .mono-modrinth.json), если файл установлен вручную с Modrinth.
    pub modrinth_project_id: Option<String>,
    /// ID проекта CurseForge (если файл установлен вручную с CurseForge) — для меты/иконки.
    pub curseforge_project_id: Option<u32>,
    /// Название проекта CurseForge (из трекера) — для показа без API-запроса.
    pub curseforge_title: Option<String>,
    /// URL логотипа проекта CurseForge (из трекера).
    pub curseforge_icon: Option<String>,
}

fn folder_dir(pack_id: &str, folder: &str) -> Result<PathBuf> {
    Ok(config::active_game_dir(pack_id)?.join(folder))
}

/// Карта `относительный путь -> URL первой загрузки` из индекса установленной
/// версии (`.mono-index.json`). Точная ссылка нужна для кнопки «открыть на Modrinth».
fn install_urls(pack_id: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let index_path = config::active_game_dir(pack_id)
        .ok()
        .map(|d| d.join(".mono-index.json"));
    let Some(index_path) = index_path else {
        return map;
    };
    let Ok(raw) = std::fs::read_to_string(&index_path) else {
        return map;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return map;
    };
    let Some(files) = value.get("files").and_then(|f| f.as_array()) else {
        return map;
    };
    for f in files {
        let path = f.get("path").and_then(|p| p.as_str());
        let url = f
            .get("downloads")
            .and_then(|d| d.as_array())
            .and_then(|d| d.first())
            .and_then(|u| u.as_str());
        if let (Some(path), Some(url)) = (path, url) {
            map.entry(path.to_string())
                .or_insert_with(|| url.to_string());
        }
    }
    map
}

/// Из URL загрузки Modrinth CDN (`https://cdn.modrinth.com/data/<proj>/versions/<ver>/...`)
/// строит страницу проекта `https://modrinth.com/mod/<proj>`.
fn modrinth_page_url(download: &str) -> Option<String> {
    let rest = download.strip_prefix("https://cdn.modrinth.com/data/")?;
    let project = rest.split('/').next()?;
    if project.is_empty() {
        return None;
    }
    Some(format!("https://modrinth.com/mod/{project}"))
}

/// Список файлов/папок в папке игры (моды/ресурспаки/шейдеры/миры).
pub fn list_files(pack_id: &str, folder: &str) -> Result<Vec<GameFileEntry>> {
    let dir = folder_dir(pack_id, folder)?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let urls = install_urls(pack_id);
    let curse_meta = curseforge::tracked_meta(pack_id, folder);
    let modrinth_map: HashMap<(String, String), String> = modrinth::tracked_mods(pack_id)
        .into_iter()
        .map(|m| ((m.folder.clone(), m.file_name.clone()), m.project_id))
        .collect();
    let mut out: Vec<GameFileEntry> = Vec::new();
    for e in std::fs::read_dir(&dir)? {
        let Ok(e) = e else { continue };
        let Ok(meta) = e.metadata() else { continue };
        let name = e.file_name().to_string_lossy().to_string();
        let is_dir = meta.is_dir();
        // Скрываем служебные файлы.
        if name.starts_with('.') {
            continue;
        }
        // Для файлов (не папок) — только интересующие форматы.
        let (raw, enabled) = match name.strip_suffix(".disabled") {
            Some(stripped) => (stripped.to_string(), false),
            None => (name.clone(), true),
        };
        let is_file = !is_dir;
        if is_file {
            let ok = match folder {
                "mods" => raw.ends_with(".jar"),
                "resourcepacks" | "shaderpacks" => raw.ends_with(".zip"),
                "saves" => false,
                _ => return Err(anyhow::anyhow!("Неизвестная папка: {folder}")),
            };
            if !ok {
                continue;
            }
        }
        let display_name = raw
            .strip_suffix(".jar")
            .or_else(|| raw.strip_suffix(".zip"))
            .unwrap_or(&raw)
            .to_string();
        let modrinth_url = if is_file {
            urls.get(&format!("{folder}/{raw}"))
                .and_then(|u| modrinth_page_url(u))
        } else {
            None
        };
        let modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let curseforge_project_id = if is_file {
            curse_meta.get(&raw).or_else(|| curse_meta.get(&name)).map(|m| m.0)
        } else {
            None
        };
        let curseforge_title = if is_file {
            curse_meta.get(&raw).or_else(|| curse_meta.get(&name)).and_then(|m| (!m.1.is_empty()).then_some(m.1.clone()))
        } else {
            None
        };
        let curseforge_icon = if is_file {
            curse_meta.get(&raw).or_else(|| curse_meta.get(&name)).and_then(|m| (!m.2.is_empty()).then_some(m.2.clone()))
        } else {
            None
        };
        let modrinth_project_id = if is_file {
            modrinth_map
                .get(&(folder.to_string(), raw.clone()))
                .or_else(|| modrinth_map.get(&(folder.to_string(), name.clone())))
                .cloned()
        } else {
            None
        };
        out.push(GameFileEntry {
            name: raw,
            display_name,
            kind: if is_dir { "dir".into() } else { "file".into() },
            enabled,
            size_bytes: meta.len(),
            modified,
            // Точная страница Modrinth — только для файлов из индекса сборки;
            // у добавленных вручную её нет, фронтенд делает поиск.
            modrinth_url,
            modrinth_project_id,
            // CurseForge-проект (для меты/иконки), если файл установлен вручную с CurseForge.
            curseforge_project_id,
            curseforge_title,
            curseforge_icon,
        });
    }
    // Включённые сверху, дальше по алфавиту.
    out.sort_by(|a, b| {
        b.enabled.cmp(&a.enabled).then_with(|| {
            a.display_name
                .to_lowercase()
                .cmp(&b.display_name.to_lowercase())
        })
    });
    Ok(out)
}

/// Включает/выключает файл: переименование имя.jar <-> имя.jar.disabled.
/// name — базовое имя (без .disabled), enabled — целевое состояние.
pub fn toggle_file(pack_id: &str, folder: &str, name: &str, enabled: bool) -> Result<()> {
    let dir = folder_dir(pack_id, folder)?;
    let cur = if enabled {
        dir.join(format!("{name}.disabled"))
    } else {
        dir.join(name)
    };
    let next = if enabled {
        dir.join(name)
    } else {
        dir.join(format!("{name}.disabled"))
    };
    if !cur.exists() {
        return Err(anyhow::anyhow!("Файл не найден: {}", cur.display()));
    }
    std::fs::rename(&cur, &next)
        .with_context(|| format!("Не удалось переименовать {}", cur.display()))?;
    Ok(())
}

/// Проверяет, что имя — простое имя файла/папки (без путей), чтобы
/// удаление нельзя было увести из папки игры (../etc, подстановки пути).
fn is_safe_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains('/')
        && !name.contains('\\')
        && !name.contains('\0')
        && !name.starts_with('.')
}

/// Удаляет файлы/папки из папки игры по базовым именам (обычно выделенные
/// моды/ресурспаки в UI). Удаляет и парную версию `*.disabled`, если есть.
/// Папки (миры) удаляются рекурсивно. Возвращает число удалённых элементов.
pub fn delete_files(pack_id: &str, folder: &str, names: &[String]) -> Result<usize> {
    let dir = folder_dir(pack_id, folder)?;
    if !dir.exists() {
        return Ok(0);
    }
    let mut removed = 0usize;
    for name in names {
        if !is_safe_name(name) {
            continue;
        }
        for cand in [dir.join(name), dir.join(format!("{name}.disabled"))] {
            let num = if cand.is_dir() {
                std::fs::remove_dir_all(&cand).map(|_| 1)
            } else if cand.is_file() {
                std::fs::remove_file(&cand).map(|_| 1)
            } else {
                continue;
            };
            num.with_context(|| format!("Не удалось удалить {}", cand.display()))?;
            removed += 1;
        }
    }
    Ok(removed)
}

/// Проверяет имя на опасные составляющие (пути, служебные символы).
#[cfg(test)]
fn assert_safe(name: &str, expect: bool) {
    assert_eq!(is_safe_name(name), expect, "имя: {name:?}");
}

#[test]
fn rejects_path_names() {
    assert_safe("mod.jar", true);
    assert_safe("world", true);
    assert_safe("../evil.jar", false);
    assert_safe("a/b.jar", false);
    assert_safe("a\\b.jar", false);
    assert_safe(".hidden", false);
    assert_safe("", false);
    assert_safe("..", false);
    assert_safe(".", false);
}

/// Иконка файла, возвращаемая батч-командой.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameFileIcon {
    pub name: String,
    pub data: Option<String>,
}

/// Иконки для списка файлов одним вызовом (вместо N отдельных IPC).
pub fn file_icons(pack_id: &str, folder: &str, names: &[String]) -> Vec<GameFileIcon> {
    names
        .iter()
        .map(|n| GameFileIcon {
            name: n.clone(),
            data: file_icon(pack_id, folder, n).ok().flatten(),
        })
        .collect()
}

/// Иконка файла/папки как base64 PNG (или None).
/// В zip-архивах ищем icon.png/pack.png; в папках миров — icon.png.
pub fn file_icon(pack_id: &str, folder: &str, name: &str) -> Result<Option<String>> {
    let dir = folder_dir(pack_id, folder)?;
    let path = dir.join(name);
    if path.is_dir() {
        // Прямые иконки в корне папки.
        for cand in ["icon.png", "icon.jpg", "pack.png", "assets/icon.png"] {
            let f = path.join(cand);
            if f.exists() {
                return encode_image(&f);
            }
        }
        // Иначе — любой icon/pack в первых подпапках (шейдеры держат в shaders/).
        if let Some(f) = find_nested_icon(&path) {
            return encode_image(&f);
        }
        return Ok(None);
    }
    if path.exists() || dir.join(format!("{name}.disabled")).exists() {
        return zip_icon(&path);
    }
    Ok(None)
}

/// Ищет иконки в подпапках (глубина ≤ 4), отдаёт самую короткую по пути.
fn find_nested_icon(root: &Path) -> Option<PathBuf> {
    let mut best: Option<(usize, PathBuf)> = None;
    let mut stack = vec![(root.to_path_buf(), 0usize)];
    while let Some((d, lvl)) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                if lvl + 1 < 4 {
                    stack.push((p, lvl + 1));
                }
            } else if let Some(n) = p.file_name().and_then(|n| n.to_str()) {
                let lower = n.to_ascii_lowercase();
                if matches!(lower.as_str(), "icon.png" | "icon.jpg" | "pack.png") {
                    let score = p.components().count();
                    if best.as_ref().map(|(s, _)| score < *s).unwrap_or(true) {
                        best = Some((score, p));
                    }
                }
            }
        }
    }
    best.map(|(_, p)| p)
}

fn encode_image(path: &Path) -> Result<Option<String>> {
    let bytes = std::fs::read(path)?;
    if bytes.len() > 2 * 1024 * 1024 || bytes.is_empty() {
        return Ok(None);
    }
    use base64::Engine;
    let mime = if path.extension().map(|e| e == "jpg").unwrap_or(false) {
        "image/jpeg"
    } else {
        "image/png"
    };
    Ok(Some(format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&bytes)
    )))
}

/// Достаёт иконку (icon.png/pack.png/mod_icon.png) из zip-архива.
fn zip_icon(path: &Path) -> Result<Option<String>> {
    let mut archive = match zip::ZipArchive::new(std::fs::File::open(path)?) {
        Ok(a) => a,
        Err(_) => return Ok(None),
    };
    // Приоритет: корень, потом любые пути (самые короткие).
    let mut names: Vec<(usize, usize, String)> = Vec::new();
    for i in 0..archive.len() {
        let Ok(f) = archive.by_index(i) else { continue };
        let n = f.name().to_string();
        let lower = n.to_ascii_lowercase();
        let base = lower.rsplit('/').next().unwrap_or("");
        if matches!(base, "icon.png" | "pack.png" | "mod_icon.png") {
            names.push((if n.contains('/') { 1 } else { 0 }, n.len(), n));
        }
    }
    names.sort();
    // Ограничение: не разворачивать сотни крупных архива, хватает первых кандидатов.
    for (_, _, name) in names.iter().take(5) {
        let Ok(mut f) = archive.by_name(name) else {
            continue;
        };
        if f.size() <= 2 * 1024 * 1024 && f.size() > 0 {
            let mut buf = Vec::with_capacity(f.size() as usize);
            if f.read_to_end(&mut buf).is_ok() {
                use base64::Engine;
                return Ok(Some(format!(
                    "data:image/png;base64,{}",
                    base64::engine::general_purpose::STANDARD.encode(&buf)
                )));
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_modrinth_cdn_url() {
        assert_eq!(
            modrinth_page_url(
                "https://cdn.modrinth.com/data/abc123/versions/xyz789/file-1.2.3.jar"
            ),
            Some("https://modrinth.com/mod/abc123".to_string())
        );
    }

    #[test]
    fn rejects_non_modrinth_urls() {
        assert_eq!(
            modrinth_page_url("https://mediafilez.forgecdn.net/files/5555/55/f.jar"),
            None
        );
        assert_eq!(modrinth_page_url("не-ссылка"), None);
    }
}
