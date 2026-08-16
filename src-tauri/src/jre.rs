use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use futures::StreamExt;
use reqwest::Client;
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;

use crate::config;

fn java_exe_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "java.exe"
    } else {
        "java"
    }
}

/// Возвращает (майор, минор) из имени папки вроде jdk-21.0.2 / 21.0.2 / jdk1.8.0_402:
/// берёт первый версионный токен (хвост вроде _402 игнорируется), legacy 1.x считает как x.
fn java_version_from_name(dir_name: &str) -> Option<(u32, u32)> {
    let name = dir_name.to_ascii_lowercase();
    let bytes = name.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                i += 1;
            }
            let token = &name[start..i];
            let parts: Vec<&str> = token.split('.').collect();
            if parts.is_empty() || parts[0].is_empty() {
                continue;
            }
            let (major, minor) = if parts[0] == "1" {
                (
                    parts
                        .get(1)
                        .and_then(|p| p.parse::<u32>().ok())
                        .unwrap_or(1),
                    parts
                        .get(2)
                        .and_then(|p| p.parse::<u32>().ok())
                        .unwrap_or(0),
                )
            } else {
                (
                    parts[0].parse::<u32>().unwrap_or(1),
                    parts
                        .get(1)
                        .and_then(|p| p.parse::<u32>().ok())
                        .unwrap_or(0),
                )
            };
            return Some((major, minor));
        }
        i += 1;
    }
    None
}

/// Кандидаты в установленную Java (папки с bin/java), без проверки работоспособности.
/// Сканирует распространённые места установки: JAVA_HOME и штатные каталоги JDK.
pub fn java_candidates() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Ok(home) = std::env::var("JAVA_HOME") {
        if !home.is_empty() {
            roots.push(PathBuf::from(home).join("bin").join(java_exe_name()));
        }
    }
    #[cfg(target_os = "windows")]
    {
        for var in ["PROGRAMFILES", "PROGRAMFILES(X86)", "LOCALAPPDATA"] {
            if let Ok(base) = std::env::var(var) {
                let base = PathBuf::from(&base);
                roots.push(base.join("Programs"));
                roots.push(base);
            }
        }
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        roots.extend([
            PathBuf::from("/usr/lib/jvm"),
            PathBuf::from("/usr/lib64/jvm"),
            PathBuf::from("/opt"),
        ]);
    }
    #[cfg(target_os = "macos")]
    {
        roots.push(PathBuf::from("/Library/Java/JavaVirtualMachines"));
        if let Ok(home) = std::process::Command::new("/usr/libexec/java_home")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        {
            if !home.is_empty() {
                roots.push(PathBuf::from(home).join("bin").join(java_exe_name()));
            }
        }
    }

    let mut found: Vec<PathBuf> = Vec::new();
    let bat = java_exe_name();
    for entry in roots.clone() {
        // Уже конкретный бинарь (JAVA_HOME / java_home) — не сканируем, а добавляем как есть.
        if entry.file_name().map(|n| n == bat).unwrap_or(false) {
            if entry.exists() {
                found.push(entry);
            }
            continue;
        }
        // Один уровень вложенности ниже корня: Java обычно лежит в <root>/<vendor>/<jdk>/bin
        let Ok(dirs) = std::fs::read_dir(&entry) else {
            continue;
        };
        for dir in dirs.flatten() {
            let jdk_dir = dir.path();
            let bin = jdk_dir.join("bin").join(bat);
            if bin.exists() {
                found.push(bin);
                continue;
            }
            let Ok(subs) = std::fs::read_dir(&jdk_dir) else {
                continue;
            };
            for sub in subs.flatten() {
                let sub_bin = sub.path().join("bin").join(bat);
                if sub_bin.exists() {
                    found.push(sub_bin);
                }
            }
        }
    }

    found.sort_by(|a, b| {
        let ka = a
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .and_then(|n| java_version_from_name(&n))
            .unwrap_or((1, 0));
        let kb = b
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .and_then(|n| java_version_from_name(&n))
            .unwrap_or((1, 0));
        kb.cmp(&ka)
    });

    let mut seen = std::collections::HashSet::new();
    found.retain(|p| seen.insert(p.to_string_lossy().to_string()));
    found
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JavaArch {
    Bit64,
    Bit32,
    Unknown,
}

/// Проверяет, что java реально запускается (`-version`), и определяет разрядность
/// (64-битные VM пишут в вывод «64-Bit»). Не работает — вернёт None.
pub fn probe_java(java: &Path) -> Option<JavaArch> {
    let out = std::process::Command::new(java)
        .arg("-version")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    let text = text.to_lowercase();
    if text.contains("64-bit") {
        Some(JavaArch::Bit64)
    } else if text.contains("hotspot") || text.contains("openjdk") || text.contains("jvm") {
        // Запустилась, но не 64-битная — это 32-битная VM (у неё потолок кучи ~3-4G).
        Some(JavaArch::Bit32)
    } else {
        Some(JavaArch::Unknown)
    }
}

/// Версия java в человекочитаемом виде (например «21.0.11») по выводу `-version`.
pub fn java_version_string(java: &Path) -> Option<String> {
    let out = std::process::Command::new(java)
        .arg("-version")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stderr);
    let line = text.lines().next()?;
    // openjdk version "21.0.11" 2026-04-21 / java version "1.8.0_402" ...
    let quote = line.split('"').nth(1)?;
    if let Some(stripped) = quote.strip_prefix("1.") {
        Some(stripped.split('.').next().unwrap_or(stripped).to_string())
    } else {
        Some(quote.split('.').take(2).collect::<Vec<_>>().join("."))
    }
}

/// Мажорный номер версии Java (21, 17, 8...) по пути к бинарю.
pub fn java_major(path: &Path) -> Option<u32> {
    let v = java_version_string(path)?;
    let s = if let Some(stripped) = v.strip_prefix("1.") {
        stripped
    } else {
        &v
    };
    s.split('.').next()?.parse().ok()
}

/// Требуемая мажорная версия Java для версии Minecraft.
/// 1.21+ / 1.20.5+ → 21; 1.17–1.20.4 → 17; до 1.17 → 8.
pub fn required_java(minecraft_version: &str) -> Option<u32> {
    let mut it = minecraft_version.split('.');
    let major: u32 = it.next()?.parse().ok()?;
    let minor: u32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch: u32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    if major > 1 {
        return Some(21);
    }
    if minor >= 21 {
        return Some(21);
    }
    if minor == 20 && patch >= 5 {
        return Some(21);
    }
    if minor >= 17 {
        return Some(17);
    }
    Some(8)
}

/// Описание найденной Java для UI.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaInfo {
    pub path: String,
    pub label: String,
    pub version: String,
    pub arch: String,
    pub is_bundled: bool,
    pub selected: bool,
}

/// Вся найденная Java (встроенная + установленная + из PATH) для UI.
pub fn list_javas() -> Vec<JavaInfo> {
    let selected = config::java_selection();
    let mut out = Vec::new();

    let mut push = |path: PathBuf, bundled: bool, is_path_cmd: bool| {
        let arch = probe_java(&path);
        let path_str = path.to_string_lossy().to_string();
        let name = if is_path_cmd {
            "Из PATH".to_string()
        } else {
            path.parent()
                .and_then(|p| p.parent())
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                .unwrap_or_else(|| path_str.clone())
        };
        out.push(JavaInfo {
            is_bundled: bundled,
            label: if bundled {
                format!("Встроенная ({name})")
            } else {
                name
            },
            version: java_version_string(&path).unwrap_or_else(|| "?".into()),
            arch: match arch {
                Some(JavaArch::Bit64) => "64-бит".into(),
                Some(JavaArch::Bit32) => "32-бит".into(),
                _ => "недоступна".into(),
            },
            selected: false,
            path: path_str,
        });
    };

    for b in bundled_javas() {
        if probe_java(&b).is_some() {
            push(b, true, false);
        }
    }
    for cand in java_candidates() {
        if probe_java(&cand).is_some() {
            push(cand, false, false);
        }
    }
    // Команда java из PATH — всегда доступна, если она работает.
    let path_java = PathBuf::from("java");
    if probe_java(&path_java).is_some() {
        push(path_java, false, true);
    }

    // Текущий выбор из настроек помечаем полем `selected`.
    let sel = selected
        .as_ref()
        .map(|s| s.trim().trim_end_matches('/').trim_end_matches('\\'))
        .unwrap_or("");
    for j in out.iter_mut() {
        let path = j.path.trim().trim_end_matches('/').trim_end_matches('\\');
        j.selected = !sel.is_empty() && path == sel;
    }
    out
}

/// Ищет Java: выбранную пользователем → встроенную в лаунчер →
/// авто-детект установленных → из PATH («java»).
/// Приоритет: сначала рабочие 64-битные (свежие по версии), затем 32-битные.
/// Если задан `preferred_major`, среди 64-битных предпочитается Java с
/// версией == `preferred_major`, иначе — не младше требуемой.
/// Возвращает путь и разрядность (для ограничения -Xmx).
pub fn find_java(preferred_major: Option<u32>) -> Result<(String, JavaArch)> {
    // 1. Явный выбор пользователя (файл java.txt в папке данных лаунчера).
    if let Some(selected) = config::java_selection() {
        let p = PathBuf::from(&selected);
        if let Some(arch) = probe_java(&p) {
            return Ok((selected, arch));
        }
    }

    let mut cands: Vec<(String, JavaArch, Option<u32>)> = Vec::new();
    for b in bundled_javas() {
        if let Some(arch) = probe_java(&b) {
            cands.push((b.to_string_lossy().to_string(), arch, java_major(&b)));
        }
    }
    for cand in java_candidates() {
        if let Some(arch) = probe_java(&cand) {
            cands.push((cand.to_string_lossy().to_string(), arch, java_major(&cand)));
        }
    }

    // pick(true) — по 64-битным, pick(false) — по 32-битным.
    let pick = |want64: bool| -> Option<(String, JavaArch)> {
        let bucket: Vec<_> = cands
            .iter()
            .filter(|c| (c.1 == JavaArch::Bit64) == want64)
            .collect();
        if let Some(pref) = preferred_major {
            if let Some(c) = bucket.iter().find(|c| c.2 == Some(pref)) {
                return Some((c.0.clone(), c.1));
            }
            if let Some(c) = bucket.iter().find(|c| c.2.map(|m| m >= pref).unwrap_or(false)) {
                return Some((c.0.clone(), c.1));
            }
        }
        bucket.first().map(|c| (c.0.clone(), c.1))
    };

    if let Some(r) = pick(true) {
        return Ok(r);
    }
    if let Some(r) = pick(false) {
        return Ok(r);
    }
    if let Some(arch) = probe_java(Path::new("java")) {
        return Ok(("java".into(), arch));
    }
    Ok(("java".into(), JavaArch::Unknown))
}

/// Папка, куда ставится автоматически скачанная Java нужного мажора.
pub fn jre_root_for(major: u32) -> PathBuf {
    config::java_root()
        .map(|r| r.join(format!("java-{major}")))
        .unwrap_or_else(|_| PathBuf::from(format!("java-{major}")))
}

/// Путь к java-бинарю автоматически скачанной Java нужного мажора.
pub fn java_path_for(major: u32) -> PathBuf {
    jre_root_for(major).join("bin").join(java_exe_name())
}

/// Все автоматически скачанные Java (каждый мажор в runtime/java-*) + устаревшая
/// одиночная в runtime/bin (JRE 21 от старых версий лаунчера).
pub fn bundled_javas() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(root) = config::java_root() {
        let legacy = root.join("bin").join(java_exe_name());
        if legacy.exists() {
            out.push(legacy);
        }
        if let Ok(entries) = std::fs::read_dir(&root) {
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                if let Some(major) = name.strip_prefix("java-") {
                    if let Ok(m) = major.parse::<u32>() {
                        let p = java_path_for(m);
                        if p.exists() {
                            out.push(p);
                        }
                    }
                }
            }
        }
    }
    out
}

/// Скачивает и распаковывает Java (Adoptium JRE) нужного мажора в папку лаунчера.
/// Возвращает путь к java-бинарию.
pub async fn ensure_java(app: &AppHandle, client: &Client, major: u32) -> Result<String> {
    let root = jre_root_for(major);
    let target = root.join("bin").join(java_exe_name());
    if target.exists() {
        if let Some(arch) = probe_java(&target) {
            if arch == JavaArch::Bit64 {
                return Ok(target.to_string_lossy().to_string());
            }
        }
    }
    std::fs::create_dir_all(&root)?;

    let (os, arch) = (
        if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "mac"
        } else {
            "linux"
        },
        if std::env::consts::ARCH == "aarch64" {
            "aarch64"
        } else {
            "x64"
        },
    );
    let url = format!(
        "https://api.adoptium.net/v3/binary/latest/{major}/ga/{os}/{arch}/jre/hotspot/normal/eclipse"
    );
    let _ = app.emit(
        "launch-log",
        crate::game::LogLine {
            stream: "sys".into(),
            line: format!("Скачивание Java {major} (~60-100 МБ): {url}"),
        },
    );

    let resp = client
        .get(&url)
        .send()
        .await
        .context("Не удалось скачать Java с api.adoptium.net")?
        .error_for_status()
        .context("Adoptium не отдал Java")?;
    let total = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();
    let archive_path = root.join("jre-download.bin");
    let mut file = tokio::fs::File::create(&archive_path).await?;
    let mut last_report = std::time::Instant::now();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Ошибка чтения архива Java")?;
        downloaded += chunk.len() as u64;
        file.write_all(&chunk).await?;
        if last_report.elapsed().as_millis() >= 200 {
            let pct = if total > 0 {
                format!("{}%", downloaded.checked_mul(100).map(|v| v / total).unwrap_or(0))
            } else {
                format!("{:.1} МБ", downloaded as f64 / 1048576.0)
            };
            let _ = app.emit(
                "launch-log",
                crate::game::LogLine {
                    stream: "sys".into(),
                    line: format!("Java {major}: скачано {pct}"),
                },
            );
            last_report = std::time::Instant::now();
        }
    }
    file.flush().await?;

    // Распаковка: zip (Windows) или tar.gz (Linux/macOS) — определяем по magic.
    let tmp = root.join(format!("jre-tmp-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&tmp)?;
    let head = {
        use std::io::Read;
        let f = std::fs::File::open(&archive_path)?;
        let mut buf = [0u8; 4];
        std::io::Read::take(std::io::BufReader::new(f), 4)
            .read_exact(&mut buf)
            .ok();
        buf
    };
    let result = if head.starts_with(b"PK") {
        extract_zip(&archive_path, &tmp)
    } else {
        extract_targz(&archive_path, &tmp)
    };
    result.context("Не удалось распаковать Java")?;

    // Переносим содержимое единственной верхнеуровневой папки в root.
    let mut inner_dirs: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(&tmp)? {
        inner_dirs.push(entry?.path());
    }
    std::fs::create_dir_all(&root)?;
    for src in inner_dirs {
        let name = src
            .file_name()
            .ok_or_else(|| anyhow!("Кривой путь в архиве Java"))?;
        let dst = root.join(name);
        if dst.exists() {
            std::fs::remove_dir_all(&dst).ok();
        }
        std::fs::rename(&src, &dst)?;
    }
    std::fs::remove_dir_all(&tmp).ok();
    std::fs::remove_file(&archive_path).ok();

    if target.exists() {
        Ok(target.to_string_lossy().to_string())
    } else {
        Err(anyhow!(
            "Java распакована, но бинарь не найден в {}",
            target.display()
        ))
    }
}

fn extract_zip(path: &Path, tmp: &Path) -> Result<()> {
    use std::io::Read;
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if entry.is_dir() {
            continue;
        }
        let name = entry
            .enclosed_name()
            .ok_or_else(|| anyhow!("Некорректное имя в архиве Java"))?
            .to_path_buf();
        // Снимаем верхнеуровневый каталог (jdk-21.0.x+7-jre/).
        let rel: PathBuf = name.components().skip(1).collect();
        let out = tmp.join(rel);
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        std::fs::write(&out, &buf)?;
    }
    Ok(())
}

fn extract_targz(path: &Path, tmp: &Path) -> Result<()> {
    use std::io::Read;
    let file = std::fs::File::open(path)?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);
    for entry in archive.entries()? {
        let mut entry = entry?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let name = entry.path()?.to_path_buf();
        // Пропускаем корневую папку архива, остальные сегменты проверяем на
        // обход каталогов (.., absolute), иначе тар может писать вне tmp.
        let mut comps = name.components();
        if comps.next().is_none() {
            continue;
        }
        let mut rel = PathBuf::new();
        for c in comps {
            match c {
                std::path::Component::Normal(seg) => rel.push(seg),
                std::path::Component::CurDir => {}
                _ => return Err(anyhow!("Недопустимый путь в архиве Java: {name:?}")),
            }
        }
        if rel.as_os_str().is_empty() {
            continue;
        }
        let out = tmp.join(&rel);
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        std::fs::write(&out, &buf)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::java_version_from_name;

    #[test]
    fn parses_jdk_dir_names() {
        assert_eq!(java_version_from_name("jdk-21.0.2"), Some((21, 0)));
        assert_eq!(java_version_from_name("21.0.5"), Some((21, 0)));
        assert_eq!(java_version_from_name("jdk1.8.0_402"), Some((8, 0)));
        assert_eq!(java_version_from_name("zulu-17.52"), Some((17, 52)));
        assert_eq!(java_version_from_name("temurin-11.0.23"), Some((11, 0)));
        assert_eq!(java_version_from_name("java-17-openjdk"), Some((17, 0)));
        assert_eq!(java_version_from_name("jre-1.8"), Some((8, 0)));
        assert_eq!(java_version_from_name("no-java-here"), None);
    }
}
