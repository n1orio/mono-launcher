use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::config;

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
}

fn folder_dir(pack_id: &str, folder: &str) -> Result<PathBuf> {
    Ok(config::active_game_dir(pack_id)?.join(folder))
}

/// Список файлов/папок в папке игры (моды/ресурспаки/шейдеры/миры).
pub fn list_files(pack_id: &str, folder: &str) -> Result<Vec<GameFileEntry>> {
    let dir = folder_dir(pack_id, folder)?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
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
        out.push(GameFileEntry {
            name: raw,
            display_name,
            kind: if is_dir { "dir".into() } else { "file".into() },
            enabled,
            size_bytes: meta.len(),
        });
    }
    // Включённые сверху, дальше по алфавиту.
    out.sort_by(|a, b| {
        b.enabled
            .cmp(&a.enabled)
            .then_with(|| a.display_name.to_lowercase().cmp(&b.display_name.to_lowercase()))
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
    std::fs::rename(&cur, &next).with_context(|| format!("Не удалось переименовать {}", cur.display()))?;
    Ok(())
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
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
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
        let Ok(mut f) = archive.by_name(name) else { continue };
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

