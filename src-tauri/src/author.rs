//! Экспорт «авторской» сборки: готовый архив для публикации сборки в GitHub.
//!
//! Помимо `.mrpack` внутри собирается `pack.json` (контракт лаунчера: имя,
//! `boostyBlog`, `minRam`), а также опциональные `servers.json`, `socials.json`,
//! `theme.json`, `README.md` и локальные `banner.png`/`icon.png`. Автор выкладывает
//! содержимое архива как GitHub Release сборки — и её можно добавить по ссылке.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config;
use crate::export;

/// Сервер сборки (`servers.json`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorServer {
    pub name: String,
    pub ip: String,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub desc: Option<String>,
}

/// Соцсеть сборки (`socials.json`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorSocial {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub color: Option<String>,
}

/// Тема лаунчера (`theme.json`, все поля — hex `#rrggbb`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorTheme {
    pub bg: Option<String>,
    pub panel: Option<String>,
    pub input: Option<String>,
    pub border: Option<String>,
    pub tx: Option<String>,
    pub tx_strong: Option<String>,
    pub tx_muted: Option<String>,
    pub accent: Option<String>,
    pub accent_strong: Option<String>,
    pub accent_hover: Option<String>,
    pub accent_deep: Option<String>,
}

/// Удобная конфигурация авторской сборки, заполняемая в мастере экспорта.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorPackConfig {
    pub name: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, rename = "boostyBlog")]
    pub boosty_blog: Option<String>,
    #[serde(default, rename = "minRam")]
    pub min_ram: Option<u32>,
    #[serde(default)]
    pub servers: Vec<AuthorServer>,
    #[serde(default)]
    pub socials: Vec<AuthorSocial>,
    #[serde(default)]
    pub theme: Option<AuthorTheme>,
    /// Язык генерируемого `README.md` (`ru`, `en`, `uk`).
    #[serde(default)]
    pub readme_lang: String,
}

fn write_json<T: Serialize>(dir: &Path, name: &str, value: &T) -> Result<()> {
    fs::write(
        dir.join(name),
        serde_json::to_string_pretty(value).context("Не удалось сериализовать JSON")?,
    )?;
    Ok(())
}

/// Собирает архив авторской сборки по пути `dest`.
///
/// `include` — список путей папки игры для `overrides` (пустой = всё по умолчанию),
/// как в обычном экспорте. `version_id` — конкретная версия сборки (пустая = активная).
pub fn export_author_pack(
    pack_id: &str,
    version_id: &str,
    dest: &Path,
    include: &[String],
    config: &AuthorPackConfig,
) -> Result<()> {
    let name = if config.name.trim().is_empty() {
        "pack".to_string()
    } else {
        config.name.trim().replace(' ', "-")
    };

    let tmp = export::temp_dir(pack_id)?;
    let tmp = tmp.join("author");
    fs::create_dir_all(&tmp)?;
    let result = (|| -> Result<()> {
        // 1. Сама сборка в формате .mrpack (+ overrides).
        export::export_mrpack(
            pack_id,
            version_id,
            &tmp.join(format!("{name}.mrpack")),
            include,
            &config.name,
            "",
        )?;

        // 2. pack.json — контракт лаунчера (имя, платность Boosty, minRam).
        write_json(
            &tmp,
            "pack.json",
            &serde_json::json!({
                "name": config.name.trim(),
                "boostyBlog": config.boosty_blog,
                "minRam": config.min_ram,
            }),
        )?;

        // 3. Опциональные метаданные репозитория.
        if !config.servers.is_empty() {
            write_json(&tmp, "servers.json", &config.servers)?;
        }
        if !config.socials.is_empty() {
            write_json(&tmp, "socials.json", &config.socials)?;
        }
        if let Some(theme) = &config.theme {
            if theme.accent.is_some() {
                write_json(&tmp, "theme.json", theme)?;
            }
        }

        // 4. Локальные баннер/иконка, если есть у сборки.
        copy_if_exists(&config::pack_dir(pack_id)?.join("banner.png"), &tmp.join("banner.png"));
        copy_if_exists(&config::pack_dir(pack_id)?.join("icon.png"), &tmp.join("icon.png"));

        // 5. README с шагами публикации.
        fs::write(tmp.join("README.md"), readme(config, &name))?;

        // 6. Упаковываем всё в один zip.
        export::zip_dir(&tmp, dest)
    })();
    let _ = fs::remove_dir_all(&tmp);
    result
}

fn copy_if_exists(src: &Path, dst: &Path) {
    if let Ok(true) = fs::metadata(src).map(|m| m.is_file()) {
        let _ = fs::copy(src, dst);
    }
}

/// Генерирует README в языке `config.readme_lang` (ru/en/uk, по умолчанию ru).
fn readme(config: &AuthorPackConfig, mrpack_name: &str) -> String {
    let author_line = if config.author.trim().is_empty() {
        String::new()
    } else {
        format!("\n**{author_label}:** {}\n", config.author.trim(), author_label = match config.readme_lang.as_str() {
            "en" => "Author",
            "uk" => "Автор",
            _ => "Автор",
        })
    };
    let desc_line = config
        .description
        .as_deref()
        .map(|d| format!("\n{d}\n"))
        .unwrap_or_default();
    let repo = repo_name(config);

    let (rel, share) = match config.readme_lang.as_str() {
        "en" => (
            format!("3. Create a GitHub Release and attach the `{mrpack_name}.mrpack` and `pack.json` files to it.\n"),
            format!("4. Share the pack link:\n\n   https://n1orio.github.io/mono-launcher/?url={repo}\n\n   The link will open the launcher and add the pack automatically.\n"),
        ),
        "uk" => (
            format!("3. Створіть GitHub Release і прикріпіть до нього файли `{mrpack_name}.mrpack` та `pack.json`.\n"),
            format!("4. Поділіться посиланням на збірку:\n\n   https://n1orio.github.io/mono-launcher/?url={repo}\n\n   Посилання відкриє лаунчер та автоматично додасть збірку.\n"),
        ),
        _ => (
            format!("3. Создайте GitHub Release и прикрепите к нему файл `{mrpack_name}.mrpack` и `pack.json`.\n"),
            format!("4. Поделитесь ссылкой на сборку:\n\n   https://n1orio.github.io/mono-launcher/?url={repo}\n\n   Ссылка откроет лаунчер и автоматически добавит сборку.\n"),
        ),
    };

    let (howto, steps, boosty) = match config.readme_lang.as_str() {
        "en" => (
            "## How to publish\n",
            "1. Create a repository on GitHub.\n2. Upload the contents of this archive into the repository.\n",
            "5. Paid packs (Boosty) and the minimum RAM are configured in `pack.json`.\n",
        ),
        "uk" => (
            "## Як опублікувати\n",
            "1. Створіть репозиторій на GitHub.\n2. Завантажте вміст цього архіву в репозиторій.\n",
            "5. Платні збірки (Boosty) і мінімальна оперативна пам'ять налаштовуються у `pack.json`.\n",
        ),
        _ => (
            "## Как опубликовать\n",
            "1. Создайте репозиторий на GitHub.\n2. Загрузите содержимое этого архива в репозиторий.\n",
            "5. Платные сборки (Boosty) и минимальная оперативка настраиваются в `pack.json`.\n",
        ),
    };

    format!(
        "# {}\n{}{}\n{}{}{}{}{}",
        config.name.trim(),
        author_line,
        desc_line,
        howto,
        steps,
        rel,
        share,
        boosty,
    )
}

/// Ник владельца из конфига (из строки `owner`), иначе — имя сборки.
fn repo_name(config: &AuthorPackConfig) -> String {
    let owner = config.author.trim();
    if owner.is_empty() {
        "YOUR-USERNAME/YOUR-REPO".to_string()
    } else {
        format!("{owner}/YOUR-REPO")
    }
}

/// Команда экспорта авторской сборки.
#[tauri::command]
pub fn export_author_pack_command(
    pack_id: String,
    version_id: String,
    dest_path: String,
    include: Vec<String>,
    config: AuthorPackConfig,
) -> Result<(), String> {
    let dest = PathBuf::from(&dest_path);
    export_author_pack(&pack_id, &version_id, &dest, &include, &config).map_err(|e| e.to_string())
}