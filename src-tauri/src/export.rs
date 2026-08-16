use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::config;
use crate::mrpack;

/// Верхнеуровневые папки/файлы игры, которые по умолчанию не попадают в экспорт
/// (рантайм и личное). Их всё равно можно включить вручную.
const EXCLUDE_NAMES: &[&str] = &[
    "assets",
    "libraries",
    "versions",
    "runtime",
    "natives",
    "logs",
    "crash-reports",
    "screenshots",
    "saves",
    "cache",
    "options.txt",
    "servers.dat",
    "usercache.json",
    "usernamecache.json",
    "launcher_profiles.json",
    "launcher_accounts.json",
];

/// Элемент дерева папки игры (плоский список с полными относительными путями).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSourceItem {
    /// Относительный путь через `/`, например "mods/example.jar" или "config".
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub default_included: bool,
}

/// Следует ли исключить элемент верхнего уровня по умолчанию.
fn is_excluded(name: &str) -> bool {
    name.starts_with('.') || EXCLUDE_NAMES.contains(&name)
}

fn dir_size(path: &Path) -> u64 {
    let mut total = 0u64;
    for entry in fs::read_dir(path).into_iter().flatten() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            total += dir_size(&path);
        } else {
            total += entry.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }
    total
}

fn resolve_version(pack_id: &str, version_id: &str) -> Result<String> {
    if !version_id.is_empty() {
        return Ok(version_id.to_string());
    }
    let active = config::active_version(pack_id)?;
    if active.is_empty() {
        anyhow::bail!("Нет активной версии сборки");
    }
    Ok(active)
}

fn walk(
    game_dir: &Path,
    base: &str,
    excluded_root: bool,
    out: &mut Vec<ExportSourceItem>,
) -> Result<()> {
    for entry in fs::read_dir(game_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let rel = if base.is_empty() {
            name.clone()
        } else {
            format!("{base}/{name}")
        };
        // Исключение действует только на элементы верхнего уровня.
        let this_root_excluded = excluded_root || is_excluded(&name);
        let is_dir = entry.file_type()?.is_dir();
        let size = if is_dir {
            dir_size(&entry.path())
        } else {
            entry.metadata().map(|m| m.len()).unwrap_or(0)
        };
        out.push(ExportSourceItem {
            default_included: !this_root_excluded,
            size,
            is_dir,
            path: rel.clone(),
        });
        if is_dir {
            walk(&entry.path(), &rel, this_root_excluded, out)?;
        }
    }
    Ok(())
}

/// Полный плоский список папок и файлов папки игры для выбора перед экспортом.
pub fn list_sources(pack_id: &str, version_id: &str) -> Result<Vec<ExportSourceItem>> {
    let version = resolve_version(pack_id, version_id)?;
    let game_dir = config::version_dir(pack_id, &version)?;
    let mut items = Vec::new();
    walk(&game_dir, "", false, &mut items)?;
    // Папки первыми, затем по алфавиту.
    items.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.path.cmp(&b.path)));
    Ok(items)
}

/// Копирует выбранные пути (папки рекурсивно, файлы по одному) в `overrides_dir`.
/// Если `include` пуст — копируются все не исключённые по умолчанию.
fn copy_selected(game_dir: &Path, overrides_dir: &Path, include: &[String]) -> Result<()> {
    fs::create_dir_all(overrides_dir)?;
    let selected: HashSet<&str> = include.iter().map(|s| s.as_str()).collect();
    let default_all = include.is_empty();

    for entry in fs::read_dir(game_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let is_dir = entry.file_type()?.is_dir();
        let default_excluded = is_excluded(&name);

        // Собираем выбранные элементы этого каталога рекурсивно сам по себе не надо —
        // здесь обрабатываем верхний уровень и рекурсивно все, что под ним.
        copy_node(
            &entry.path(),
            overrides_dir,
            &name,
            is_dir,
            if default_all {
                default_excluded
            } else {
                !selected.contains(name.as_str())
            },
            default_all,
            &selected,
        )?;
    }
    Ok(())
}

/// Рекурсивно копирует один узел. Параметр `skip` включает рекурсивную фильтрацию
/// (когда пользователь выбрал только часть содержимого каталога).
fn copy_node(
    src: &Path,
    overrides_dir: &Path,
    rel: &str,
    is_dir: bool,
    skip: bool,
    default_all: bool,
    selected: &HashSet<&str>,
) -> Result<()> {
    let dest = overrides_dir.join(rel);
    if is_dir {
        if skip {
            // Копируем только выбранное содержимое каталога.
            if default_all {
                return Ok(());
            }
            for child in fs::read_dir(src)? {
                let child = child?;
                let cname = child.file_name().to_string_lossy().into_owned();
                let crel = format!("{rel}/{cname}");
                let cis_dir = child.file_type()?.is_dir();
                copy_node(
                    &child.path(),
                    overrides_dir,
                    &crel,
                    cis_dir,
                    !selected.contains(crel.as_str()),
                    default_all,
                    selected,
                )?;
            }
        } else {
            fs::create_dir_all(&dest)?;
            copy_tree(src, &dest)?;
        }
    } else if !skip {
        fs::create_dir_all(dest.parent().unwrap_or(overrides_dir))?;
        fs::copy(src, &dest)?;
    }
    Ok(())
}

/// Рекурсивно копирует каталог целиком.
fn copy_tree(src: &Path, dst: &Path) -> Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let target = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            fs::create_dir_all(&target)?;
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// Упаковывает папку `tmp` в zip-архив по пути `dest`.
pub(crate) fn zip_dir(tmp: &Path, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(dest).context("Не удалось создать файл экспорта")?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    fn add_tree(
        zip: &mut zip::ZipWriter<File>,
        base: &Path,
        dir: &Path,
        opts: &zip::write::SimpleFileOptions,
    ) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let rel = path
                .strip_prefix(base)?
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            if entry.file_type()?.is_dir() {
                zip.start_file(format!("{rel}/"), *opts)?;
                add_tree(zip, base, &path, opts)?;
            } else {
                zip.start_file(rel, *opts)?;
                let mut f = File::open(&path)?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)?;
                zip.write_all(&buf)?;
            }
        }
        Ok(())
    }

    add_tree(&mut zip, tmp, tmp, &opts)?;
    zip.finish()?;
    Ok(())
}

pub(crate) fn temp_dir(pack_id: &str) -> Result<PathBuf> {
    let name = format!("mono-export-{pack_id}-{}", std::process::id());
    let dir = std::env::temp_dir().join(name);
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Экспорт сборки в формат `.mrpack` (Modrinth).
pub fn export_mrpack(
    pack_id: &str,
    version_id: &str,
    dest: &Path,
    include: &[String],
    name: &str,
    export_version: &str,
) -> Result<()> {
    let version = resolve_version(pack_id, version_id)?;
    let game_dir = config::version_dir(pack_id, &version)?;
    let index = mrpack::read_version_index(pack_id, &version)
        .ok_or_else(|| anyhow::anyhow!("Индекс сборки не найден"))?;

    let tmp = temp_dir(pack_id)?;
    let result = (|| -> Result<()> {
        let index_json = serde_json::json!({
            "formatVersion": 1,
            "game": "minecraft",
            "versionId": if export_version.is_empty() { index.version_id } else { export_version.to_string() },
            "name": if name.is_empty() { index.name } else { name.to_string() },
            "summary": index.summary,
            "files": [],
            "dependencies": index.dependencies,
        });
        fs::write(
            tmp.join("modrinth.index.json"),
            serde_json::to_string_pretty(&index_json)?,
        )?;

        copy_selected(&game_dir, &tmp.join("overrides"), include)?;
        zip_dir(&tmp, dest)
    })();
    let _ = fs::remove_dir_all(&tmp);
    result
}

/// Экспорт сборки в `.zip` для CurseForge (manifest.json + overrides).
pub fn export_curseforge(
    pack_id: &str,
    version_id: &str,
    dest: &Path,
    include: &[String],
    name: &str,
    export_version: &str,
) -> Result<()> {
    let version = resolve_version(pack_id, version_id)?;
    let game_dir = config::version_dir(pack_id, &version)?;
    let index = mrpack::read_version_index(pack_id, &version)
        .ok_or_else(|| anyhow::anyhow!("Индекс сборки не найден"))?;

    let minecraft = index.dependencies.get("minecraft").cloned().unwrap_or_default();
    let (loader, loader_version) = index
        .dependencies
        .iter()
        .find(|(k, _)| k.ends_with("-loader"))
        .map(|(k, v)| (k.trim_end_matches("-loader").to_string(), v.clone()))
        .unwrap_or_default();

    let tmp = temp_dir(pack_id)?;
    let result = (|| -> Result<()> {
        let manifest = serde_json::json!({
            "minecraft": {
                "version": minecraft,
                "modLoaders": [{
                    "id": format!("{loader}-{loader_version}"),
                    "primary": true
                }]
            },
            "manifestType": "minecraftModpack",
            "manifestVersion": 1,
            "name": if name.is_empty() { index.name } else { name.to_string() },
            "version": if export_version.is_empty() { index.version_id } else { export_version.to_string() },
            "author": "",
            "files": [],
            "overrides": "overrides"
        });
        fs::write(tmp.join("manifest.json"), serde_json::to_string_pretty(&manifest)?)?;

        copy_selected(&game_dir, &tmp.join("overrides"), include)?;
        zip_dir(&tmp, dest)
    })();
    let _ = fs::remove_dir_all(&tmp);
    result
}

/// Команда списка папок/файлов (дерево, плоский список) для выбора перед экспортом.
#[tauri::command]
pub fn export_list_command(pack_id: String, version_id: String) -> Result<Vec<ExportSourceItem>, String> {
    list_sources(&pack_id, &version_id).map_err(|e| e.to_string())
}

/// Команда экспорта сборки (mrpack / curseforge).
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn export_pack_command(
    pack_id: String,
    version_id: String,
    format: String,
    dest_path: String,
    include: Vec<String>,
    name: String,
    version: String,
) -> Result<(), String> {
    let dest = PathBuf::from(&dest_path);
    if format != "mrpack" && format != "curseforge" {
        return Err(format!("Неизвестный формат экспорта: {format}"));
    }
    let res: Result<()> = if format == "mrpack" {
        export_mrpack(&pack_id, &version_id, &dest, &include, &name, &version)
    } else {
        export_curseforge(&pack_id, &version_id, &dest, &include, &name, &version)
    };
    res.map_err(|e| e.to_string())
}