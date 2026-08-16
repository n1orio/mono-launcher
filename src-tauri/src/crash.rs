//! Анализ краш-репортов и логов запуска.
//!
//! После выхода игры со сбоем ищем свежие артефакты (crash-reports/*.txt,
//! hs_err_pid*.log, хвост logs/latest.log), разбираем исключение и стектрейс,
//! классифицируем известные причины и пытаемся найти «подозреваемый» мод по
//! пакетам из стека (методом CrashReportAnalyser из MinecraftForge): пакет из
//! фрейма матчится с записями внутри установленных .jar.

use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::config;

/// Подозреваемый мод: найден по совпадению пакета из стектрейса и jar-файла.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuspectedMod {
    pub name: String,
    pub file: String,
    pub package: String,
}

/// Результат анализа. Пустой (has_crash=false), если ничего не нашлось.
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CrashAnalysis {
    pub has_crash: bool,
    /// Анализируемый файл (относительно корня игры): напр. `crash-reports/crash-2026-...txt`.
    pub file: String,
    /// Ключ причины — фронтенд локализует заголовок и совет:
    /// `oom`, `javaVersion`, `gpu`, `modConflict`, `corrupt`, `mod`, `other`.
    pub kind: String,
    /// Текст первого исключения.
    pub exception: String,
    /// Человеческое `Description:` из начала краш-репорта.
    pub description: String,
    /// Рекомендуемая версия Java (если дело в ней).
    pub java_hint: Option<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub suspected: Vec<SuspectedMod>,
}

/// Пакеты фреймворка/самого Minecraft — их не считаем «подозреваемыми».
const IGNORED_PREFIXES: &[&str] = &[
    "java.",
    "javax.",
    "jdk.",
    "sun.",
    "com.sun.",
    "net.minecraft.",
    "net.minecraftforge.",
    "net.neoforged.",
    "net.fabricmc.",
    "com.mojang.",
    "org.spongepowered.",
    "org.objectweb.",
    "org.apache.",
    "org.slf4j.",
    "org.jetbrains.",
    "org.yaml.",
    "org.lwjgl.",
    "org.lwjglx.",
    "io.netty.",
    "io.github.spair.",
    "kotlin.",
    "scala.",
    "groovy.",
    "com.google.",
    "org.joml.",
    "ca.weblite.",
];

/// Режим поиска — что анализируем.
enum Source {
    CrashReport(PathBuf),
    HsErr(PathBuf),
    LatestLog(PathBuf),
    None,
}

/// Ищет самый свежий краш-артефакт для сборки.
fn newest_source(game_dir: &Path) -> Source {
    // 1. crash-reports/crash-*.txt
    let reports_dir = game_dir.join("crash-reports");
    if let Ok(rd) = std::fs::read_dir(&reports_dir) {
        let mut best: Option<PathBuf> = None;
        let mut best_m = 0i64;
        for e in rd.flatten() {
            let p = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            if !name.starts_with("crash-") {
                continue;
            }
            let m = e
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            if m >= best_m {
                best_m = m;
                best = Some(p);
            }
        }
        if let Some(p) = best {
            return Source::CrashReport(p);
        }
    }
    // 2. hs_err_pid*.log в корне игры.
    if let Ok(rd) = std::fs::read_dir(game_dir) {
        for e in rd.flatten() {
            let p = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with("hs_err_pid") && name.ends_with(".log") && p.is_file() {
                return Source::HsErr(p);
            }
        }
    }
    // 3. хвост logs/latest.log.
    let latest = game_dir.join("logs").join("latest.log");
    if latest.is_file() {
        return Source::LatestLog(latest);
    }
    Source::None
}

fn read_tail(path: &Path, max_bytes: usize) -> String {
    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(_) => return String::new(),
    };
    let start = data.len().saturating_sub(max_bytes);
    String::from_utf8_lossy(&data[start..]).into_owned()
}

/// Класс (FQCN) из строки фрейма вида `\tat com.example.mod.Class.method(Class.java:1)`.
/// Возвращает `None` для строк, что не похожи на фрейм.
fn frame_class(line: &str) -> Option<String> {
    let t = line.trim_start();
    let t = t.strip_prefix("at ")?;
    let method = t.split_once('(').map(|(h, _)| h).unwrap_or(t);
    let method = method.trim();
    let method = method.trim_end_matches('.');
    if method.ends_with(')') {
        return None;
    }
    // class.method — достаточно двух сегментов и всех букв/цифр/_/$/.
    if !method.is_empty()
        && method.split('.').count() >= 2
        && method
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ".$_".contains(c))
    {
        Some(method.to_string())
    } else {
        None
    }
}

/// Строка вида `com.example.SomeException: message` или `...Caused by: ...`.
fn looks_like_exception(line: &str) -> Option<String> {
    let t = line.trim_start().trim_end();
    if t.is_empty() {
        return None;
    }
    // Убираем маркеры фреймов/`Caused by:`.
    let t = t
        .trim_start_matches("Caused by:")
        .trim();
    // Класс (несколько сегментов) + двоеточие или конец.
    let head = match t.split_once(':') {
        Some((h, _)) => h.trim(),
        None => t,
    };
    let head = head.trim_end_matches("Exception").trim_end();
    if head.split('.').count() < 2 {
        return None;
    }
    if !head
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || ".$_<>".contains(c))
    {
        return None;
    }
    if t.contains("Exception")
        || t.contains("Error:")
        || t.contains("Throwable")
        || t.ends_with("Error")
    {
        Some(t.to_string())
    } else {
        None
    }
}

/// Извлекает строки «Description: <текст>» в начале краш-репорта (первую).
fn first_description(text: &str) -> String {
    for line in text.lines() {
        let t = line.trim();
        if let Some(v) = t.strip_prefix("Description:") {
            let v = v.trim();
            if !v.is_empty() {
                return v.to_string();
            }
        }
    }
    String::new()
}

/// Кандидаты-пакеты из стектрейса (базовые, по 2 сегмента), многослойно дедуплицируя.
fn extract_packages(text: &str) -> Vec<String> {
    let mut pkgs: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for line in text.lines() {
        let Some(class) = frame_class(line) else {
            continue;
        };
        let mut parts: Vec<&str> = class.split('.').collect();
        if parts.len() >= 2 {
            // Убираем имя метода (последний сегмент).
            parts.pop();
        }
        if parts.len() < 2 {
            continue;
        }
        // Базовый пакет — первые 2 сегмента (напр. `com.example`).
        let base = parts[..2.min(parts.len())].join(".");
        let ignored = IGNORED_PREFIXES.iter().any(|p| {
            let core = p.trim_end_matches('.');
            base == core || base.starts_with(p)
        });
        if ignored {
            continue;
        }
        if seen.insert(base.clone()) {
            pkgs.push(base);
        }
        if pkgs.len() >= 40 {
            break;
        }
    }
    pkgs
}

/// Открывает jar и возвращает имя записи, если она лежит под одним из кандидат-пакетов.
fn jar_first_match(jar_path: &Path, candidates: &[String]) -> Option<String> {
    let f = File::open(jar_path).ok()?;
    let mut z = zip::ZipArchive::new(f).ok()?;
    for i in 0..z.len() {
        let Ok(entry) = z.by_index(i) else {
            continue;
        };
        let name = entry.name().to_string();
        for cand in candidates {
            let prefix = format!("{}/", cand.replace('.', "/"));
            if name.starts_with(&prefix) {
                return Some(cand.clone());
            }
        }
    }
    None
}

/// Человекочитаемое имя мода из его метаданных внутри jar (fallback — имя файла).
fn mod_display_name(jar_path: &Path) -> Option<String> {
    let f = File::open(jar_path).ok()?;
    let mut z = zip::ZipArchive::new(f).ok()?;
    let candidates = ["fabric.mod.json", "quilt.mod.json", "META-INF/mods.toml"];
    for name in candidates {
        let Ok(mut entry) = z.by_name(name) else {
            continue;
        };
        let mut buf = Vec::new();
        if entry.read_to_end(&mut buf).is_err() {
            continue;
        }
        let text = String::from_utf8_lossy(&buf);
        let parsed = serde_json::from_str::<serde_json::Value>(&text);
        if let Ok(v) = parsed {
            if let Some(n) = v.get("name").and_then(|n| n.as_str()) {
                let n = n.trim();
                if !n.is_empty() {
                    return Some(n.to_string());
                }
            }
        } else if name.ends_with("mods.toml") {
            for line in text.lines() {
                let t = line.trim();
                if let Some(v) = t.strip_prefix("displayName=") {
                    let v = v.trim().trim_matches('"');
                    if !v.is_empty() {
                        return Some(v.to_string());
                    }
                }
            }
        }
    }
    None
}

/// Находит подозреваемые моды по пакетам из стека, сканируя `mods/*.jar`.
fn find_suspected(game_dir: &Path, packages: &[String]) -> Vec<SuspectedMod> {
    if packages.is_empty() {
        return Vec::new();
    }
    let mods_dir = game_dir.join("mods");
    let Ok(rd) = std::fs::read_dir(&mods_dir) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut matched: HashSet<String> = HashSet::new();
    let mut jars: Vec<PathBuf> = Vec::new();
    for e in rd.flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().to_string();
        if name.ends_with(".jar") && !name.ends_with(".jar.disabled") && p.is_file() {
            jars.push(p);
        }
    }
    for jar in jars {
        let Some(pkg) = jar_first_match(&jar, packages) else {
            continue;
        };
        if !matched.insert(pkg.clone()) {
            continue;
        }
        let fname = jar
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let name = mod_display_name(&jar).unwrap_or_else(|| {
            fname
                .strip_suffix(".jar")
                .unwrap_or(&fname)
                .to_string()
        });
        out.push(SuspectedMod {
            name,
            file: fname,
            package: pkg,
        });
    }
    out
}

/// Классифицирует причину по тексту и списку кандидат-пакетов (если виноват мод).
fn classify(text: &str, exception: &str, packages: &[String]) -> (String, Option<u32>) {
    let hay = format!("{text}\n{exception}").to_lowercase();
    let in_mixin = hay.contains("mixin apply")
        || hay.contains("org.spongepowered.asm.mixin")
        || hay.contains("mixinbooter")
        || exception.to_lowercase().contains("mixin");

    // OutOfMemory / недостаток памяти (и hs_err «insufficient memory»).
    if hay.contains("outofmemoryerror")
        || hay.contains("not enough memory")
        || hay.contains("insufficient memory for the java runtime")
        || hay.contains("native memory allocation")
    {
        return (if in_mixin { "modConflict".into() } else { "oom".into() }, None);
    }

    // Java version: класс/библиотека требует другой мажор.
    if hay.contains("unsupportedclassversionerror")
        || hay.contains("class file version")
        || hay.contains("a java 6.0 jre required")
        || exception.to_lowercase().contains("version")
    {
        return ("javaVersion".into(), Some(0)); // hint обновим ниже
    }

    // Графика / окно.
    if hay.contains("opengl")
        || hay.contains("couldn't init gl")
        || hay.contains("failed to initialize window")
        || hay.contains("glfw")
        || hay.contains("lwjgl")
        || hay.contains("gpu")
    {
        return ("gpu".into(), None);
    }

    // Повреждённый/нечитаемый файл (zip, jar).
    if hay.contains("zipexception")
        || hay.contains("invalid path")
        || hay.contains("couldn't read zip")
        || hay.contains("corrupted")
        || hay.contains("not a zip file")
    {
        return ("corrupt".into(), None);
    }

    // MIXIN / известно, что конфликт.
    if in_mixin {
        return ("modConflict".into(), None);
    }

    // Отсутствующий класс/метод/поле либо рантайм-линковка.
    if hay.contains("noclassdeffounderror")
        || hay.contains("nosuchmethoderror")
        || hay.contains("nosuchfielderror")
        || hay.contains("classnotfoundexception")
        || hay.contains("incompatibleclasschangeerror")
        || hay.contains("linkageerror")
        || hay.contains("exceptionininitializererror")
    {
        return ("mod".into(), None);
    }

    // Есть неигнорированная кандидат-пакет — вероятно, виноват мод.
    if !packages.is_empty() {
        return ("mod".into(), None);
    }

    ("other".into(), None)
}

/// Ищет рекомендуемый мажор Java из версии Minecraft (для javaVersion-совета).
fn java_hint_for(pack_id: &str) -> Option<u32> {
    let game_dir = config::active_game_dir(pack_id).ok()?;
    let index = game_dir.join(".mono-index.json");
    let raw = std::fs::read_to_string(index).ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let mc = v.get("dependencies")?.get("minecraft")?.as_str()?;
    crate::jre::required_java(mc)
}

/// Публичный анализ краш-артефактов сборки. Никогда не паникует и не «падает» —
/// при любой ошибке возвращает пустой результат (has_crash = false).
pub fn analyze_pack(pack_id: &str) -> CrashAnalysis {
    let game_dir = config::active_game_dir(pack_id).ok();
    let Some(game_dir) = game_dir else {
        return CrashAnalysis::default();
    };

    let source = newest_source(&game_dir);
    let (text, rel, is_hs_err) = match source {
        Source::CrashReport(p) => (read_tail(&p, 512 * 1024), p, false),
        Source::HsErr(p) => (read_tail(&p, 512 * 1024), p, true),
        Source::LatestLog(p) => (read_tail(&p, 512 * 1024), p, false),
        Source::None => return CrashAnalysis::default(),
    };
    if text.trim().is_empty() {
        return CrashAnalysis::default();
    }

    let rel_name = rel
        .strip_prefix(&game_dir)
        .ok()
        .map(|r| r.to_string_lossy().into_owned())
        .unwrap_or_else(|| {
            rel.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        });

    let exception = if is_hs_err {
        String::new()
    } else {
        // Первая строка, похожая на исключение.
        text.lines()
            .filter_map(looks_like_exception)
            .next()
            .unwrap_or_default()
    };
    let description = if is_hs_err || !text.contains('\n') {
        String::new()
    } else {
        first_description(&text)
    };

    let packages = extract_packages(&text);
    let (kind, _) = classify(&text, &exception, &packages);
    let hinted = java_hint_for(pack_id);

    // Если это ошибка версии Java — конкретный совет по мажору.
    let mut java_hint = None;
    if kind == "javaVersion" {
        java_hint = hinted;
    }

    CrashAnalysis {
        has_crash: true,
        file: rel_name,
        kind,
        exception,
        description,
        java_hint,
        suspected: find_suspected(&game_dir, &packages),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CRASH_TXT: &str = "\
---- Minecraft Crash Report ----
// Who set us up the TNT?

Time: 8/15/26 10:00:00
Description: Ticking entity

Description:
java.lang.RuntimeException: Ticking entity
\tat net.minecraft.world.level.Level.tick(Level.java:100)
\tat com.example.mymod.WorldHook.tick(WorldHook.java:40)
\tat net.minecraft.server.MinecraftServer.tick(MinecraftServer.java:500)
\tat net.minecraft.server.MinecraftServer.run(MinecraftServer.java:777)
\nA detailed walkthrough of the error, its code path and all known details is as follows:\n...";

    #[test]
    fn parses_description_and_exception() {
        assert_eq!(first_description(CRASH_TXT), "Ticking entity");
        let exc = CRASH_TXT
            .lines()
            .filter_map(looks_like_exception)
            .next()
            .unwrap();
        assert!(exc.starts_with("java.lang.RuntimeException"));
    }

    #[test]
    fn extracts_package_frames() {
        let pkgs = extract_packages(CRASH_TXT);
        assert!(pkgs.contains(&"com.example".to_string()));
        assert!(!pkgs.iter().any(|p| p.starts_with("net.minecraft")));
    }

    #[test]
    fn frame_class_extracts_fqcn() {
        let c = frame_class("\tat com.example.mod.Class.method(Class.java:1)").unwrap();
        assert_eq!(c, "com.example.mod.Class.method");
    }

    #[test]
    fn classifies_oom() {
        let (k, _) = classify(
            "\nOutOfMemoryError\nat com.example.X.foo(X.java:1)",
            "java.lang.OutOfMemoryError: Java heap space",
            &[],
        );
        assert_eq!(k, "oom");
    }

    #[test]
    fn classifies_mixin_conflict() {
        let (k, _) = classify(
            "org.spongepowered.asm.mixin.injection.MixinTargetSelector apply",
            "java.lang.reflect.InvocationTargetException",
            &["com.example".to_string()],
        );
        assert_eq!(k, "modConflict");
    }

    #[test]
    fn classifies_java_version() {
        let (k, _) = classify(
            "java.lang.UnsupportedClassVersionError: Class has been compiled by a more recent version",
            "java.lang.UnsupportedClassVersionError",
            &[],
        );
        assert_eq!(k, "javaVersion");
    }

    #[test]
    fn frame_class_rejects_false_positive() {
        assert!(frame_class("Hello world").is_none());
        assert!(frame_class("\tat java.lang.String.isBlank(String.java)").is_some());
    }
}